//! `rank` and `select` over 2^20 bits: hakmem's rank9 directory against
//! the directories people use today (sux's rank9 and select9, sucds's
//! `Rank9Sel`, vers-vecs's `RsVec`), on a dense slice (half the bits
//! set) and a sparse one (one bit in 64). 1024 random queries each; the
//! build is not timed.

// criterion_group! expands to an undocumented pub fn.
#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hakmem::rank9::Rank9;
use sucds::bit_vectors::{Rank as _, Rank9Sel, Select as _};
use sux::bits::BitVec;
use sux::rank_sel::{Rank9 as SuxRank9, Select9};
use sux::traits::{Rank as _, Select as _};
use vers_vecs::{BitVec as VersBitVec, RsVec};

/// 2^20 bits.
const WORDS: usize = 1 << 14;
const QUERIES: usize = 1024;

fn xorshift(mut s: u64) -> impl FnMut() -> u64 {
    move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    }
}

/// Random words; each extra `density_shift` halves the density.
fn words(density_shift: u32) -> Vec<u64> {
    let mut next = xorshift(0x9E37_79B9_7F4A_7C15);
    (0..WORDS)
        .map(|_| {
            let mut w = next();
            for _ in 0..density_shift {
                w &= next();
            }
            w
        })
        .collect()
}

const fn bit(bits: &[u64], i: usize) -> bool {
    bits[i / 64] >> (i % 64) & 1 == 1
}

fn bench_rank_select(c: &mut Criterion) {
    for (label, shift) in [("dense", 0u32), ("sparse", 6)] {
        let bits = words(shift);
        let n = bits.len() * 64;

        let mut counts = vec![0; Rank9::counts_len(bits.len())];
        let mut select = vec![0; Rank9::select_len(bits.len())];
        Rank9::build(&bits, &mut counts, &mut select);
        let ours = Rank9::new(&bits, &counts, &select);
        let ones = ours.count_ones();

        let bv: BitVec = (0..n).map(|i| bit(&bits, i)).collect();
        let sux_rank = SuxRank9::new(bv.clone());
        let sux_select = Select9::new(SuxRank9::new(bv));
        let sucds = Rank9Sel::from_bits((0..n).map(|i| bit(&bits, i))).select1_hints();
        let vers = RsVec::from_bit_vec(VersBitVec::from_limbs(&bits));

        let mut next = xorshift(0x2545_F491_4F6C_DD1D);
        // `n` and `ones` fit a u64 and the remainders fit a usize.
        #[allow(clippy::cast_possible_truncation)]
        let positions: Vec<usize> = (0..QUERIES).map(|_| (next() % n as u64) as usize).collect();
        #[allow(clippy::cast_possible_truncation)]
        let ranks: Vec<usize> = (0..QUERIES)
            .map(|_| (next() % ones as u64) as usize)
            .collect();

        {
            let mut group = c.benchmark_group("rank");
            group.bench_with_input(BenchmarkId::new("hakmem", label), &positions, |b, q| {
                b.iter(|| {
                    q.iter()
                        .map(|&i| ours.rank(i))
                        .fold(0usize, usize::wrapping_add)
                });
            });
            group.bench_with_input(BenchmarkId::new("sux", label), &positions, |b, q| {
                b.iter(|| {
                    q.iter()
                        .map(|&i| sux_rank.rank(i))
                        .fold(0usize, usize::wrapping_add)
                });
            });
            group.bench_with_input(BenchmarkId::new("sucds", label), &positions, |b, q| {
                b.iter(|| {
                    q.iter()
                        .map(|&i| sucds.rank1(i).unwrap())
                        .fold(0usize, usize::wrapping_add)
                });
            });
            group.bench_with_input(BenchmarkId::new("vers", label), &positions, |b, q| {
                b.iter(|| {
                    q.iter()
                        .map(|&i| vers.rank1(i))
                        .fold(0usize, usize::wrapping_add)
                });
            });
            group.finish();
        }
        {
            let mut group = c.benchmark_group("select");
            group.bench_with_input(BenchmarkId::new("hakmem", label), &ranks, |b, q| {
                b.iter(|| {
                    q.iter()
                        .map(|&k| ours.select(k).unwrap())
                        .fold(0usize, usize::wrapping_add)
                });
            });
            group.bench_with_input(BenchmarkId::new("sux", label), &ranks, |b, q| {
                b.iter(|| {
                    q.iter()
                        .map(|&k| sux_select.select(k).unwrap())
                        .fold(0usize, usize::wrapping_add)
                });
            });
            group.bench_with_input(BenchmarkId::new("sucds", label), &ranks, |b, q| {
                b.iter(|| {
                    q.iter()
                        .map(|&k| sucds.select1(k).unwrap())
                        .fold(0usize, usize::wrapping_add)
                });
            });
            group.bench_with_input(BenchmarkId::new("vers", label), &ranks, |b, q| {
                b.iter(|| {
                    q.iter()
                        .map(|&k| vers.select1(k))
                        .fold(0usize, usize::wrapping_add)
                });
            });
            group.finish();
        }
        black_box(ours);
    }
}

criterion_group!(benches, bench_rank_select);
criterion_main!(benches);

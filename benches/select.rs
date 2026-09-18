//! `select` on `u64`: whatever `Word::select_lowest` compiles to for
//! this target (PDEP with `bmi2`, broadword otherwise) against the
//! always-portable broadword function and a clear-lowest-bit loop.
//!
//! Run with and without `RUSTFLAGS="-C target-feature=+bmi2"`, and with
//! `--features portable`, to see all three configurations.

// criterion_group! expands to an undocumented pub fn.
#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use hakmem::prelude::*;
use hakmem::word::select_broadword64;

/// Deterministic xorshift; larger `density_shift` = sparser words.
fn words(n: usize, density_shift: u32) -> Vec<u64> {
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    let mut next = move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };
    (0..n)
        .map(|_| {
            let mut w = next();
            for _ in 0..density_shift {
                w &= next();
            }
            w | 1
        })
        .collect()
}

fn select_loop(mut x: u64, k: u32) -> u32 {
    for _ in 0..k {
        x &= x - 1;
    }
    x.trailing_zeros()
}

fn bench_select(c: &mut Criterion) {
    let mut group = c.benchmark_group("select64");
    for (label, shift) in [("dense", 0u32), ("sparse", 2)] {
        let ws = words(1024, shift);
        // Rank in the middle of each word's population.
        let pairs: Vec<(u64, u32)> = ws.iter().map(|&w| (w, w.count_ones() / 2)).collect();

        group.bench_with_input(BenchmarkId::new("target", label), &pairs, |b, pairs| {
            b.iter(|| {
                let mut acc = 0u32;
                for &(w, k) in black_box(pairs) {
                    acc = acc.wrapping_add(w.select(k).unwrap_or(64));
                }
                acc
            });
        });
        group.bench_with_input(BenchmarkId::new("broadword", label), &pairs, |b, pairs| {
            b.iter(|| {
                let mut acc = 0u32;
                for &(w, k) in black_box(pairs) {
                    acc = acc.wrapping_add(select_broadword64(w, k));
                }
                acc
            });
        });
        group.bench_with_input(BenchmarkId::new("loop", label), &pairs, |b, pairs| {
            b.iter(|| {
                let mut acc = 0u32;
                for &(w, k) in black_box(pairs) {
                    acc = acc.wrapping_add(select_loop(w, k));
                }
                acc
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_select);
criterion_main!(benches);

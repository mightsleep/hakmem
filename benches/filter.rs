//! Filtering a bitmap by a mask bitmap, the way arrow-rs
//! `filter_bits_compress` does it: compact each 64-bit word by its mask
//! word, then pack the results end to end. Measures the compact
//! fallbacks proposed for targets without PEXT (arrow-rs #11213) against
//! the Hacker's Delight 7-4 network, scalar, split into a reusable plan,
//! and over eight words at once as SIMD lanes (plain arrays, left to the
//! autovectoriser, so stable Rust), on uniform and clustered masks.
//!
//! Build without `bmi2` to see the fallbacks as a target without PEXT
//! runs them; `target` is whatever `u64::compact` compiles to.

// criterion_group! expands to an undocumented pub fn.
#![allow(missing_docs)]

use std::hint::black_box;
use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use hakmem::prelude::*;

/// Words per filtered buffer: 64 Ki rows, three buffers fit in L2.
const WORDS: usize = 1024;

const fn xorshift(s: &mut u64) -> u64 {
    *s ^= *s << 13;
    *s ^= *s >> 7;
    *s ^= *s << 17;
    *s
}

/// Each bit set independently with probability `num / den`.
fn uniform(num: u64, den: u64, s: &mut u64) -> Vec<u64> {
    (0..WORDS)
        .map(|_| (0..64).fold(0, |w, i| w | u64::from(xorshift(s) % den < num) << i))
        .collect()
}

/// A two-state Markov stream of density `num / den` whose runs of ones
/// average `run` bits; runs of zeros scale to keep the density.
fn clustered(num: u64, den: u64, run: u64, s: &mut u64) -> Vec<u64> {
    // Probabilities in 32-bit fixed point: leave a run of ones with 1 / run,
    // a run of zeros with 1 / (its mean length, run * (den - num) / num).
    let one = 1u64 << 32;
    let (leave_ones, leave_zeros) = (one / run, one * num / (run * (den - num)));
    let mut on = xorshift(s) & 0xFFFF_FFFF < one * num / den;
    (0..WORDS)
        .map(|_| {
            (0..64).fold(0, |w, i| {
                let bit = u64::from(on) << i;
                let t = xorshift(s) & 0xFFFF_FFFF;
                on = if on { t >= leave_ones } else { t < leave_zeros };
                w | bit
            })
        })
        .collect()
}

// ---------------------------------------------------------------------
// Compact fallbacks, one word at a time
// ---------------------------------------------------------------------

/// arrow-rs today: one step per set bit of `mask`, branch-free inside.
fn arrow_loop(value: u64, mask: u64) -> u64 {
    let (mut mask, mut result, mut dest) = (mask, 0, 1);
    while mask != 0 {
        let rest = mask & (mask - 1);
        let keep = u64::from(value & (mask ^ rest) != 0);
        result |= dest & keep.wrapping_neg();
        dest <<= 1;
        mask = rest;
    }
    result
}

/// #11213, first proposal: walk the kept bits or the dropped ones,
/// whichever are fewer.
fn arrow_hybrid(value: u64, mask: u64) -> u64 {
    let kept = mask.count_ones();
    if kept > 32 {
        return drop_zeros(value, mask, kept);
    }
    arrow_loop(value, mask)
}

/// The dense half of the hybrid: remove the dropped bits, highest first so
/// the lower ones stay put. One step per clear bit of `mask`.
fn drop_zeros(mut value: u64, mask: u64, kept: u32) -> u64 {
    let mut dropped = !mask;
    while dropped != 0 {
        let below = (1 << dropped.ilog2()) - 1;
        value = (value & below) | ((value >> 1) & !below);
        dropped &= below;
    }
    // `kept == 0` would shift by 64; then `value` is already all dropped.
    value & u64::MAX.checked_shr(64 - kept).unwrap_or(0)
}

/// `NIBBLE[mask][value]` is the compact of a 4-bit value by a 4-bit mask.
static NIBBLE: [[u8; 16]; 16] = {
    let mut t = [[0; 16]; 16];
    let mut m = 0u8;
    while m < 16 {
        let mut v = 0u8;
        while v < 16 {
            let (mut r, mut d, mut b) = (0u8, 0, 0);
            while b < 4 {
                if m >> b & 1 == 1 {
                    r |= (v >> b & 1) << d;
                    d += 1;
                }
                b += 1;
            }
            t[m as usize][v as usize] = r;
            v += 1;
        }
        m += 1;
    }
    t
};

/// #11213, second proposal: sixteen nibbles through a 256-byte table.
fn arrow_nibble(value: u64, mask: u64) -> u64 {
    let (mut result, mut shift) = (0, 0);
    for i in 0..16 {
        let m = (mask >> (i * 4) & 0xF) as usize;
        let v = (value >> (i * 4) & 0xF) as usize;
        result |= u64::from(NIBBLE[m][v]) << shift;
        shift += m.count_ones();
    }
    result
}

// ---------------------------------------------------------------------
// Hacker's Delight 7-4 as a plan: the network's move masks depend on the
// mask only, so they are computed once and applied to every buffer the
// mask filters. Round `s` moves a kept bit down by 2^s exactly when bit
// `s` of its displacement (the zeros below it) is set; the plan is that
// displacement, bit-sliced. The same masks run backwards are `expand`.
// ---------------------------------------------------------------------

#[derive(Clone, Copy)]
struct CompactPlan {
    mask: u64,
    moves: [u64; 6],
}

impl CompactPlan {
    #[inline]
    fn new(mask: u64) -> Self {
        let mut zeros = !mask;
        let mut moves = [0; 6];
        for (s, m) in moves.iter_mut().enumerate() {
            let n = 1 << s;
            // Parity of the zeros below each bit, in strides of `n`.
            let mut parity = zeros;
            let mut len = n;
            while len < 64 {
                parity ^= parity << len;
                len <<= 1;
            }
            *m = parity;
            zeros &= !parity;
            zeros ^= zeros >> n;
        }
        Self { mask, moves }
    }

    #[inline]
    fn apply(&self, value: u64) -> u64 {
        let mut x = value & self.mask;
        for (s, &m) in self.moves.iter().enumerate() {
            let q = x & m;
            x ^= q ^ (q >> (1 << s));
        }
        x
    }
}

fn plan_once(value: u64, mask: u64) -> u64 {
    CompactPlan::new(mask).apply(value)
}

/// The network over eight words at once. Every step is an and, a xor or
/// a shift by a constant shared by all lanes, so the loops over `i`
/// vectorise: two lanes a register on SSE2 and NEON, eight on AVX-512.
#[inline]
fn compact8(value: &[u64; 8], mask: &[u64; 8]) -> [u64; 8] {
    let mut zeros = mask.map(|m| !m);
    let mut x = [0; 8];
    for i in 0..8 {
        x[i] = value[i] & mask[i];
    }
    for s in 0..6 {
        let n = 1 << s;
        let mut parity = zeros;
        let mut len = n;
        while len < 64 {
            for p in &mut parity {
                *p ^= *p << len;
            }
            len <<= 1;
        }
        for i in 0..8 {
            let q = x[i] & parity[i];
            x[i] ^= q ^ (q >> n);
            zeros[i] &= !parity[i];
            zeros[i] ^= zeros[i] >> n;
        }
    }
    x
}

// ---------------------------------------------------------------------
// The whole filter
// ---------------------------------------------------------------------

/// arrow-rs `Packer`: appends the low `count` bits of each result.
struct Packer<'a> {
    out: &'a mut [u64],
    idx: usize,
    current: u64,
    filled: u32,
}

impl Packer<'_> {
    #[inline]
    const fn push(&mut self, bits: u64, count: u32) {
        self.current |= bits << self.filled;
        let total = self.filled + count;
        if total < 64 {
            self.filled = total;
        } else {
            self.out[self.idx] = self.current;
            self.idx += 1;
            // `bits >> (64 - filled)`, with `filled == 0` shifting all out.
            self.current = (bits >> 1) >> (63 - self.filled);
            self.filled = total - 64;
        }
    }

    const fn finish(self) {
        if self.filled > 0 {
            self.out[self.idx] = self.current;
        }
    }
}

#[inline]
fn filter_serial(values: &[u64], masks: &[u64], out: &mut [u64], f: impl Fn(u64, u64) -> u64) {
    let mut packer = Packer {
        out,
        idx: 0,
        current: 0,
        filled: 0,
    };
    for (&v, &m) in values.iter().zip(masks) {
        packer.push(f(v, m), m.count_ones());
    }
    packer.finish();
}

/// Two phases a block: compact eight words as lanes, then pack them.
/// Only the packer is serial, and it is a few operations a word.
fn filter_lanes(values: &[u64], masks: &[u64], out: &mut [u64]) {
    let mut packer = Packer {
        out,
        idx: 0,
        current: 0,
        filled: 0,
    };
    let (vs, v_rest) = values.as_chunks::<8>();
    let (ms, m_rest) = masks.as_chunks::<8>();
    for (v, m) in vs.iter().zip(ms) {
        let bits = compact8(v, m);
        for i in 0..8 {
            packer.push(bits[i], m[i].count_ones());
        }
    }
    for (&v, &m) in v_rest.iter().zip(m_rest) {
        packer.push(plan_once(v, m), m.count_ones());
    }
    packer.finish();
}

/// Bits walked in a block of eight words at or under which walking beats
/// the lanes. A word walks the smaller of its kept and dropped sides, at
/// about half a nanosecond a bit on Zen 5, and a lane word costs about
/// 4.4, flat; the costs add, so the block's `sum(min(k, 64 - k))` is the
/// whole criterion. Empty and full words count zero, as they should.
const WALK: u32 = 56;

/// Per block of eight words: walk each word from its sparser side, or run
/// the lanes, whichever the block's popcounts say is cheaper. One branch
/// a block, and the packer needs the counts anyway.
fn filter_adaptive(values: &[u64], masks: &[u64], out: &mut [u64]) {
    let mut packer = Packer {
        out,
        idx: 0,
        current: 0,
        filled: 0,
    };
    let (vs, v_rest) = values.as_chunks::<8>();
    let (ms, m_rest) = masks.as_chunks::<8>();
    for (v, m) in vs.iter().zip(ms) {
        let counts = m.map(u64::count_ones);
        let walk: u32 = counts.iter().map(|&k| k.min(64 - k)).sum();
        if walk <= WALK {
            for i in 0..8 {
                let bits = if counts[i] > 32 {
                    drop_zeros(v[i], m[i], counts[i])
                } else {
                    arrow_loop(v[i], m[i])
                };
                packer.push(bits, counts[i]);
            }
        } else {
            let bits = compact8(v, m);
            for i in 0..8 {
                packer.push(bits[i], counts[i]);
            }
        }
    }
    for (&v, &m) in v_rest.iter().zip(m_rest) {
        packer.push(plan_once(v, m), m.count_ones());
    }
    packer.finish();
}

type Filter = fn(&[u64], &[u64], &mut [u64]);

const FILTERS: [(&str, Filter); 8] = [
    ("target", |v, m, o| filter_serial(v, m, o, u64::compact)),
    ("arrow_loop", |v, m, o| filter_serial(v, m, o, arrow_loop)),
    ("arrow_hybrid", |v, m, o| {
        filter_serial(v, m, o, arrow_hybrid);
    }),
    ("arrow_nibble", |v, m, o| {
        filter_serial(v, m, o, arrow_nibble);
    }),
    ("broadword", |v, m, o| {
        filter_serial(v, m, o, hakmem::word::compress_broadword::<u64>);
    }),
    ("plan", |v, m, o| filter_serial(v, m, o, plan_once)),
    ("lanes8", filter_lanes),
    ("adaptive", filter_adaptive),
];

fn masks() -> Vec<(String, Vec<u64>)> {
    let mut s = 0x9E37_79B9_7F4A_7C15;
    let mut cases = Vec::new();
    for (num, den) in [(1, 32), (1, 8), (1, 2), (7, 8), (31, 32)] {
        cases.push((format!("{num}of{den}/uniform"), uniform(num, den, &mut s)));
    }
    for (num, den, run) in [
        (1, 32, 64),
        (1, 2, 8),
        (1, 2, 64),
        (7, 8, 64),
        (1, 8, 1024),
        (1, 2, 1024),
    ] {
        cases.push((
            format!("{num}of{den}/run{run}"),
            clustered(num, den, run, &mut s),
        ));
    }
    cases
}

fn bench_filter(c: &mut Criterion) {
    let mut s = 0x2545_F491_4F6C_DD1D;
    let values: Vec<u64> = (0..WORDS).map(|_| xorshift(&mut s)).collect();
    let mut group = c.benchmark_group("filter_bits");
    group.throughput(Throughput::Elements(64 * WORDS as u64));
    group.warm_up_time(Duration::from_millis(300));
    group.measurement_time(Duration::from_secs(1));
    for (label, masks) in masks() {
        let mut expected = vec![0; WORDS + 1];
        FILTERS[0].1(&values, &masks, &mut expected);
        for (name, filter) in FILTERS {
            let mut out = vec![0; WORDS + 1];
            filter(&values, &masks, &mut out);
            assert_eq!(out, expected, "{name} on {label}");
            group.bench_with_input(BenchmarkId::new(name, &label), &masks, |b, masks| {
                b.iter(|| filter(black_box(&values), black_box(masks), &mut out));
            });
        }
    }
    group.finish();
}

/// One mask, `k` buffers: the values and the validity of a column, or
/// every column of a batch. The plan is paid once per mask word.
fn bench_plan_reuse(c: &mut Criterion) {
    let mut s = 0x2545_F491_4F6C_DD1D;
    let masks = uniform(1, 2, &mut s);
    let mut group = c.benchmark_group("plan_reuse");
    group.warm_up_time(Duration::from_millis(300));
    group.measurement_time(Duration::from_secs(1));
    for k in [1usize, 2, 8] {
        let values: Vec<u64> = (0..WORDS * k).map(|_| xorshift(&mut s)).collect();
        group.throughput(Throughput::Elements((64 * WORDS * k) as u64));
        group.bench_with_input(BenchmarkId::new("plan", k), &k, |b, &k| {
            b.iter(|| {
                let mut acc = 0u64;
                for (i, &m) in black_box(&masks).iter().enumerate() {
                    let plan = CompactPlan::new(m);
                    for v in &values[i * k..(i + 1) * k] {
                        acc ^= plan.apply(*v);
                    }
                }
                acc
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_filter, bench_plan_reuse);
criterion_main!(benches);

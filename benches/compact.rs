//! `compact` / `expand` on `u64`: whatever `Word::pext` / `Word::pdep`
//! compile to for this target (PEXT / PDEP with `bmi2`, broadword
//! otherwise), the always-portable broadword definitions, and a loop
//! over the mask's set bits, on dense and on sparse masks.
//!
//! Run with and without `RUSTFLAGS="-C target-feature=+bmi2,+pclmulqdq"`
//! to see the hardware, the scan-assisted and the plain broadword paths.

// criterion_group! expands to an undocumented pub fn.
#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hakmem::prelude::*;
use hakmem::word::{compress_broadword, expand_broadword};

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
            w
        })
        .collect()
}

const fn compress_loop(x: u64, mut mask: u64) -> u64 {
    let mut out = 0;
    let mut k = 0;
    while mask != 0 {
        if x >> mask.trailing_zeros() & 1 != 0 {
            out |= 1 << k;
        }
        k += 1;
        mask &= mask - 1;
    }
    out
}

const fn expand_loop(x: u64, mut mask: u64) -> u64 {
    let mut out = 0;
    let mut k = 0;
    while mask != 0 {
        if x >> k & 1 != 0 {
            out |= 1 << mask.trailing_zeros();
        }
        k += 1;
        mask &= mask - 1;
    }
    out
}

fn bench_compact(c: &mut Criterion) {
    for (op, target, broadword, looped) in [
        (
            "compact64",
            u64::compact as fn(u64, u64) -> u64,
            compress_broadword::<u64> as fn(u64, u64) -> u64,
            compress_loop as fn(u64, u64) -> u64,
        ),
        (
            "expand64",
            u64::expand,
            expand_broadword::<u64>,
            expand_loop,
        ),
    ] {
        let mut group = c.benchmark_group(op);
        for (label, shift) in [("dense", 0u32), ("sparse", 2)] {
            let xs = words(1024, 0);
            let ms = words(1024, shift);
            let pairs: Vec<(u64, u64)> = xs.into_iter().zip(ms).collect();
            for (name, f) in [
                ("target", target),
                ("broadword", broadword),
                ("loop", looped),
            ] {
                group.bench_with_input(BenchmarkId::new(name, label), &pairs, |b, pairs| {
                    b.iter(|| {
                        let mut acc = 0u64;
                        for &(x, m) in black_box(pairs) {
                            acc = acc.wrapping_add(f(x, m));
                        }
                        acc
                    });
                });
            }
        }
        group.finish();
    }
}

criterion_group!(benches, bench_compact);
criterion_main!(benches);

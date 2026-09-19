//! hakmem against the crates people use today.
//!
//! - `select64`: `Bits::select` vs `broadword::select1_raw` (Tov's crate, used by `succinct`).
//! - `edit_distance`: `myers::edit_distance` vs `strsim::levenshtein` (chars, the default choice)
//!   and `triple_accel::levenshtein` / `levenshtein_exp` (SIMD Myers, the fast choice).
//!
//! Run plain and with `RUSTFLAGS="-C target-feature=+bmi2,+pclmulqdq"`;
//! `triple_accel` detects AVX2 at run time on its own.

// criterion_group! expands to an undocumented pub fn.
#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hakmem::myers::edit_distance;
use hakmem::{Bits, Wide};

fn xorshift(mut s: u64) -> impl FnMut() -> u64 {
    move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    }
}

fn bench_select(c: &mut Criterion) {
    let mut next = xorshift(0x9E37_79B9_7F4A_7C15);
    let pairs: Vec<(u64, u32)> = (0..1024)
        .map(|_| {
            let w = next() | 1;
            (w, w.count_ones() / 2)
        })
        .collect();

    let mut group = c.benchmark_group("select64_vs_broadword");
    group.bench_function("hakmem", |b| {
        b.iter(|| {
            let mut acc = 0u32;
            for &(w, k) in black_box(&pairs) {
                acc = acc.wrapping_add(w.select(k).unwrap_or(64));
            }
            acc
        });
    });
    group.bench_function("broadword::select1_raw", |b| {
        b.iter(|| {
            let mut acc = 0usize;
            for &(w, k) in black_box(&pairs) {
                acc = acc.wrapping_add(broadword::select1_raw(k as usize, w));
            }
            acc
        });
    });
    group.finish();
}

/// Random lowercase ASCII, so `strsim` (chars) and byte APIs see the
/// same strings.
fn text(len: usize, seed: u64) -> Vec<u8> {
    let mut next = xorshift(seed);
    (0..len).map(|_| b'a' + (next() % 8) as u8).collect()
}

fn bench_edit_distance(c: &mut Criterion) {
    let mut group = c.benchmark_group("edit_distance");
    for (m, n) in [
        (16usize, 64usize),
        (32, 256),
        (64, 256),
        (64, 1024),
        (128, 1024),
        (256, 1024),
        (512, 1024),
    ] {
        let p = text(m, 1);
        let t = text(n, 2);
        let (ps, ts) = (
            std::str::from_utf8(&p).unwrap().to_owned(),
            std::str::from_utf8(&t).unwrap().to_owned(),
        );
        let id = format!("m{m}_n{n}");

        if m <= 64 {
            group.bench_with_input(
                BenchmarkId::new("hakmem_u64", &id),
                &(&p, &t),
                |b, (p, t)| {
                    b.iter(|| edit_distance::<u64>(black_box(p), black_box(t)));
                },
            );
        }
        if m <= 128 {
            group.bench_with_input(
                BenchmarkId::new("hakmem_u128", &id),
                &(&p, &t),
                |b, (p, t)| {
                    b.iter(|| edit_distance::<u128>(black_box(p), black_box(t)));
                },
            );
        }
        if m <= 256 {
            group.bench_with_input(
                BenchmarkId::new("hakmem_wide4", &id),
                &(&p, &t),
                |b, (p, t)| {
                    b.iter(|| edit_distance::<Wide<4>>(black_box(p), black_box(t)));
                },
            );
        }
        group.bench_with_input(
            BenchmarkId::new("hakmem_wide8", &id),
            &(&p, &t),
            |b, (p, t)| {
                b.iter(|| edit_distance::<Wide<8>>(black_box(p), black_box(t)));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("triple_accel", &id),
            &(&p, &t),
            |b, (p, t)| {
                b.iter(|| triple_accel::levenshtein(black_box(p), black_box(t)));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("triple_accel_exp", &id),
            &(&p, &t),
            |b, (p, t)| {
                b.iter(|| triple_accel::levenshtein_exp(black_box(p), black_box(t)));
            },
        );
        group.bench_with_input(BenchmarkId::new("strsim", &id), &(&ps, &ts), |b, (p, t)| {
            b.iter(|| strsim::levenshtein(black_box(p), black_box(t)));
        });
    }
    group.finish();
}

/// `triple_accel`'s design point: two strings of the same length that
/// differ in a handful of edits, with a known bound `k`. `hakmem`
/// always computes the full distance; the banded SIMD only touches the
/// diagonal band.
fn bench_similar_strings(c: &mut Criterion) {
    let mut group = c.benchmark_group("similar_strings");
    for (len, edits) in [(64usize, 4usize), (128, 8), (512, 8), (512, 32)] {
        let a = text(len, 7);
        let mut b = a.clone();
        let mut next = xorshift(11);
        for _ in 0..edits {
            let i = usize::try_from(next() % u64::try_from(len).unwrap()).unwrap();
            b[i] = b'a' + (next() % 8) as u8;
        }
        let (a_s, b_s) = (
            std::str::from_utf8(&a).unwrap().to_owned(),
            std::str::from_utf8(&b).unwrap().to_owned(),
        );
        let id = format!("len{len}_edits{edits}");
        let k = u32::try_from(edits).unwrap() * 2;

        if len <= 64 {
            group.bench_with_input(
                BenchmarkId::new("hakmem_u64", &id),
                &(&a, &b),
                |bb, (a, b)| {
                    bb.iter(|| edit_distance::<u64>(black_box(a), black_box(b)));
                },
            );
        }
        if len <= 128 {
            group.bench_with_input(
                BenchmarkId::new("hakmem_u128", &id),
                &(&a, &b),
                |bb, (a, b)| {
                    bb.iter(|| edit_distance::<u128>(black_box(a), black_box(b)));
                },
            );
        }
        group.bench_with_input(
            BenchmarkId::new("hakmem_wide8", &id),
            &(&a, &b),
            |bb, (a, b)| {
                bb.iter(|| edit_distance::<Wide<8>>(black_box(a), black_box(b)));
            },
        );
        group.bench_with_input(
            // k is not a free choice: twice the edits, one per size, so the id
            // carries no k; the README names the column the same way.
            BenchmarkId::new("triple_accel_simd_k", &id),
            &(&a, &b),
            |bb, (a, b)| {
                bb.iter(|| {
                    triple_accel::levenshtein::levenshtein_simd_k(black_box(a), black_box(b), k)
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("triple_accel_exp", &id),
            &(&a, &b),
            |bb, (a, b)| {
                bb.iter(|| triple_accel::levenshtein_exp(black_box(a), black_box(b)));
            },
        );
        group.bench_with_input(
            BenchmarkId::new("strsim", &id),
            &(&a_s, &b_s),
            |bb, (a, b)| {
                bb.iter(|| strsim::levenshtein(black_box(a), black_box(b)));
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_select,
    bench_edit_distance,
    bench_similar_strings
);
criterion_main!(benches);

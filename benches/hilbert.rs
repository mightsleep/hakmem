//! Hilbert curve on `u64` (32 levels, `u32` coordinates): the two-scan
//! decode and the Kogge–Stone encode against the crates people use
//! (`fast_hilbert`: a 512-byte transition table, three levels a step;
//! `lindel`: Skilling's transpose, one bit a step), the textbook
//! `d2xy` / `xy2d` loops and the four-state machine in
//! `laws::reference`, and a Morton decode / encode for the price of
//! the frames.
//!
//! Run with and without `RUSTFLAGS="-C target-feature=+pclmulqdq"`: the
//! decode's two suffix XORs are one CLMUL each with it, six operations
//! each without.

// criterion_group! expands to an undocumented pub fn.
#![allow(missing_docs)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use hakmem::laws::reference;
use hakmem::prelude::*;

const N: usize = 1024;

/// Deterministic xorshift.
fn words() -> Vec<u64> {
    let mut s = 0x9E37_79B9_7F4A_7C15u64;
    (0..N)
        .map(|_| {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            s
        })
        .collect()
}

fn bench(c: &mut Criterion) {
    let indices = words();
    let coords: Vec<(u64, u64)> = indices
        .iter()
        .map(|&h| Hilbert2::<u64>::from_index(h).decode())
        .collect();
    // Truncation is the point: the coordinates fit in 32 bits.
    #[allow(clippy::cast_possible_truncation)]
    let coords32: Vec<(u32, u32)> = coords.iter().map(|&(x, y)| (x as u32, y as u32)).collect();

    // All three name the same curve: same index for every sample.
    for (&h, &(x, y)) in indices.iter().zip(&coords32) {
        assert_eq!(fast_hilbert::xy2h(x, y, 32), h, "fast_hilbert orientation");
        assert_eq!(lindel::hilbert_encode([x, y]), h, "lindel orientation");
    }

    {
        let mut g = c.benchmark_group("hilbert/decode");
        g.bench_function("hakmem", |b| {
            b.iter(|| {
                for &h in &indices {
                    black_box(Hilbert2::<u64>::from_index(black_box(h)).decode());
                }
            });
        });
        g.bench_function("fast_hilbert", |b| {
            b.iter(|| {
                for &h in &indices {
                    black_box(fast_hilbert::h2xy::<u32>(black_box(h), 32));
                }
            });
        });
        g.bench_function("lindel", |b| {
            b.iter(|| {
                for &h in &indices {
                    black_box(lindel::hilbert_decode::<u32, 2>(black_box(h)));
                }
            });
        });
        g.bench_function("reference d2xy", |b| {
            b.iter(|| {
                for &h in &indices {
                    black_box(reference::hilbert_coords(black_box(h), 32));
                }
            });
        });
        g.bench_function("morton decode", |b| {
            b.iter(|| {
                for &h in &indices {
                    black_box(Morton2::<u64>::from_code(black_box(h)).decode());
                }
            });
        });
        g.finish();
    }

    {
        let mut g = c.benchmark_group("hilbert/encode");
        g.bench_function("hakmem", |b| {
            b.iter(|| {
                for &(x, y) in &coords {
                    black_box(Hilbert2::<u64>::encode(black_box(x), black_box(y)));
                }
            });
        });
        g.bench_function("fast_hilbert", |b| {
            b.iter(|| {
                for &(x, y) in &coords32 {
                    black_box(fast_hilbert::xy2h(black_box(x), black_box(y), 32));
                }
            });
        });
        g.bench_function("lindel", |b| {
            b.iter(|| {
                for &(x, y) in &coords32 {
                    black_box(lindel::hilbert_encode([black_box(x), black_box(y)]));
                }
            });
        });
        g.bench_function("state machine loop", |b| {
            b.iter(|| {
                for &(x, y) in &coords {
                    black_box(reference::hilbert_index_machine(black_box(x), black_box(y)));
                }
            });
        });
        g.bench_function("reference xy2d", |b| {
            b.iter(|| {
                for &(x, y) in &coords {
                    black_box(reference::hilbert_index(black_box(x), black_box(y), 32));
                }
            });
        });
        g.bench_function("morton encode", |b| {
            b.iter(|| {
                for &(x, y) in &coords {
                    black_box(Morton2::<u64>::encode(black_box(x), black_box(y)));
                }
            });
        });
        g.finish();
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);

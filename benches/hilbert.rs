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

// criterion_group! expands to an undocumented pub fn; the tables module
// is private, so its `pub(super)` is what rustc's unreachable_pub wants.
#![allow(missing_docs, clippy::redundant_pub_crate)]

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

fn bench3(c: &mut Criterion) {
    let indices: Vec<u64> = words().iter().map(|&h| h >> 1).collect();
    let coords: Vec<(u64, u64, u64)> = indices
        .iter()
        .map(|&h| Hilbert3::<u64>::from_index(h).decode())
        .collect();
    for (&h, &(x, y, z)) in indices.iter().zip(&coords) {
        assert_eq!(tables::decode(h, 21), (x, y, z), "table orientation");
        assert_eq!(tables::encode(x, y, z, 21), h, "table orientation");
    }
    {
        let mut g = c.benchmark_group("hilbert3/decode");
        g.bench_function("hakmem", |b| {
            b.iter(|| {
                for &h in &indices {
                    black_box(Hilbert3::<u64>::from_index(black_box(h)).decode());
                }
            });
        });
        g.bench_function("rawrunprotected tables", |b| {
            b.iter(|| {
                for &h in &indices {
                    black_box(tables::decode(black_box(h), 21));
                }
            });
        });
        g.bench_function("morton decode", |b| {
            b.iter(|| {
                for &h in &indices {
                    black_box(Morton3::<u64>::from_code(black_box(h)).decode());
                }
            });
        });
        g.finish();
    }
    {
        let mut g = c.benchmark_group("hilbert3/encode");
        g.bench_function("hakmem", |b| {
            b.iter(|| {
                for &(x, y, z) in &coords {
                    black_box(Hilbert3::<u64>::encode(
                        black_box(x),
                        black_box(y),
                        black_box(z),
                    ));
                }
            });
        });
        g.bench_function("rawrunprotected tables", |b| {
            b.iter(|| {
                for &(x, y, z) in &coords {
                    black_box(tables::encode(black_box(x), black_box(y), black_box(z), 21));
                }
            });
        });
        g.bench_function("morton encode", |b| {
            b.iter(|| {
                for &(x, y, z) in &coords {
                    black_box(Morton3::<u64>::encode(
                        black_box(x),
                        black_box(y),
                        black_box(z),
                    ));
                }
            });
        });
        g.finish();
    }
}

/// rawrunprotected's 3D tables (public domain), the incumbent for the
/// 3D curve: a twelve-state machine, one dependent load per level.
mod tables {
    use hakmem::prelude::*;

    #[rustfmt::skip]
    const MORTON_TO_HILBERT: [u8; 96] = [
        48, 33, 27, 34, 47, 78, 28, 77, 66, 29, 51, 52, 65, 30, 72, 63,
        76, 95, 75, 24, 53, 54, 82, 81, 18,  3, 17, 80, 61,  4, 62, 15,
         0, 59, 71, 60, 49, 50, 86, 85, 84, 83,  5, 90, 79, 56,  6, 89,
        32, 23,  1, 94, 11, 12,  2, 93, 42, 41, 13, 14, 35, 88, 36, 31,
        92, 37, 87, 38, 91, 74,  8, 73, 46, 45,  9, 10,  7, 20, 64, 19,
        70, 25, 39, 16, 69, 26, 44, 43, 22, 55, 21, 68, 57, 40, 58, 67,
    ];
    #[rustfmt::skip]
    const HILBERT_TO_MORTON: [u8; 96] = [
        48, 33, 35, 26, 30, 79, 77, 44, 78, 68, 64, 50, 51, 25, 29, 63,
        27, 87, 86, 74, 72, 52, 53, 89, 83, 18, 16,  1,  5, 60, 62, 15,
         0, 52, 53, 57, 59, 87, 86, 66, 61, 95, 91, 81, 80,  2,  6, 76,
        32,  2,  6, 12, 13, 95, 91, 17, 93, 41, 40, 36, 38, 10, 11, 31,
        14, 79, 77, 92, 88, 33, 35, 82, 70, 10, 11, 23, 21, 41, 40,  4,
        19, 25, 29, 47, 46, 68, 64, 34, 45, 60, 62, 71, 67, 18, 16, 49,
    ];

    fn transform(input: u64, levels: u32, table: &[u8; 96]) -> u64 {
        let mut state = 0usize;
        let mut out = 0u64;
        for level in (0..levels).rev() {
            let entry = usize::from(table[state | ((input >> (3 * level)) & 7) as usize]);
            out = (out << 3) | (entry & 7) as u64;
            state = entry & !7;
        }
        out
    }

    pub(super) fn encode(x: u64, y: u64, z: u64, levels: u32) -> u64 {
        transform(
            Morton3::<u64>::encode(x, y, z).code(),
            levels,
            &MORTON_TO_HILBERT,
        )
    }

    pub(super) fn decode(h: u64, levels: u32) -> (u64, u64, u64) {
        Morton3::<u64>::from_code(transform(h, levels, &HILBERT_TO_MORTON)).decode()
    }
}

criterion_group!(benches, bench, bench3);
criterion_main!(benches);

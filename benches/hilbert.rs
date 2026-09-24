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

/// Batch encode against the per-key forms, from coordinates to keys.
fn bench_batch(c: &mut Criterion) {
    let coords: Vec<(u64, u64)> = words()
        .iter()
        .map(|&h| Hilbert2::<u64>::from_index(h).decode())
        .collect();
    // Truncation is the point: the coordinates fit in 32 bits.
    #[allow(clippy::cast_possible_truncation)]
    let coords32: Vec<(u32, u32)> = coords.iter().map(|&(x, y)| (x as u32, y as u32)).collect();
    // Batch encode, from coordinates to keys the way a caller does it:
    // Morton per point, then the in-place conversion. Against the
    // per-key forms on the same points.
    {
        let mut keys = vec![0u64; N];
        let mut g = c.benchmark_group("hilbert/encode batch");
        g.bench_function("hakmem in place", |b| {
            b.iter(|| {
                for (k, &(x, y)) in keys.iter_mut().zip(&coords) {
                    *k = Morton2::<u64>::encode(black_box(x), black_box(y)).code();
                }
                Hilbert2::<u64>::from_morton_in_place(&mut keys);
                black_box(&keys);
            });
        });
        let (cx, cy): (Vec<u64>, Vec<u64>) = coords.iter().copied().unzip();
        g.bench_function("hakmem columns", |b| {
            b.iter(|| {
                Hilbert2::<u64>::encode_columns(black_box(&cx), &cy, &mut keys);
                black_box(&keys);
            });
        });
        g.bench_function("hakmem per key", |b| {
            b.iter(|| {
                for (k, &(x, y)) in keys.iter_mut().zip(&coords) {
                    *k = Hilbert2::<u64>::encode(black_box(x), black_box(y)).index();
                }
                black_box(&keys);
            });
        });
        g.bench_function("fast_hilbert", |b| {
            b.iter(|| {
                for (k, &(x, y)) in keys.iter_mut().zip(&coords32) {
                    *k = fast_hilbert::xy2h(black_box(x), black_box(y), 32);
                }
                black_box(&keys);
            });
        });
        g.finish();
    }

    // The packed R-tree case (flatbush, geo-index): 16-bit coordinates,
    // 32-bit keys, 16 levels.
    {
        // Truncation is the point: 16-bit coordinates.
        #[allow(clippy::cast_possible_truncation)]
        let coords16: Vec<(u16, u16)> = coords32
            .iter()
            .map(|&(x, y)| (x as u16, y as u16))
            .collect();
        for &(x, y) in &coords16 {
            assert_eq!(
                fast_hilbert::xy2h(x, y, 16),
                Hilbert2::<u32>::encode(u32::from(x), u32::from(y)).index(),
                "fast_hilbert orientation at 16 levels"
            );
        }
        let mut keys = vec![0u32; N];
        let mut g = c.benchmark_group("hilbert/encode batch u16");
        g.bench_function("hakmem in place", |b| {
            b.iter(|| {
                for (k, &(x, y)) in keys.iter_mut().zip(&coords16) {
                    *k = Morton2::<u32>::encode(u32::from(black_box(x)), u32::from(black_box(y)))
                        .code();
                }
                Hilbert2::<u32>::from_morton_in_place(&mut keys);
                black_box(&keys);
            });
        });
        g.bench_function("hakmem per key", |b| {
            b.iter(|| {
                for (k, &(x, y)) in keys.iter_mut().zip(&coords16) {
                    *k = Hilbert2::<u32>::encode(u32::from(black_box(x)), u32::from(black_box(y)))
                        .index();
                }
                black_box(&keys);
            });
        });
        g.bench_function("fast_hilbert", |b| {
            b.iter(|| {
                for (k, &(x, y)) in keys.iter_mut().zip(&coords16) {
                    *k = fast_hilbert::xy2h(black_box(x), black_box(y), 16);
                }
                black_box(&keys);
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

/// 3D batch conversions against the per-key forms and the tables.
fn bench3_batch(c: &mut Criterion) {
    let indices: Vec<u64> = words().iter().map(|&h| h >> 1).collect();
    let coords: Vec<(u64, u64, u64)> = indices
        .iter()
        .map(|&h| Hilbert3::<u64>::from_index(h).decode())
        .collect();
    {
        let mut keys = vec![0u64; N];
        let mut out = vec![(0u64, 0u64, 0u64); N];
        let mut g = c.benchmark_group("hilbert3/decode batch");
        g.bench_function("hakmem in place", |b| {
            b.iter(|| {
                keys.copy_from_slice(&indices);
                Hilbert3::<u64>::into_morton_in_place(&mut keys);
                for (o, &k) in out.iter_mut().zip(&keys) {
                    *o = Morton3::<u64>::from_code(k).decode();
                }
                black_box(&out);
            });
        });
        g.bench_function("hakmem per key", |b| {
            b.iter(|| {
                for (o, &h) in out.iter_mut().zip(&indices) {
                    *o = Hilbert3::<u64>::from_index(black_box(h)).decode();
                }
                black_box(&out);
            });
        });
        g.bench_function("rawrunprotected tables", |b| {
            b.iter(|| {
                for (o, &h) in out.iter_mut().zip(&indices) {
                    *o = tables::decode(black_box(h), 21);
                }
                black_box(&out);
            });
        });
        let (mut xs, mut ys, mut zs) = (vec![0u64; N], vec![0u64; N], vec![0u64; N]);
        g.bench_function("hakmem columns", |b| {
            b.iter(|| {
                Hilbert3::<u64>::decode_columns(black_box(&indices), &mut xs, &mut ys, &mut zs);
                black_box((&xs, &ys, &zs));
            });
        });
        g.finish();
    }
    {
        let mut keys = vec![0u64; N];
        let mut g = c.benchmark_group("hilbert3/encode batch");
        g.bench_function("hakmem in place", |b| {
            b.iter(|| {
                for (k, &(x, y, z)) in keys.iter_mut().zip(&coords) {
                    *k = Morton3::<u64>::encode(black_box(x), black_box(y), black_box(z)).code();
                }
                Hilbert3::<u64>::from_morton_in_place(&mut keys);
                black_box(&keys);
            });
        });
        g.bench_function("hakmem per key", |b| {
            b.iter(|| {
                for (k, &(x, y, z)) in keys.iter_mut().zip(&coords) {
                    *k = Hilbert3::<u64>::encode(black_box(x), black_box(y), black_box(z)).index();
                }
                black_box(&keys);
            });
        });
        g.bench_function("rawrunprotected tables", |b| {
            b.iter(|| {
                for (k, &(x, y, z)) in keys.iter_mut().zip(&coords) {
                    *k = tables::encode(black_box(x), black_box(y), black_box(z), 21);
                }
                black_box(&keys);
            });
        });
        let xs: Vec<u64> = coords.iter().map(|c| c.0).collect();
        let ys: Vec<u64> = coords.iter().map(|c| c.1).collect();
        let zs: Vec<u64> = coords.iter().map(|c| c.2).collect();
        g.bench_function("hakmem columns", |b| {
            b.iter(|| {
                Hilbert3::<u64>::encode_columns(black_box(&xs), &ys, &zs, &mut keys);
                black_box(&keys);
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

/// 2D Morton over columns against the per-point form.
fn bench_morton_batch(c: &mut Criterion) {
    let w = words();
    let xs: Vec<u64> = w.iter().map(|&v| v & 0xFFFF_FFFF).collect();
    let ys: Vec<u64> = w.iter().map(|&v| v >> 32).collect();
    let mut codes = vec![0u64; N];
    {
        let mut g = c.benchmark_group("morton2/encode batch");
        g.bench_function("per point", |b| {
            b.iter(|| {
                for ((o, &x), &y) in codes.iter_mut().zip(&xs).zip(&ys) {
                    *o = Morton2::<u64>::encode(black_box(x), black_box(y)).code();
                }
                black_box(&codes);
            });
        });
        g.bench_function("columns", |b| {
            b.iter(|| {
                Morton2::<u64>::encode_columns(black_box(&xs), &ys, &mut codes);
                black_box(&codes);
            });
        });
        g.finish();
    }
    let (mut dx, mut dy) = (vec![0u64; N], vec![0u64; N]);
    {
        let mut g = c.benchmark_group("morton2/decode batch");
        g.bench_function("per point", |b| {
            b.iter(|| {
                for ((&m, x), y) in codes.iter().zip(&mut dx).zip(&mut dy) {
                    (*x, *y) = Morton2::<u64>::from_code(black_box(m)).decode();
                }
                black_box((&dx, &dy));
            });
        });
        g.bench_function("columns", |b| {
            b.iter(|| {
                Morton2::<u64>::decode_columns(black_box(&codes), &mut dx, &mut dy);
                black_box((&dx, &dy));
            });
        });
        g.finish();
    }
}

/// A granule and a rectangle, `intersects` in its argument order.
type Query = ((u64, u64), (u64, u64), (u64, u64));
/// A curve for the intersects bench: name, encode, the test.
type Curve = (
    &'static str,
    fn(u64, u64) -> u64,
    fn((u64, u64), (u64, u64), (u64, u64)) -> bool,
);

/// Rectangles to key ranges on the full `u64` grid: 256 random
/// rectangles of sides up to `2^k` cells, one call each.
fn bench_cover(c: &mut Criterion) {
    let w = words();
    for side_bits in [8u32, 16, 24] {
        let rects: Vec<((u64, u64), (u64, u64))> = w
            .chunks(4)
            .map(|q| {
                let mask = (1u64 << side_bits) - 1;
                let (x0, y0) = (q[0] & 0xFFFF_FFFF, q[1] & 0xFFFF_FFFF);
                let (x1, y1) = (
                    (x0 + (q[2] & mask)).min(0xFFFF_FFFF),
                    (y0 + (q[3] & mask)).min(0xFFFF_FFFF),
                );
                ((x0, x1), (y0, y1))
            })
            .collect();
        let mut g = c.benchmark_group(format!("cover/sides to 2^{side_bits}"));
        // Granules as a sparse index meets them: around a cell inside
        // (hits), around a cell just left of the rectangle and missing
        // it (the descent goes to the bottom), and 2^40 keys anywhere
        // (turned away at the root; the only kind this bench used to
        // time, which flattered it).
        let curves: [Curve; 2] = [
            (
                "Morton2",
                |x, y| Morton2::<u64>::encode(x, y).code(),
                Morton2::<u64>::intersects,
            ),
            (
                "Hilbert2",
                |x, y| Hilbert2::<u64>::encode(x, y).index(),
                Hilbert2::<u64>::intersects,
            ),
        ];
        for (name, key, test) in curves {
            let mut state = 0x2545_F491_4F6C_DD1Du64 ^ u64::from(side_bits);
            let mut next = move || {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                state
            };
            let mut sets: [Vec<Query>; 3] = Default::default();
            for &(x, y) in rects.iter().cycle() {
                if sets.iter().all(|s| s.len() >= rects.len()) {
                    break;
                }
                let width = 1u64 << (next() % 24);
                let around = |k: u64, a: u64, b: u64| {
                    (k.saturating_sub(a % width), k.saturating_add(b % width))
                };
                let (cx, cy, row) = (
                    x.0 + next() % (x.1 - x.0 + 1),
                    y.0 + next() % (y.1 - y.0 + 1),
                    y.0 + next() % (y.1 - y.0 + 1),
                );
                let hit = around(key(cx, cy), next(), next());
                let near = around(key(x.0.wrapping_sub(1), row), next(), next());
                let far = {
                    let a = next();
                    (a, a.saturating_add(1 << 40))
                };
                for (set, span, want) in [(0, hit, true), (1, near, false), (2, far, false)] {
                    if sets[set].len() < rects.len() && x.0 > 0 && test(span, x, y) == want {
                        sets[set].push((span, x, y));
                    }
                }
            }
            for (label, set) in ["hit", "near miss", "far miss"].iter().zip(&sets) {
                g.bench_function(format!("{name} intersects, {label}"), |b| {
                    b.iter(|| {
                        for &(s, x, y) in set {
                            black_box(test(black_box(s), x, y));
                        }
                    });
                });
            }
        }
        for budget in [16usize, 64] {
            let mut out = vec![(0u64, 0u64); budget];
            g.bench_function(format!("Morton2, {budget} ranges"), |b| {
                b.iter(|| {
                    for &(x, y) in &rects {
                        black_box(Morton2::<u64>::cover(black_box(x), y, &mut out));
                    }
                });
            });
            g.bench_function(format!("Hilbert2, {budget} ranges"), |b| {
                b.iter(|| {
                    for &(x, y) in &rects {
                        black_box(Hilbert2::<u64>::cover(black_box(x), y, &mut out));
                    }
                });
            });
        }
        g.finish();
    }
}

criterion_group!(
    benches,
    bench,
    bench_batch,
    bench3,
    bench3_batch,
    bench_morton_batch,
    bench_cover
);
criterion_main!(benches);

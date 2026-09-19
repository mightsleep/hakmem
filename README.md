# hakmem

[![x86_64](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml/badge.svg)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml)
[![aarch64](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml/badge.svg)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml)
[![crates.io](https://img.shields.io/crates/v/hakmem.svg)](https://crates.io/crates/hakmem)
[![docs.rs](https://img.shields.io/docsrs/hakmem)](https://docs.rs/hakmem)
[![msrv 1.87](https://img.shields.io/badge/msrv-1.87-blue)](https://github.com/mightsleep/hakmem/blob/main/Cargo.toml)
![no-std](https://img.shields.io/badge/no__std-yes-blue)
![license](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)


Typed combinator algebra over machine words. A register is an 8- to
128-element container; an instruction (POPCNT, TZCNT, the carry chain,
PEXT/PDEP, PCLMULQDQ) is a combinator; a kernel is a composition. The
crate names the combinators, restricts them to the domains where they
are lawful, and exports the laws they obey so downstream code can
property-test its own carriers and backends.

Named after [HAKMEM](https://en.wikipedia.org/wiki/HAKMEM) (MIT AI Memo
239, 1972), items 161–180 of which are the first catalogue of these
sentences.

```rust
use hakmem::{Bits, Morton2};

// Runs: bit p set iff bits p..p+3 are all set (Hacker's Delight 6-5).
let x: u64 = 0b0111_0110;
assert_eq!(x.run_starts(3).first_set(), Some(4));

// Rank / select / positions: succinct-structure primitives, O(1).
assert_eq!(x.rank_below(4), 2);
assert_eq!(x.select(2), Some(4));
assert_eq!(x.positions().collect::<Vec<_>>(), [1, 2, 4, 5, 6]);

// Scans: prefix XOR turns quote toggles into an inside-string mask;
// Kogge–Stone fills slide along a stride through a propagation mask.
assert_eq!(0b0100_0100u64.prefix_xor(), 0b0011_1100);
assert_eq!(1u8.fill_up(0b0000_0111, 1), 0b0000_0111);

// SWAR byte lanes: exact zero/equal/less-than tests, eight at a time.
assert_eq!(0x41_0A_42_09u32.bytes_lt(0x20), 0x00_80_00_80);
assert!(0x0012_0034u32.has_zero_byte());

// Compact / expand: PEXT / PDEP with a portable fallback.
assert_eq!(0b1001u32.compact(0b1010), 0b10);

// Dilated integers and Morton codes for 2D addressing.
let m = Morton2::<u32>::encode(3, 5);
assert_eq!(m.step_x().decode(), (4, 5));

// 8×8 bit matrices: three delta swaps per transpose.
use hakmem::permute::board8::transpose;
assert_eq!(transpose(1 << (8 * 1 + 5)), 1 << (8 * 5 + 1));

// Hacker's Delight ch. 2 and HAKMEM 175, named and lawful.
assert_eq!(0b1011_0110u32.clear_lowest_run(), 0b1011_0000);
assert_eq!(37u32.round_up_pow2(), Some(64));
assert_eq!(0b0011u8.next_same_popcount(), Some(0b0101)); // Gosper's hack
assert_eq!(0b0111_0110u32.longest_run(), 3);

// simdjson's find_escaped: quotes preceded by an odd run of backslashes.
let (escaped, _carry) = 0b01_1101_1010u16.find_escaped(false);
assert_eq!(escaped, 0b10_0000_0100);

// The carry chain as a dynamic-programming column (Myers 1999).
use hakmem::myers::{edit_distance, search};
assert_eq!(edit_distance::<u64>(b"kitten", b"sitting"), Some(3));
let hits: Vec<_> = search::<u64>(b"lo", b"hello lo", 0).unwrap().collect();
assert_eq!(hits, [(5, 0), (8, 0)]);

// Patterns wider than a register: `Wide<N>` is `[u64; N]` as one word,
// and the same Myers code runs on it unchanged.
use hakmem::Wide;
let long = [b'a'; 200];
assert_eq!(edit_distance::<Wide<4>>(&long, &long[..190]), Some(10));

// 2D runs: a w×h block in a bitmap of rows, by two halving chains
// (binary erosion by a rectangle; first-fit for tile allocators).
let rows = [0b0111_1000u8, 0b0111_1100, 0b0011_1100, 0];
let mut scratch = [0u8; 4];
assert_eq!(hakmem::grid::find_block(&rows, 2, 2, &mut scratch), Some((0, 3)));

// Slices of words, no index needed.
assert_eq!(hakmem::slice::find_run(&[0xFFu64 << 56, u64::MAX], 16), Some(56));
```

## Status

This is a learning project, one person's, and far from production.
I am working through the broadword literature by giving each trick a
type, a law and a test; the crate is what that leaves behind. Expect
the API to change between minor versions, and do not expect every
definition to be the best one available: the portable `select` loses
to a plain loop for small `k`, the portable `pext` is a loop over the
mask, and definitions were chosen for clarity and lawfulness before
speed.

What I do take seriously is finding bugs. Every combinator ships
with laws; the laws run on every carrier, with and without the
hardware paths, exhaustively at 8 and 16 bits, and under Miri for
the intrinsics; CI builds all of it as sandboxed Nix derivations on
`x86_64` and `aarch64`. A wrong result is a bug and I want to hear about
it. A slow one may be known; the design notes linked at the end list
what is.

If you need a stable dependency today, take the two or three lines
you need from Hacker's Delight; that is where this crate started. If
you want to see them named, typed and checked, read on.

## Benchmarks against the crates people use

Criterion, AMD Zen 5, one thread, `cargo bench -p hakmem --bench
incumbents`. Numbers are from a sandboxed run; rerun on your machine
before believing them. What the table does not hide: without BMI2 the
select is slower than the `broadword` crate's, and `Wide<N>` pays `N`
limb operations per step.

| select in a `u64` (1024 words) | portable | `+bmi2` |
|---|---|---|
| `hakmem` `Bits::select` | 5.24 µs | **1.21 µs** |
| `broadword::select1_raw` | **4.60 µs** | 4.54 µs |

| Levenshtein, pattern m × text n | `hakmem` u64 | `hakmem` u128 | `triple_accel` | `triple_accel_exp` | `strsim` |
|---|---|---|---|---|---|
| 16 × 64 | **153 ns** | 224 ns | 852 ns | 590 ns | 711 ns |
| 32 × 256 | **602 ns** | 829 ns | 19.6 µs | 6.10 µs | 5.74 µs |
| 64 × 256 | **614 ns** | 860 ns | 21.4 µs | 6.80 µs | 11.4 µs |
| 64 × 1024 | **2.37 µs** | 3.26 µs | 235 µs | 221 µs | 44.1 µs |
| 128 × 1024 | n/a | **3.24 µs** | 249 µs | 231 µs | 90.0 µs |

| Longer patterns, `Wide<N>` | `hakmem` `Wide<4>` | `hakmem` `Wide<8>` | `triple_accel` | `triple_accel_exp` | `strsim` |
|---|---|---|---|---|---|
| 128 × 1024 | **7.41 µs** | 19.3 µs | 245 µs | 231 µs | 109 µs |
| 256 × 1024 | **8.06 µs** | 20.4 µs | 274 µs | 257 µs | 176 µs |
| 512 × 1024 | n/a | **22.3 µs** | 326 µs | 307 µs | 352 µs |

Why the gap: Myers' bit-parallel column costs ~12 word operations per
text byte for the whole pattern (64 cells per word), the state lives in
registers, nothing is allocated and no UTF-8 is decoded; the others pay
per cell, and `triple_accel`'s full-distance API searches upward over
the band width, which is a poor fit when the lengths differ a lot.
Longer patterns use `Wide<N>` (`N × 64` bits as one word): each step
then costs `N` limb operations, still well below per-cell DP.

`triple_accel`'s own design point is two strings of similar length that
differ in a few edits, with a known bound `k` (`levenshtein_simd_k`).
There it catches up and, past a couple of words, wins:

| same length, few edits | `hakmem` | `triple_accel_simd_k` | `triple_accel_exp` | `strsim` |
|---|---|---|---|---|
| 64, 4 edits (u64) | **165 ns** | 980 ns | 1.03 µs | 2.74 µs |
| 128, 8 edits (u128) | **494 ns** | 1.86 µs | 1.94 µs | 11.7 µs |
| 512, 8 edits (`Wide<8>`) | 13.5 µs | **7.13 µs** | 7.39 µs | 177 µs |
| 512, 32 edits (`Wide<8>`) | 13.8 µs | 10.1 µs | **7.39 µs** | 178 µs |

`hakmem` always computes the full distance; a banded variant (Ukkonen
cut-off over the active blocks) is the obvious next step for long
similar strings and is not implemented.

## Cookbook

`hakmem::cookbook` explains how the shipped kernels were composed:
the simdjson inside-string mask, first-fit runs across words, the
Myers column step, Morton neighbours, `strlen` in one word, sliding
attacks, each as problem, decomposition, the laws that justify it,
and cost, with doctests. The last recipe is the one deliberately left
unbuilt (banded Myers over `Wide<N>`), sketched so you can.

## Hardware paths

Chosen at compile time, never at run time: build with
`-C target-feature=+bmi2,+pclmulqdq` (or `-C target-cpu=native`) and
`pext`/`pdep`/`select`/`prefix_xor` become single instructions; without
them every combinator has a portable definition with the same
contract. The `portable` cargo feature turns the hardware paths off even
when the target feature is present, for the microarchitectures where the
instruction exists but is microcoded (PDEP/PEXT on AMD Zen 1 and 2).

## Laws

Every combinator ships with the laws it obeys (`hakmem::laws`), checked
by property tests on all carriers with and without the hardware paths,
and exhaustively for widths up to 16 bits. A backend that fails a law
is a bug, never a documented caveat.

`no_std`, zero dependencies, stable Rust.

Design: [`docs/design.md`](https://github.com/mightsleep/hakmem/blob/main/docs/design.md),
the decisions behind the API, the hardware policy, how the laws are
verified, and where every combinator comes from.

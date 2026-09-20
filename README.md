<h1 align="center">hakmem</h1>
<p align="center">Bit tricks as a lawful algebra.</p>
<p align="center">
  <a href="https://mightsleep.github.io/hakmem/doc/hakmem/">docs</a> ·
  <a href="https://github.com/mightsleep/hakmem/blob/main/docs/design.md">design</a> ·
  <a href="https://mightsleep.github.io/hakmem/doc/hakmem/cookbook/index.html">cookbook</a> ·
  <a href="https://mightsleep.github.io/hakmem/dev/bench/">benchmarks</a>
</p>
<p align="center">
  <a href="https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain"><img alt="x86_64" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fall.json&style=for-the-badge"></a>
  <a href="https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain"><img alt="aarch64" src="https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fall.json&style=for-the-badge"></a>
  <a href="https://crates.io/crates/hakmem"><img alt="crates.io" src="https://img.shields.io/crates/v/hakmem?style=for-the-badge&labelColor=313244&color=a6e3a1&logo=rust&logoColor=cdd6f4"></a>
  <a href="https://docs.rs/hakmem"><img alt="docs.rs" src="https://img.shields.io/docsrs/hakmem?style=for-the-badge&labelColor=313244&color=cba6f7&logo=docsdotrs&logoColor=cdd6f4"></a>
  <img alt="msrv 1.89" src="https://img.shields.io/badge/msrv-1.89-fab387?style=for-the-badge&labelColor=313244&logo=rust&logoColor=cdd6f4">
  <img alt="no_std" src="https://img.shields.io/badge/no__std-yes-94e2d5?style=for-the-badge&labelColor=313244">
  <img alt="license" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-b4befe?style=for-the-badge&labelColor=313244">
</p>

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
use hakmem::{Bits, Hilbert2, Hilbert3, Morton2};

// Runs: bit p set iff bits p..p+3 are all set (Hacker's Delight 6-5).
let x: u64 = 0b0111_0110;
assert_eq!(x.run_starts(3).first_set(), Some(4));

// Rank / select / positions: succinct-structure primitives, O(1).
assert_eq!(x.rank(4), 2);
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

// Hilbert indices: the decode is two suffix XORs over the Morton code.
assert_eq!(Hilbert2::<u8>::encode_order(3, 1, 2).index(), 12);
assert_eq!(Hilbert2::from_index(13u8).decode_order(2), (2, 1));
// In 3D the frames form A₄ = AGL(1, 4): the decode is a scan over GF(4).
assert_eq!(Hilbert3::<u64>::encode(1000, 2000, 3000).decode(), (1000, 2000, 3000));

// 8×8 bit matrices: three delta swaps per transpose.
use hakmem::permute::board8::transpose;
assert_eq!(transpose(1 << (8 * 1 + 5)), 1 << (8 * 5 + 1));
use hakmem::permute::board8::{Dir, slide};
let rook = 1u64; // a1, attacking north up to the blocker on a5
assert_eq!(slide(rook, !(1u64 << 32), Dir::North), 0x0000_0001_0101_0100);

// Hacker's Delight ch. 2 and HAKMEM 175, named and lawful.
assert_eq!(0b1011_0110u32.clear_lowest_run(), 0b1011_0000);
assert_eq!(37u32.round_up_pow2(), Some(64));
assert_eq!(0b0011u8.next_same_popcount(), Some(0b0101)); // Gosper's hack
assert_eq!(0b0111_0110u32.longest_run(), 3);

// simdjson's find_escaped: quotes preceded by an odd run of backslashes.
let (escaped, _carry) = 0b01_1101_1010u16.find_escaped(false);
assert_eq!(escaped, 0b10_0000_0100);

// The carry chain as a dynamic-programming column (Myers 1999).
use hakmem::myers::{distance, edit_distance, search};
assert_eq!(edit_distance::<u64>(b"kitten", b"sitting"), Some(3));
assert_eq!(distance(b"kitten", b"sitting"), Some(3)); // carrier by length
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

// Byte lanes, the SIMD half: sixteen bytes to a mask, then back to `Bits`.
use hakmem::lanes::{Lanes, U8x16};
let block = U8x16::load(b"{\"a\": [1, 2]}   ");
let quotes = block.cmp_eq(U8x16::splat(b'"')).to_bits();
assert_eq!(quotes, 0b1010);
assert_eq!(quotes.prefix_xor(), 0b0110); // inside the string

// A rank/select directory over any `&[u64]`, storage you provide.
use hakmem::rank9::Rank9;
let bits = [0b1011u64, u64::MAX, 0];
let mut counts = vec![0; Rank9::counts_len(bits.len())];
let mut select = vec![0; Rank9::select_len(bits.len())];
Rank9::build(&bits, &mut counts, &mut select);
let dir = Rank9::new(&bits, &counts, &select);
assert_eq!((dir.rank(64), dir.select(3)), (3, Some(64)));
```

## Status

This is a learning project, one person's, and far from production.
I am working through the broadword literature by giving each trick a
type, a law and a test; the crate is what that leaves behind. Expect
the API to change between minor versions, and do not expect every
definition to be the best one available: the portable `select` loses
to a plain loop for small `k`, the portable `compact` to one over sparse
masks, and definitions were chosen for clarity and lawfulness before
speed.

What I do take seriously is finding bugs. Every combinator ships
with laws; the laws run on every carrier, with and without the
hardware paths, exhaustively at 8 and 16 bits, and under Miri for
the intrinsics (GFNI under Miri only, the runners being a mix of CPUs);
CI builds all of it as sandboxed Nix derivations on
`x86_64` and `aarch64`. A wrong result is a bug and I want to hear about
it. A slow one may be known; the design notes linked at the end list
what is.

If you need a stable dependency today, take the two or three lines
you need from Hacker's Delight; that is where this crate started. If
you want to see them named, typed and checked, read on.

## Checks

Every cell is one Nix derivation built in a sandbox without network,
on every push to `main`. The badges are written by CI after each run
(`.github/status.sh`) and read from the [site](https://mightsleep.github.io/hakmem/).

| check | `x86_64` | `aarch64` |
|---|---|---|
| tests, portable | [![hakmem-test-portable-default](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-test-portable-default.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-test-portable-default](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-test-portable-default.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| tests, `+bmi2,+pclmulqdq` | [![hakmem-test-bmi2-default](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-test-bmi2-default.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| tests, `+bmi2` with feature `portable` | [![hakmem-test-bmi2-portable-feature](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-test-bmi2-portable-feature.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| doctests (this README), portable | [![hakmem-doctest-portable](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-doctest-portable.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-doctest-portable](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-doctest-portable.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| doctests, `+bmi2` | [![hakmem-doctest-bmi2](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-doctest-bmi2.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| clippy, portable | [![hakmem-clippy-portable-default](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-clippy-portable-default.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-clippy-portable-default](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-clippy-portable-default.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| clippy, `+bmi2` | [![hakmem-clippy-bmi2-default](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-clippy-bmi2-default.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| clippy, `+bmi2` with feature `portable` | [![hakmem-clippy-bmi2-portable-feature](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-clippy-bmi2-portable-feature.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| rustdoc, warnings as errors | [![hakmem-doc](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-doc.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-doc](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-doc.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| MSRV 1.89 build of the packaged tarball | [![hakmem-msrv](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-msrv.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-msrv](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-msrv.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| cargo-deny (licences, bans, sources) | [![hakmem-deny](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-deny.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-deny](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-deny.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| cargo-audit (advisories, offline) | [![hakmem-audit](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-audit.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-audit](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-audit.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| treefmt | [![treefmt](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Ftreefmt.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![treefmt](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Ftreefmt.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| Miri over the intrinsics, both paths | [![miri](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fmiri.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |

## Benchmarks against the crates people use

Criterion, AMD Zen 5, one thread, `cargo bench --bench
incumbents`. Numbers are from a sandboxed run; rerun on your machine
before believing them. What the table does not hide: without BMI2 the
select is slower than the `broadword` crate's, and `Wide<N>` pays `N`
limb operations per step.

The same benches run in CI on a shared GitHub runner after every change
to `src/`, `benches/` or the Cargo files, and the
[benchmarks page](https://mightsleep.github.io/hakmem/dev/bench/) keeps
every run. Absolute times there move with the runner; the ratio to the
best incumbent inside one run does not, and that ratio is what the page
tracks.

| select in a `u64` (1024 words) | portable | `+bmi2` |
|---|---|---|
| `hakmem` `Bits::select` | 5.24 µs | **1.21 µs** |
| `broadword::select1_raw` | **4.60 µs** | 4.54 µs |

| `compact` in a `u64` (1024 words) | portable | `+bmi2,+pclmulqdq` |
|---|---|---|
| `hakmem` `Bits::compact` | 11.2 µs | **1.30 µs** |
| broadword definition, dense mask (about 32 set bits) | 11.1 µs | 8.00 µs |
| loop over the set bits, dense mask | 21.3 µs | 19.5 µs |
| loop over the set bits, sparse mask (about 8 set bits) | **4.36 µs** | 3.75 µs |

| rank / select over 2^20 bits, 1024 queries | `rank` dense | `rank` sparse | `select` dense | `select` sparse |
|---|---|---|---|---|
| `hakmem` `Rank9` | **1.33 µs** | **1.33 µs** | **5.26 µs** | 1.7 µs |
| `sux` rank9 / select9 | 2.02 µs | 2.03 µs | 5.52 µs | **1.44 µs** |
| `sucds` `Rank9Sel` | 1.90 µs | 1.89 µs | 8.83 µs | 11.3 µs |
| `vers-vecs` `RsVec` | 2.83 µs | 2.82 µs | 7.63 µs | 10.5 µs |

Dense is half the bits set, sparse one in 64. Rank is the same rank9
directory everywhere; the differences are bounds checks and layout.
Select keeps an inventory entry per 512 set bits and, by how many
blocks those 512 span, stores either packed 16-bit block counts (one or
two SWAR compares) or the positions themselves (no search at all), the
shape of Vigna's select9 with the cases cut at block boundaries. The
remaining gap to sux on the sparse slice is within run-to-run noise of
the bounds checks this crate keeps: it has no `unsafe` outside the
intrinsics, sux indexes unchecked. The price of the inventory is three
words per eight words of bits on top of rank9's two.

The portable `compact` is constant time: a loop over the set bits wins
below about 17 of them (12 when a PCLMULQDQ scan is available) and loses
above, the same trade as the portable `select`.

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
differ in a few edits, with a known bound `k` (`levenshtein_simd_k`;
below, `k` is twice the edits). There it catches up and, past a couple
of words, wins:

| same length, few edits | `hakmem` | `triple_accel_simd_k` | `triple_accel_exp` | `strsim` |
|---|---|---|---|---|
| 64, 4 edits (u64) | **165 ns** | 980 ns | 1.03 µs | 2.74 µs |
| 128, 8 edits (u128) | **494 ns** | 1.86 µs | 1.94 µs | 11.7 µs |
| 512, 8 edits (`Wide<8>`) | 13.5 µs | **7.13 µs** | 7.39 µs | 177 µs |
| 512, 32 edits (`Wide<8>`) | 13.8 µs | 10.1 µs | **7.39 µs** | 178 µs |

`hakmem` always computes the full distance; a banded variant (Ukkonen
cut-off over the active blocks) is the obvious next step for long
similar strings and is not implemented.

| Hilbert curve, 32 levels, `u32` coordinates (1024 points) | decode, portable | decode, `+pclmulqdq` | encode, portable | encode, `+bmi2,+pclmulqdq` |
|---|---|---|---|---|
| `hakmem` `Hilbert2` | **3.0 µs** | **1.7 µs** | **10.9 µs** | **5.4 µs** |
| `fast_hilbert` (512-byte transition table) | 12.8 µs | 12.7 µs | 11.5 µs | 11.4 µs |
| `lindel` (Skilling) | 66 µs | 71 µs | 70 µs | 76 µs |
| `Morton2`, for the price of the frames | 1.6 µs | 0.56 µs | 1.6 µs | 0.46 µs |

| Hilbert curve in 3D, 21 levels (1024 points) | decode, portable | decode, `+bmi2` | encode, portable | encode, `+bmi2` |
|---|---|---|---|---|
| `hakmem` `Hilbert3` | **10.5 µs** | **8.8 µs** | 19.6 µs | 19.6 µs |
| rawrunprotected's tables, 96 bytes each way | 16.9 µs | 14.9 µs | **18.7 µs** | **15.0 µs** |
| `Morton3` | 2.1 µs | 0.87 µs | 2.0 µs | 0.72 µs |

In 3D the frames form the alternating group `A₄`, which is
`AGL(1, 4)`, so the 2D encode's scan is the 3D decode; the 3D encode
has no such structure and is the twelve-state machine memoised into
the crate's one table, 96 bytes built at compile time from the same
arithmetic, which is why it ties the incumbent instead of beating it.
The 2D decode is two suffix XORs over the Morton code and a Morton
decode, straight-line. The encode is a Kogge–Stone scan over the
per-level frame maps, affine maps over GF(4) once levels are paired,
four rounds of about 24 word operations after the pairing: the
adder's carry chain with GF(4) in place of GF(2), and no instruction
for it. The construction is rawrunprotected's (threadlocalmutex.com,
2016); this is it on any carrier, with laws. `fast_hilbert` walks a state table three levels a
step, eleven dependent loads for 32 levels; the scan has no loads and
no carried chain, which is why it gains from `+bmi2` (the BMI
instructions) where the table walk cannot. Recipe 13 of the cookbook
has the derivation and the group that makes the encode the heavier
direction.

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
`pext`/`pdep`/`select`/`prefix_xor`/`suffix_xor` become single
instructions; without
them every combinator has a portable definition with the same
contract. `pext` and `pdep` fall back to Hacker's Delight's
parallel-suffix compress and expand (7-4, 7-5), `log₂ w` rounds of one
prefix-XOR scan each, constant time. The `portable` cargo feature turns
the hardware paths off even when the target feature is present, for the
microarchitectures where the instruction exists but is microcoded
(PDEP/PEXT on AMD Zen 1 and 2).
For the lanes, `+ssse3` (NEON on aarch64) makes `U8x16` a vector
register, and `+gfni` makes every byte map, `affine` and the shifts,
rotates and bit reversal built on it, one `gf2p8affineqb`; without it
a byte map is two nibble lookups.

## Laws

Every combinator ships with the laws it obeys (`hakmem::laws`), checked
by property tests on all carriers with and without the hardware paths,
and exhaustively for widths up to 16 bits. A backend that fails a law
is a bug, never a documented caveat.

Every combinator is total over its carrier where a total definition
exists: `rank(i)` past the width counts every bit, `run_starts(0)`
starts everywhere and `run_starts(k)` above the width nowhere, a fill
with a stride of zero or past the width is the identity, `low_ones` at
the width is all ones. Where an argument has a domain the definition
cannot absorb (shift amounts, run lengths above the width in
`slice::find_run`, overlapping `delta_swap` masks, `bytes_ge` above
128, grid sizes), the domain is in the method's docs, checked with
`debug_assert!` in debug builds, and unspecified in release. Run your
tests in debug once.

`no_std`, zero dependencies, stable Rust.

Design: [`docs/design.md`](https://github.com/mightsleep/hakmem/blob/main/docs/design.md),
the decisions behind the API, the hardware policy, how the laws are
verified, and where every combinator comes from.
Changes: [`CHANGELOG.md`](https://github.com/mightsleep/hakmem/blob/main/CHANGELOG.md).

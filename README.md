<h1 align="center">hakmem</h1>
<p align="center">Space-filling curves, rank/select and edit distance, built from bit tricks with laws.</p>
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

Space-filling curves, rank/select, bit-parallel edit distance, and the
bit tricks they are made of. `no_std`, no dependencies, stable Rust, no
`unsafe` outside the intrinsics.

| job | `hakmem` | what people use |
|---|---|---|
| 2D Hilbert keys from coordinates, 1024 points | **1.3 µs** | `fast_hilbert` 12.2 µs |
| 2D Morton keys from coordinates, 1024 points | **0.17 µs** | `zorder` 1.9 µs, 0.89 with its BMI2 path |
| a rectangle as 16 sorted ranges of keys | 0.3 µs Morton, 1.1 µs Hilbert | |
| can a block of keys hold a point of the rectangle | 10 to 100 ns | |
| rank over 2^20 bits, 1024 queries | **1.3 µs** | `sux` 2.0 µs |
| Levenshtein, 64-byte pattern over 1 KiB of text | **2.4 µs** | `strsim` 44 µs |

One thread of a Zen 5; the column rows pick AVX2 or AVX-512 at run
time and need no build flags. Where this crate loses, and it does, the
[tables below](#benchmarks-against-the-crates-people-use) say so.

Every function comes with the laws it obeys, exported as property
functions and checked on every width from `u8` to `u128` and
`[u64; N]`, with and without the hardware paths, exhaustively at 8 and
16 bits. CI also checks what the compiler makes of the loops that
matter, down to which function each instruction was inlined from
([Checks](#checks)). Named after
[HAKMEM](https://en.wikipedia.org/wiki/HAKMEM) (MIT AI Memo 239,
1972), items 161–180 of which are the first catalogue of these tricks.
They still work; now a compiler checks.

Every line of Rust in this README is a doctest, so if it lies, CI goes
red.

### Spatial keys

Morton and Hilbert codes in 2D and 3D, a point at a time or a column
at a time (`encode_columns`, `decode_columns`, `from_morton_in_place`),
and the query side of a sorted key column: a rectangle as a few ranges
to seek (`cover`), and whether a block of keys, a row group or a
granule by its least and greatest key, can hold anything in it
(`intersects`).

```rust
use hakmem::prelude::*;

let points = [(10u64, 10u64), (11, 12), (500, 500), (12, 11)];
let mut keys: Vec<u64> = points
    .iter()
    .map(|&(x, y)| Hilbert2::encode(x, y).index())
    .collect();
keys.sort_unstable();
// The block 8..=15 × 8..=15 in at most four ranges, a seek each.
let mut ranges = [(0u64, 0u64); 4];
let hits: usize = Hilbert2::cover(8..=15, 8..=15, &mut ranges)
    .iter()
    .map(|&(a, b)| keys.partition_point(|&k| k <= b) - keys.partition_point(|&k| k < a))
    .sum();
assert_eq!(hits, 3);

// Columns in, keys out; the kernel is chosen at run time.
let (xs, ys) = ([3u64, 40_000, 1 << 31], [5u64, 7, 12]);
let mut codes = [0u64; 3];
Morton2::<u64>::encode_columns(&xs, &ys, &mut codes);
assert_eq!(Morton2::from_code(codes[1]).decode(), (40_000, 7));
```

### Succinct bit vectors

Rank and select over any `&[u64]`, in storage you provide: the rank9
directory, and a select inventory that stores positions outright where
the set bits are sparse.

```rust
use hakmem::rank9::Rank9;

let bits = [0b1011u64, u64::MAX, 0];
let mut counts = vec![0; Rank9::counts_len(bits.len())];
let mut select = vec![0; Rank9::select_len(bits.len())];
let dir = Rank9::build(&bits, &mut counts, &mut select);
assert_eq!((dir.rank(64), dir.select(3)), (3, Some(64)));
```

### Fuzzy text

Levenshtein distance by Myers' bit-parallel column, 64 cells a word,
patterns longer than a register on `Wide<N>`; and search that reports
where each match starts, which the column alone does not know.

```rust
use hakmem::myers::{distance, distance_in, search};

assert_eq!(distance_in::<u64>(b"kitten", b"sitting"), Some(3));
assert_eq!(distance(b"kitten", b"sitting"), Some(3)); // carrier by length
// The typo is deliberate.
let text = b"the quick brown fox and the quikc brown fox";
let found: Vec<_> = search::<u64>(b"quick brown fox", text, 2)
    .unwrap()
    .occurrences()
    .map(|o| (o.start(), o.distance()))
    .collect();
assert_eq!(found, [(4, 0), (28, 2)]);
```

### Bits and bytes

PEXT and PDEP with a portable definition behind them, select in a word,
runs, scans, and sixteen byte lanes (SSSE3, NEON, or SWAR) whose masks
hand over to the word tricks: simdjson's first stage, as parts.

```rust
use hakmem::lanes::{Lanes, U8x16};
use hakmem::{Bits, Words};

assert_eq!(0b1001u32.compact(0b1010), 0b10); // PEXT
assert_eq!(0b0111_0110u64.select(2), Some(4));
assert_eq!([0xFFu64 << 56, u64::MAX].find_run(16), Some(56));

let block = U8x16::load(b"{\"a\": [1, 2]}   ");
let quotes = block.cmp_eq(U8x16::splat(b'"')).to_bitmask();
assert_eq!(quotes.prefix_xor(), 0b0110); // inside the string
```

## Everything else, one screen

Every fast codebase has a `bits.rs`: one-liners from Hacker's Delight,
each under a comment that says `// do not touch`. This crate started as
that file with the comment replaced by a proof obligation: every trick
has a name, a type, a domain where it is defined, and the laws it
obeys. The jobs above are what those tricks add up to; here is the rest.

```rust
use hakmem::{Bits, Hilbert2, Hilbert3, Morton2};

// Runs: bit p set iff bits p..p+3 are all set (Hacker's Delight 6-5).
let x: u64 = 0b0111_0110;
assert_eq!(x.run_starts(3).first_set(), Some(4));
assert_eq!(x.rank(4), 2);
assert_eq!(x.positions().collect::<Vec<_>>(), [1, 2, 4, 5, 6]);

// Kogge–Stone fills slide along a stride through a propagation mask.
assert_eq!(1u8.fill_up(0b0000_0111, 1), 0b0000_0111);

// SWAR byte lanes: exact zero/equal/less-than tests, eight at a time.
assert_eq!(0x41_0A_42_09u32.bytes_lt(0x20), 0x00_80_00_80);
assert!(0x0012_0034u32.has_zero_byte());

// Morton neighbours without decoding.
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

// Patterns wider than a register: `Wide<N>` is `[u64; N]` as one word,
// and the same Myers code runs on it unchanged.
use hakmem::Wide;
use hakmem::myers::distance_in;
let long = [b'a'; 200];
assert_eq!(distance_in::<Wide<4>>(&long, &long[..190]), Some(10));

// 2D runs: a w×h block in a bitmap of rows, by two halving chains
// (binary erosion by a rectangle; first-fit for tile allocators).
let rows = [0b0111_1000u8, 0b0111_1100, 0b0011_1100, 0];
let mut scratch = [0u8; 4];
assert_eq!(hakmem::grid::find_block(&rows, 2, 2, &mut scratch), Some((0, 3)));
```

## Status

Young, and one person's: I am working through the broadword
literature by giving each trick a type, a law and a test, and the crate
is what that leaves behind. What is here is checked harder than most
crates check anything; what is not here yet, and the API's habit of
moving, are the reasons to pin a version. Definitions were chosen for
lawfulness first and speed second, and it shows in places: the
portable `select` loses to a plain loop for small `k`, the portable
`compact` to one over sparse masks.

The API will change between minor versions (0.2 broke 0.1 without
apology). It changes on purpose, though: `public-api/` is checked in,
one file per target, and diffed in CI, and cargo-semver-checks says whether the version
number admits it.

What I do take seriously is finding bugs. Every combinator ships
with laws; the laws run on every carrier, with and without the
hardware paths, exhaustively at 8 and 16 bits, and under Miri for
the intrinsics (GFNI under Miri only, the runners being a mix of CPUs);
CI builds all of it as sandboxed Nix derivations on
`x86_64` and `aarch64`. A wrong result is a bug and I want to hear about
it. A slow one may be known; the design notes linked at the end list
what is.

If you need a stable dependency today, take the two or three lines
you need from Hacker's Delight; that is where this crate started, and
their MSRV is 1972. If you want them named, typed and checked, read on.

## Examples

`examples/` holds four programs written the way a user would write them,
each checking itself against the slow way: `geo_index` (a million
points in row groups, a rectangle query that skips most of them),
`primes` (the primes below ten million as a bitmap with rank and
select), `json_structure` (the first stage of simdjson, carries across
64-byte blocks) and `fuzzy` (the nearest words to a typo, and a phrase
found with its typos). `cargo run --release --example geo_index`.

## Benchmarks against the crates people use

Criterion, AMD Zen 5, one thread, `cargo bench --bench
incumbents`. Numbers are from a sandboxed run; rerun on your machine
before believing them. Bold marks the winner, also when it is not
this crate. What the tables do not hide: without BMI2 the
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

| Morton code in 2D, `u32` coordinates (1024 points) | encode, portable | encode, `+bmi2` | decode, portable | decode, `+bmi2` |
|---|---|---|---|---|
| `hakmem` `Morton2`, columns | **0.17 µs** | **0.17 µs** | **0.22 µs** | **0.22 µs** |
| `hakmem` `Morton2`, per point | 1.81 µs | 0.47 µs | 1.39 µs | 0.44 µs |
| `zorder`, with its BMI2 path where the CPU has it | 0.89 µs | 0.47 µs | 0.95 µs | 0.44 µs |
| `zorder`, portable | 1.88 µs | 1.87 µs | 1.36 µs | 1.36 µs |
| `morton-encoding` (also `lindel`'s Morton) | 37 µs | 36 µs | 36 µs | 36 µs |

| Morton code in 2D, `u16` coordinates (1024 points) | encode, portable | encode, `+bmi2` | decode, portable | decode, `+bmi2` |
|---|---|---|---|---|
| `hakmem` `Morton2<u32>`, columns | **0.09 µs** | **0.09 µs** | **0.11 µs** | **0.11 µs** |
| `hakmem` `Morton2<u32>`, per point | 1.52 µs | 0.58 µs | 1.15 µs | 0.39 µs |
| `morton` | 1.52 µs | 1.52 µs | 1.15 µs | 1.16 µs |

The columns win because they choose AVX2 at run time and spread a
coordinate's nibbles with one PSHUFB. A point at a time, a Morton code
is a shift ladder or one PDEP each way, and it ties with the crates
that do the same: the decode is `Word::unzip`, two PEXT of the code
with BMI2, and without it, on a `u32` key, one ladder over a `u64`
holding both halves, which is `morton`'s trick. Every per-point row
writes its coordinates at the coordinates' width; the columns write
them as wide as the key, since that is their API, and lead anyway. The
one column this crate does
not take is the per-point form without build flags, where `zorder`
checks for BMI2 on every call and this crate uses what the build
proves: ask once instead, with the columns or `dispatch!`
([Hardware paths](#hardware-paths)).

| Hilbert curve, 32 levels, `u32` coordinates (1024 points) | decode, portable | decode, `+pclmulqdq` | encode, portable | encode, `+bmi2,+pclmulqdq` |
|---|---|---|---|---|
| `hakmem` `Hilbert2` | **2.6 µs** | **1.7 µs** | **10.9 µs** | **5.4 µs** |
| `fast_hilbert` (512-byte transition table) | 12.8 µs | 12.7 µs | 11.5 µs | 11.4 µs |
| `lindel` (Skilling) | 66 µs | 71 µs | 70 µs | 76 µs |
| `Morton2`, for the price of the frames | 1.6 µs | 0.56 µs | 1.6 µs | 0.46 µs |

| Hilbert curve in 3D, 21 levels (1024 points) | decode, portable | decode, `+bmi2` | encode, portable | encode, `+bmi2` |
|---|---|---|---|---|
| `hakmem` `Hilbert3` | **10.6 µs** | **8.8 µs** | **18.3 µs** | **13.2 µs** |
| rawrunprotected's tables, 96 bytes each way | 17.5 µs | 14.9 µs | 19.0 µs | 15.0 µs |
| `Morton3` | 2.1 µs | 0.78 µs | 2.0 µs | 0.72 µs |

In 3D the frames form the alternating group `A₄`, which is
`AGL(1, 4)`, so the 2D encode's scan is the 3D decode; the 3D encode
is not a scan and is the twelve-state machine memoised into
a table of 96 bytes built at compile time from the same
arithmetic. It is the incumbent's machine, ahead of it by a table padded
to 128 entries (the masked index needs no bounds check) and a loop the
compiler vectorises over points; in batches a `vpermi2b` walks it for
64 keys at a time.
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

Batches of keys convert in place: fill a slice with Morton codes from
whatever layout the points are in, then turn them into Hilbert indices
in one call (`from_morton_in_place`, and `to_morton_in_place` back;
the `_order` forms for a curve of fewer levels). With AVX-512 VBMI a
step is one `vpermi2b` per register of keys (2D, three levels through
128 entries, the frame modulo a reflection; 3D, the 96-byte encode or
decode table); with AVX2 alone the 3D machine modulo its translations,
two PSHUFB a level; NEON uses `tbl`; elsewhere it is the per-key
conversion. Coordinates never enter these kernels. For the 3D curve on
`u64` they may, as three columns (`Hilbert3::<u64>::encode_columns`,
`decode_columns`): with VBMI and GFNI that skips the Morton code both
ways, 1.2 and 1.0 ns a point against 1.7 and 1.8, and elsewhere it is
the Morton code and the batch above.

```rust
use hakmem::prelude::*;

let points = [(3u64, 5u64), (40_000, 7), (1 << 31, 12)];
let mut keys: Vec<u64> = points
    .iter()
    .map(|&(x, y)| Morton2::encode(x, y).code())
    .collect();
Hilbert2::<u64>::from_morton_in_place(&mut keys);
assert_eq!(keys[1], Hilbert2::<u64>::encode(40_000, 7).index());
keys.sort_unstable(); // curve order, as a packed R-tree builds it
```

| Batch conversion, 1024 points, `target-cpu=native` on Zen 5 | in place | per key | incumbent |
|---|---|---|---|
| 2D encode, `u64` keys, 32 levels | **1.6 µs** | 6.1 µs | `fast_hilbert` 12.2 µs |
| 2D encode, `u16` coordinates, `u32` keys, 16 levels | **0.90 µs** | 6.3 µs | `fast_hilbert` 8.5 µs |
| 2D encode from two columns, `u64` keys (`Hilbert2::encode_columns`) | **1.3 µs** | 6.1 µs | `fast_hilbert` 12.2 µs |
| 3D encode, `u64` keys, 21 levels | **1.8 µs** | 13.1 µs | rawrunprotected's tables 15.2 µs |
| 3D decode, `u64` keys, 21 levels | **1.8 µs** | 8.8 µs | rawrunprotected's tables 15.1 µs |

The query side: `Hilbert2::cover` and `Morton2::cover` turn a rectangle
into at most as many ranges of keys as the output slice holds, sorted
and inclusive, every cell of the rectangle in one. With room for the
exact cover they hold nothing else; with less they hold the least
extra a budget of ranges of the deepest cover that fits can hold. A
scan of the sorted keys seeks once per range. No allocation, and the
depth is counted rather than searched for: 0.3 µs a rectangle for 16
Morton ranges on the full `u64` grid, 1.1 µs for 16 Hilbert ranges.
The other way round, `intersects(keys, x, y)`, all three ranges, says whether a block of
keys (a granule, a row group, a file, by its least and greatest key)
can hold a point of the rectangle: one descent where a node is an
interval of keys and a square of cells at once. 40 to 90 ns when the
answer is yes, 100 when a granule just misses, 10 when it is nowhere
near.

## Cookbook

`hakmem::cookbook` explains how the shipped kernels were composed:
the simdjson inside-string mask, first-fit runs across words, the
Myers column step, Morton neighbours, `strlen` in one word, sliding
attacks, each as problem, decomposition, the laws that justify it,
and cost, with doctests. The last recipe is the one deliberately left
unbuilt (banded Myers over `Wide<N>`), sketched so you can.

## Hardware paths

The batch conversions (`from_morton_in_place`, `to_morton_in_place`,
`encode_columns`, `decode_columns`, the last two on `Morton2` and
`Hilbert2` too) choose their kernel at run time on
`x86_64`: AVX-512 VBMI and GFNI (the `X86V4` level), else AVX2 (`X86V3`), else the per-key form,
with no build flags and still `no_std`; the `portable` feature turns
that off. The methods on words and lanes use what the build proves:
build with
`-C target-feature=+bmi2,+pclmulqdq` (or `-C target-cpu=native`) and
`pext`/`pdep`/`select`/`prefix_xor`/`suffix_xor` become single
instructions; without
them every combinator has a portable definition with the same
contract. A hot loop that should not depend on the build asks once
instead (`hakmem::isa`, design notes section 3):

```rust
use hakmem::isa::Isa;

fn gather<I: Isa>(cpu: I, xs: &[u64], mask: u64) -> u64 {
    xs.iter().fold(0, |a, &x| a ^ cpu.pext(x, mask))
}
// One CPUID, then the loop compiled for the level it found: PEXT here
// even in a build without flags.
let n = hakmem::dispatch!(|cpu| gather(cpu, &[0b1011, 0b0110], 0b0110));
assert_eq!(n, 0b01 ^ 0b11);
```

Anything the loop calls has to inline into it, or it is compiled
without the level's features and each primitive in it is a call. `pext` and `pdep` fall back to Hacker's Delight's
parallel-suffix compress and expand (7-4, 7-5), `log₂ w` rounds of one
prefix-XOR scan each, constant time. The `portable` cargo feature turns
the hardware paths off even when the target feature is present, for the
microarchitectures where the instruction exists but is microcoded
(PDEP/PEXT on AMD Zen 1 and 2).
For the lanes, `+ssse3` (NEON on aarch64) makes `U8x16` a vector
register, and `+gfni` makes every byte map, `affine` and the shifts,
rotates and bit reversal built on it, one `gf2p8affineqb`; without it
a byte map is two nibble lookups. Under a token the level's own
carrier is `I::U8x16`, made with `I::U8x16::load(cpu, bytes)`: a value
of it is the proof the CPU has its instructions, which is why the
constructor asks for the token.

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
`Words::find_run`, overlapping `delta_swap` masks, `bytes_ge` above
128, grid sizes), the domain is in the method's docs, checked with
`debug_assert!` in debug builds, and unspecified in release. Run your
tests in debug once.

`no_std`, zero dependencies, stable Rust. The one allocating convenience
(`Rank9Buf`) sits behind the default `alloc` feature; with
`default-features = false` nothing in the crate allocates.

## Checks

Every cell is one Nix derivation built in a sandbox without network,
on every push to `main`. The badges are written by CI after each run
(`.github/status.sh`) and read from the [site](https://mightsleep.github.io/hakmem/).
A lot of cells for a crate of one-liners; the one-liner wrong for one
input in 65 536 is the bug they exist for.

| check | `x86_64` | `aarch64` |
|---|---|---|
| tests, portable | [![hakmem-test-portable-default](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-test-portable-default.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-test-portable-default](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-test-portable-default.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| tests, `+bmi2,+pclmulqdq,+avx2` | [![hakmem-test-bmi2-default](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-test-bmi2-default.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| tests, `+bmi2` with feature `portable` | [![hakmem-test-bmi2-portable-feature](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-test-bmi2-portable-feature.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| tests, debug build (asserts, overflow checks) | [![hakmem-test-debug](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-test-debug.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-test-debug](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-test-debug.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| doctests (this README), portable | [![hakmem-doctest-portable](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-doctest-portable.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-doctest-portable](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-doctest-portable.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| doctests, `+bmi2` | [![hakmem-doctest-bmi2](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-doctest-bmi2.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| clippy, portable | [![hakmem-clippy-portable-default](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-clippy-portable-default.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-clippy-portable-default](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-clippy-portable-default.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| clippy, `+bmi2` | [![hakmem-clippy-bmi2-default](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-clippy-bmi2-default.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| clippy, `+bmi2` with feature `portable` | [![hakmem-clippy-bmi2-portable-feature](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-clippy-bmi2-portable-feature.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| rustdoc, warnings as errors | [![hakmem-doc](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-doc.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-doc](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-doc.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| public API matches `public-api/` | [![hakmem-public-api](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-public-api.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-public-api](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-public-api.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| instructions, `+bmi2,+pclmulqdq,+ssse3,+avx2` (`codegen/`) | [![hakmem-codegen-bmi2](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-codegen-bmi2.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| instructions, no flags (`codegen/`) | [![hakmem-codegen-portable](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-codegen-portable.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| inlining in the benches' binaries, no flags (`codegen/`) | [![hakmem-codegen-outlined-portable](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-codegen-outlined-portable.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| inlining in the benches' binaries, `x86-64-v3` (`codegen/`) | [![hakmem-codegen-outlined-v3](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-codegen-outlined-v3.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| innermost bench loops by hakmem function, llvm-mca cycles, no flags (`codegen/`) | [![hakmem-codegen-loops-portable](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-codegen-loops-portable.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| innermost bench loops by hakmem function, llvm-mca cycles, `x86-64-v3` (`codegen/`) | [![hakmem-codegen-loops-v3](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-codegen-loops-v3.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| bare metal and kernels: `x86_64-unknown-none`, `aarch64-unknown-none{,-softfloat}` | [![hakmem-none](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-none.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |
| MSRV 1.89 build of the packaged tarball | [![hakmem-msrv](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-msrv.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-msrv](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-msrv.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| what `cargo publish` uploads matches `release/package.txt`, the version its changelog heading | [![hakmem-package](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-package.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-package](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-package.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| cargo-deny (licences, bans, sources) | [![hakmem-deny](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-deny.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-deny](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-deny.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| cargo-audit (advisories, offline) | [![hakmem-audit](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fhakmem-audit.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![hakmem-audit](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fhakmem-audit.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| treefmt | [![treefmt](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Ftreefmt.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![treefmt](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Ftreefmt.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| `.github/` is what `nix/workflows.nix` renders; actionlint, shellcheck, zizmor | [![workflows](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fworkflows.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | [![workflows](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Faarch64-linux%2Fworkflows.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-aarch64.yml?query=branch%3Amain) |
| Miri over the intrinsics, both paths | [![miri](https://img.shields.io/endpoint?url=https%3A%2F%2Fmightsleep.github.io%2Fhakmem%2Fstatus%2Fx86_64-linux%2Fmiri.json&style=flat-square)](https://github.com/mightsleep/hakmem/actions/workflows/ci-x86_64.yml?query=branch%3Amain) | n/a |

### How the checks got here

Each row answers a question a green test suite did not. The laws came
first: every combinator against a bit-loop model, on every carrier, with
and without the hardware paths. The `public-api/` files are one per
target since the instruction-set tokens made `x86_64` and `aarch64`
export different things. The instruction cells hold the README's claims
to the compiler's output, and the first one found `select` paying for
its popcount in software under the flags the README recommended.

The last four rows came from a regression nothing saw. Routing `Word`
through the tokens left `pext`'s `#[inline]` behind, and the 2D Hilbert
decode ran eight times slower in a build without flags, with every
check green: the tracked benches build with `target-cpu=native`, where
PEXT is one instruction however it inlines, and the instruction cells
compile one codegen unit, where everything inlines. `perf` found it.
Now two cells read the benches' linked binaries, built the way a
dependency is, and check in the hakmem functions that survived there as
functions of their own; two more take every loop a bench times apart by
the hakmem function behind each instruction, with llvm-mca's cycles.
Put the missing `#[inline]` back and both fail with the line that says
why. Clippy's `missing_inline_in_public_items` now asks before the fact,
and on its first run it found `Rank9::rank` and `select` out of reach of
any caller outside the crate: 6 to 39 % back.

The `.github/` row came from the 0.2.0 release, whose first run stopped
on a checkout a GitHub action had left dirty, in a step order nothing
local ever ran. The workflows are Nix values now (`nix/workflows.nix`),
rendered by a small emitter of our own with every action pinned by
commit (the flake is written with `|>`, so evaluating it needs the
`pipe-operators` experimental feature); the check holds the committed files to the rendering, parses
them back to what they were rendered from, and lints them. The release
workflow runs on pull requests as a rehearsal, semver runs from Nix
instead of an action, and every release packs its tarball twice, from
two checkouts, since a rerun recognises its own upload on crates.io by
the checksum.

Design: [`docs/design.md`](https://github.com/mightsleep/hakmem/blob/main/docs/design.md),
the decisions behind the API, the hardware policy, how the laws are
verified, and where every combinator comes from.
Changes: [`CHANGELOG.md`](https://github.com/mightsleep/hakmem/blob/main/CHANGELOG.md).

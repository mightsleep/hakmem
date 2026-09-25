# Changelog

## 0.2.0, unreleased

`hakmem::hilbert::Hilbert2`: 2D Hilbert indices over any `Word`,
full width or by order. The decode is two suffix XORs over the Morton
code (Hacker's Delight 16-2) and a Morton decode. The encode's
per-level maps generate `AGL(2, 2) ≅ S₄`, so no parity and no adder
computes them; a bit-sliced Kogge–Stone over the affine maps does,
rawrunprotected's 2016 construction with the linear parts folded
into GF(4), the adder's carry chain over a four-element field, here
on any carrier; four rounds for a `u64` after a pairing round, four
to seven times the speed of the four-state loop (cookbook recipe 13,
design notes section 8). Benched against `fast_hilbert` and `lindel`
in `benches/hilbert.rs` and the README. `hakmem::hilbert3::Hilbert3` and
`Morton3`: the 3D curve of rawrunprotected's tables, whose frames are
`A₄ ≅ AGL(1, 4)`, so the decode is the 2D encode's scan in log depth
(1.6 to 1.7 times the table loop); the encode, whose transition monoid
has no structure, is the twelve-state machine memoised into a
table of 96 bytes built by a `const fn` from the algebra,
on par with the incumbent. Both tables factor through
`A₄ = V₄ ⋊ C₃`, 24 entries between two XORs a level, checked at
compile time. Checked against the published tables on
every cell of every order up to 5. Cookbook recipe 14. Laws against the
textbook `xy2d` / `d2xy` loops, the path property and the order
recursion, exhaustive on `u16` up to order 8. `Bits::suffix_xor` and
the `Word::xor_scan_down` primitive behind it, with a PCLMULQDQ path
(the high half of the product by all-ones); `gray_decode` is now that
scan by another name.

Batch conversions, in place over a slice of keys the caller owns:
`Hilbert2::from_morton_in_place` / `to_morton_in_place` and the same
for `Hilbert3`, on `u32` and `u64`. The caller fills the keys with
Morton codes from any point layout; the kernel sees words only (design
notes section 3). With AVX-512 VBMI a step is one `vpermi2b` per
register of keys: in 2D through 128 entries, three levels a step, the
frame taken modulo the reflection of both axes and carried as a XOR
mask on the cells (the unreduced table would be 256); in 3D through
the 96-byte encode table, a level a step, the frame riding in the
index byte. With NEON the 2D reduction two levels a step, 32 entries
in two registers for `tbl`, and the 3D tables in `tbl` and `tbx`. With
AVX2 alone the 3D machine modulo its translations, 24 entries in two
PSHUFB a level: 5.3 ns a key encoding and 5.7 decoding against 12.6
and 8.7 per key (`x86-64-v3` on the same core). Otherwise the per-key
conversion. On Zen 5: 1.6 ns a key for the 2D `u64` encode from
coordinates against 11.9 for `fast_hilbert`, 0.90 against 8.3 for
16-bit coordinates, 1.7 against 15.2 for the 3D tables, the `u64` keys
in byte planes: a plane is one level of 64 keys, so a `vpermi2b`
serves 64 keys and not eight, for two transposes by `vpmultishiftqb`
and three rounds of `vpermt2b` each way. Laws: the
batch equals the per-key conversion both ways, on every length to 300
(the tails of the 16-, 32- and 64-key batches) and around the 512-key
groups of the planes. Miri runs the AVX2 kernel
(`nix run .#miri-hakmem`, a new cell) and not yet the VBMI ones, whose
AVX-512 intrinsics Miri is still learning (rust-lang/miri#5345); the
`+bmi2` CI cell runs AVX2 natively, and the NEON kernels run on the aarch64 CI
runner. The 3D decode batches too,
through the inverse of the encode table (a bijection of the octants per
state, checked at compile time): 1.8 ns a key against 8.8 for the
algebraic scan per key and 15.1 for the tables. `_order` forms of all
four for curves of fewer levels, as `encode_order` / `decode_order` per
key: a lane swap (2D) or a rotation of every bit triple (3D) over the
Morton codes, then the full-width batch. The per-key 3D encode indexes
its table padded to 128 entries under a mask: no bounds check, and the
loop over points still vectorises, 18.3 µs per 1024 points portable and
13.2 with `+bmi2` against 19.0 and 15.0 for the tables, where it tied
or trailed before.

`Hilbert3::<u64>::encode_columns` and `decode_columns`: three columns of
coordinates to indices and back, no Morton code on the way. With
AVX-512 VBMI and GFNI the columns feed the byte planes directly, one
`vpmultishiftqb` per axis at offsets `l`, `l - 1` and `l - 2`, which the
rotation of the multishift makes right at `l = 0`; the way back leaves
level `i` at byte `7 - i` and a `gf2p8affineqb` transposes each key's
8 × 8 bits into a byte of `x`, one of `y` and one of `z`. 1.2 ns a point
encoding and 1.0 decoding, against 1.7 and 1.8 through Morton codes
and the batch. Elsewhere the Morton code and the batch. Laws: the
per-point conversion both ways, full-width coordinates whose bits
above 21 must drop, every length to 200 and around the groups; a new
Miri run with GFNI.

`Morton2::encode_columns` and `decode_columns` on `u32` and `u64`: with
AVX2 a coordinate's bytes widen to 16 bits, each nibble takes a byte,
and one PSHUFB through 16 entries spreads it over the even bits (the
odd for `y`); decoding, two PSHUFB per axis and a pack inside 16 bits.
0.14 ns a point encoding and 0.22 decoding on `u64`, three times
PDEP and PEXT and seven to nine times the portable form a build
without flags had. Chosen at run time; per point elsewhere.

`Hilbert2::encode_columns` and `decode_columns` on `u32` and `u64`, the
shape geodata arrives in: the Morton columns into the output, then
`from_morton_in_place`, 1.3 ns a point on `u64` with or without build
flags, against 2.8 through the per-point Morton code in a build without
them. Decoding goes per key to Morton codes in blocks of 256 on the
stack and through the Morton columns. Skipping the Morton code as
`Hilbert3` does would need 2D byte planes for a tenth of the time.

`Hilbert2::cover` and `Morton2::cover`: a rectangle as at most
`out.len()` sorted inclusive ranges of keys holding every cell of it,
exact when the budget allows. The depth comes from counting, not
walking: a run starts where the curve's predecessor leaves the
rectangle, which on Z-order is a borrow and a count of arithmetic
progressions per level, and on Hilbert a step entering across a side,
counted per frame by a small automaton over one column of nodes, the
frames read off the indices of the four corners (the decoder's frames
form the Klein group, so a frame is two parities). A guess from the
perimeter and a step or two find the deepest level with at most two
budgets of runs. Then two walks, three levels a step with the 64
descendants as one `u64` mask per frame: one writes
the gaps into the output as scratch and counts them by bit length,
the threshold that leaves `budget` runs is found by class (bit length
and three bits of mantissa, the classes of an offset allocator, a
quickselect only when a class holds several values), and the other
closes the gaps below it. The
least over-cover that cover allows, no allocation. Laws cell by cell
on the `u16` grid (coverage, exactness, the optimal merge), by points
on `u64`, and the counts against the walk on every `u8` rectangle.
0.3 µs a rectangle for 16 Morton ranges, 1.0 for Hilbert, 1.0 and 2.8
for 64.

`Hilbert2::intersects` and `Morton2::intersects`: whether a block of
keys `a..=b` holds a cell of a rectangle, the pruning test of a sparse
index over data sorted on the curve (ClickHouse runs it per granule,
by cutting the interval into aligned blocks and decoding each). Here
one descent: a node is an interval of keys and a square of cells, so
either disjointness rules it out and either containment (with the
other met) settles it, and only the nodes on the paths of `a` and `b`
are partly in the interval; three levels a step, the interval and the
rectangle each two masks of the 64 descendants. On `u64`, 40 to 90 ns
for a hit, 100 for a near miss, 10 for a granule nowhere near; a
level at a time it was 5 to 7 times that, except the last, which is
the one the first benchmark measured. Laws against
the cells on `u16` and against `cover` on `u64`, whose gaps must miss.

The batch conversions dispatch at run time on `x86_64`: each kernel
compiled under its own `#[target_feature]`, chosen once a call by
CPUID and XCR0 (`cpu.rs`, `core::arch` and one atomic, so still
`no_std` and without dependencies), and by the compiler alone when
the build has the features. A build without flags gets the kernels:
the 2D `u64` encode from coordinates 2.8 ns a point against 6.9, the
3D columns 1.2 and 1.1 as with `target-cpu=native`. The `portable`
feature and Miri keep the compile-time choice.

`hakmem::lanes`: the SIMD half of the algebra on stable Rust. `Lanes` is
the trait for independent 8-bit lanes (bitwise, wrapping add and
subtract, per-lane shifts, unsigned compares to masks, the 16-entry
table lookup of PSHUFB / `tbl`, and `to_bitmask`, the fold of a lane mask
into a `Word` where the carry algebra takes over). Carriers: `U8x8`,
eight lanes in a `u64` by SWAR, and `U8x16`, sixteen lanes on SSSE3 or
NEON, two SWAR halves otherwise. Laws: per-lane definitions, table
composition, agreement with `Bits`' byte tests, and the wide carrier
against its halves; the SWAR carrier is swept over every byte pair. The
five lane-only primitives the SIMD corpus is written in: `shuffle` (byte
permute by data), `concat_shift` (a window across two registers),
`add_sat` / `sub_sat`, `unpack_lo` / `unpack_hi` (interleave), and the
two horizontal ones, `sum_abs_diff` and `mul_add_pairs`.

`hakmem::affine::Affine8`: the affine maps on the bits of a byte, 8×8
matrices over GF(2) with a constant, `const` throughout, composition
as matrix multiplication. `Lanes::affine` applies one to every lane
(`gf2p8affineqb` with `+gfni`, two nibble lookups by linearity
otherwise, parity folds on the SWAR carrier); `reverse_bits`, `sra`,
`rotl` and `rotr` are named maps, and so are the x86 lane shifts under
GFNI. `Lanes::avg_round` / `avg_floor` (PAVGB, `urhadd`; Hacker's
Delight 2-5). `Bits::ternary` and `Lanes::ternary` with
`bits::truth_table`: VPTERNLOG's immediate is the function at
`(0xF0, 0xCC, 0xAA)`. `Bits::signed_add_overflows` /
`signed_sub_overflows` from the sign bits (Hacker's Delight 2-13).
Cookbook recipes 8 to 10. MSRV 1.89, for the GFNI intrinsics.

The chess programming canon, first two: `Bits::next_subset` and
`Bits::subsets`, the carry-rippler over the subsets of a mask, and the
gather family, `Bits::gather` with `gather_factor` / `gather_factor_by`,
a multiply as PEXT (Kindergarten bitboards), with `gather_is_exact_by`
as the brute-force check and `strided_gather_is_exact` as the theorem
that covers files and diagonals. Cookbook recipes 11 and 12.

`hakmem::curve`: `Curve2` over `Morton2` and `Hilbert2`, `Curve3` over
`Morton3` and `Hilbert3`, for an index that should change curve by
changing a type: `encode`, `decode`, `key`, `from_key`, and in 2D
`cover` and `intersects`. `From` between a Hilbert key and the Morton
code of the same cell. Keys and `Wide` print as the numbers they are
in `{:b}`, `{:x}` and `{:X}`, default to zero, and `Wide` orders as
an unsigned integer. `cover` and `intersects` take any range, `..`
included, and `cover` returns the part of the buffer it filled.

`examples/`: a spatial index over row groups, the primes below 10^7
with `Rank9`, the first stage of simdjson, and fuzzy search, written the
way a user would write them. Writing them found the next four:
`Words::bit`, `set_bit` and `clear_bit`; `Bits::prefix_xor_carry`, the
prefix parity over a stream of words in the shape of `find_escaped`, so
the two chain block by block alike; `myers::Search::occurrences`, one
`Occurrence` per run of ends at its best end, with its start found by
walking back with the pattern reversed (the merge was the part the
example got wrong first); and `alloc` on by default.

The public API is checked in under `public-api/`, one file per target
(the instruction-set tokens differ by architecture), and a nix check
fails when the crate says otherwise, so a change to it is in the diff of the
commit that makes it; cargo-semver-checks runs against the last release
in its own workflow. Methods answer to their instruction names in the
docs search (`pext`, `pdep`, `popcnt`, `movemask`, `pshufb`, `z-order`),
and the two claims that something does not compile are doctests that
fail to, with the error code.

`hakmem::isa`: instruction sets as values (design notes section 11).
A caller that dispatched the usual way, a loop under
`#[target_feature]` picked after a CPUID check, got the portable
primitives anyway, since `cfg` is decided once per crate: 291
instructions and no PEXT for a `compact`. A token (`Portable`,
`Native`, `X86V2` for x86-64-v2, `X86V3` for x86-64-v3 with PCLMULQDQ,
`X86V4` for x86-64-v4 with VBMI and GFNI) is a zero-sized proof that
the CPU has its features; `dispatch!` detects once and runs a body
compiled for the level it found, and `cpu.pext(x, m)` in it is a PEXT
in a build without flags. `detect` takes the higher of the build and
the CPU, so a build that already proves the level keeps `Native` and
the dispatch folds away; `available` lists every level the machine
can run.
`Word` routes its five hardware primitives through `_in(…, isa)`
forms, the plain methods being `Native`; `hakmem::x86` has the leaves,
safe `#[target_feature]` functions with exactly the features they
use, for code under someone else's dispatch. Sixteen lanes are per
level: `X86x16<L>`, `Swar16`, `Neon16`, `I::U8x16` under a token and
`U8x16` the build's; under `X86V4` a byte map is one `gf2p8affineqb`
without flags. `Words::count_ones`, `rank`, `select` and the new
`for_each_position`, and `Rank9::build`, `rank` and `select`, ask the
CPU once a call: without flags, at 1024 words, 3.4 times the speed of
`count_ones` and 3.5 of `select`, twice `Rank9`'s dense select, and
no slower at one word or with `-C target-cpu=native`. `_in` forms take
a token for loops that have chosen. The batch kernels take the level
too, so a CPU with VBMI and no GFNI (Cannon Lake) now runs the AVX2
ones. `tests/isa.rs` checks every level the machine has in one
`cargo test`, where it used to take one build per `RUSTFLAGS`.
Soft-float x86 targets (`x86_64-unknown-none`, kernels) build again:
the tokens, `X86x16`, the batch kernels and the PCLMULQDQ leaf want
SSE2 in the build, and `detect` there answers `Portable` without
asking the CPU, whose answer is about userspace's registers. A kernel
built with `+bmi2` still gets PEXT through `Native`.

`codegen/` holds the README's claims about instructions as FileCheck
directives next to the wrappers they are about, and the count of each
mnemonic per function as a snapshot a nix check diffs, like the public
API: a build without flags and one with `+bmi2,+pclmulqdq,+ssse3,+avx2`.
It found `select` paying for its popcount in software under the flags
the README recommended. Miri runs its three cells side by side in two
minutes where the old six took twenty; the AVX-512 cells wait for Miri
to interpret AVX-512 (rust-lang/miri#5345).

### Changed, for anyone on 0.1

A clean break, no deprecated aliases: 0.2 is a new major for Cargo,
and the old names were wrong in ways an alias would have kept.

- `Word` asks for `Ord`, `Hash`, `Binary`, `LowerHex`, `UpperHex`,
  `Send`, `Sync` and `'static` besides `Copy + Eq + Debug`, and says
  out loud that `BITS` is a multiple of 8, at most `2^16`. Every
  carrier in the crate already was all of it; a carrier of your own
  may need derives, and an `Ord` that compares from the top.
- `Word::select_lowest` returns `BITS` when there is no `k`-th set bit,
  on every build and every carrier; `word::select_broadword64` returns
  64. It used to panic, return garbage or return 64, by flags.
- The `slice` functions are methods of the `Words` trait on `[W]`, in
  the prelude: `slice::rank(&words, i)` is `words.rank(i)`, and so on.
  `popcount` is `count_ones`; `next_set_after` is `next_set_from`,
  which is what it always did.
- `Rank9::build` returns the directory it built; `Rank9::new`, which
  views directories built before, is `Rank9::from_parts`.
- `grid::set_block(…, true)` and `(…, false)` are `fill_block` and
  `clear_block`.
- `myers::edit_distance::<W>` is `distance_in::<W>` and
  `min_distance::<W>` is `substring_distance::<W>`; `distance` still
  picks the width itself.
- `Dilated::into_int` is `to_int`, as `Copy` types spell it.
- The prelude trades `Positions` (name it from `hakmem::set`) for
  `Words`, `Lanes`, `Curve2` and `Curve3`.
- `alloc` is a default feature; `default-features = false` is the
  crate with nothing that allocates, as it was.
- `myers::search` borrows the pattern for as long as the text: the
  search keeps it, to find where each occurrence starts.
- The `Lanes` constructors take the carrier's token, `splat(isa, b)`,
  `load(isa, bytes)` and `zero(isa)`, and a carrier has `type Isa` and
  `isa()`: a value of an SSSE3 carrier is the proof the CPU has SSSE3.
  `U8x16::load(bytes)` and `U8x8::splat(b)` read as before, being
  inherent; code generic over `L: Lanes` passes `x.isa()`.
- `U8x16` is an alias for the build's carrier (`X86x16<Native>`,
  `Neon16` or `Swar16<Native>`), not a type of its own.
- `Word` gains `pext_in`, `pdep_in`, `select_lowest_in`, `xor_scan_in`
  and `xor_scan_down_in`, with portable defaults, and the plain methods
  call them with `Native`. A carrier of your own that overrode `pext`
  for speed overrides `pext_in` now, or a token gets the default.

## 0.1.0, 2026-09-19

First release, of a learning project far from production; the README's
Status section says what to expect. One trait (`Bits`) over
`u8`…`u128` and `Wide<N>`: runs, rank/select/positions, scans (prefix
XOR/OR, Gray, `find_escaped`, Kogge–Stone fills), compact/expand,
Hacker's Delight ch. 2 basics and Gosper's hack, SWAR byte lanes, delta
swaps and 8×8 board permutations, dilated integers and Morton codes,
slice and 2D-grid operations, Myers edit distance / search. Every
combinator ships with laws; laws are property-tested on all carriers
with and without BMI2/PCLMULQDQ and exhaustively for widths up to 16
bits. `hakmem::cookbook` explains how the shipped kernels were composed.
Every primitive has a constant-time portable definition; `pext` and `pdep`
follow Hacker's Delight 7-4 and 7-5. `hakmem::rank9::Rank9` is an O(1)
rank/select directory over a borrowed `&[u64]`, rank9 plus a
select9-shaped inventory, with storage the caller supplies, no allocator,
pinned to `slice` by laws.

API decisions before the first release: `Word` is open, with portable
defaults for the derivable primitives, so a carrier outside the crate
is one `impl` block and the laws are its acceptance test. `rank_below`
is `rank` everywhere. Run lengths and fill strides are total. Added
`slice::positions`, `myers::distance` (carrier chosen by pattern
length), `permute::board8::{Dir, slide}` for sliding attacks in all
eight directions, and, behind the `alloc` feature, `rank9::Rank9Buf`.

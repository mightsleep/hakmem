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
`Hilbert2::from_morton_in_place` / `into_morton_in_place` and the same
for `Hilbert3`, on `u32` and `u64`. The caller fills the keys with
Morton codes from any point layout; the kernel sees words only (design
notes section 3). With AVX-512 VBMI a step is one `vpermi2b` per
register of keys: in 2D through 128 entries, three levels a step, the
frame taken modulo the reflection of both axes and carried as a XOR
mask on the cells (the unreduced table would be 256); in 3D through
the 96-byte encode table, a level a step, the frame riding in the
index byte. With NEON a 64-entry 2D table (two levels a step) and the
3D tables in `tbl` and `tbx`; otherwise the per-key conversion. On
Zen 5: 1.6 ns a key for the 2D `u64` encode from coordinates against
11.9 for `fast_hilbert`, 0.90 against 8.3 for 16-bit coordinates, 2.7
against 15.2 for the 3D tables. Laws: the batch equals the per-key
conversion both ways, on every length to 300 (the tails of the 16-,
32- and 64-key batches).
Miri runs the VBMI kernels (`nix run .#miri-hakmem`, a new cell); the
NEON kernels run on the aarch64 CI runner. The 3D decode batches too,
through the inverse of the encode table (a bijection of the octants per
state, checked at compile time): 2.7 ns a key against 8.8 for the
algebraic scan per key and 15.1 for the tables. `_order` forms of all
four for curves of fewer levels, as `encode_order` / `decode_order` per
key: a lane swap (2D) or a rotation of every bit triple (3D) over the
Morton codes, then the full-width batch. The per-key 3D encode indexes
its table padded to 128 entries under a mask: no bounds check, and the
loop over points still vectorises, 18.3 µs per 1024 points portable and
13.2 with `+bmi2` against 19.0 and 15.0 for the tables, where it tied
or trailed before.

`hakmem::lanes`: the SIMD half of the algebra on stable Rust. `Lanes` is
the trait for independent 8-bit lanes (bitwise, wrapping add and
subtract, per-lane shifts, unsigned compares to masks, the 16-entry
table lookup of PSHUFB / `tbl`, and `to_bits`, the fold of a lane mask
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

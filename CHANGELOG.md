# Changelog

## 0.2.0, unreleased

`hakmem::lanes`: the SIMD half of the algebra on stable Rust. `Lanes` is
the trait for independent 8-bit lanes (bitwise, wrapping add and
subtract, per-lane shifts, unsigned compares to masks, the 16-entry
table lookup of PSHUFB / `tbl`, and `to_bits`, the fold of a lane mask
into a `Word` where the carry algebra takes over). Carriers: `U8x8`,
eight lanes in a `u64` by SWAR, and `U8x16`, sixteen lanes on SSSE3 or
NEON, two SWAR halves otherwise. Laws: per-lane definitions, table
composition, agreement with `Bits`' byte tests, and the wide carrier
against its halves; the SWAR carrier is swept over every byte pair.

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

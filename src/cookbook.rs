//! How the shipped kernels were composed, so you can compose your own.
//!
//! This module is documentation only. Every recipe has the same shape:
//! the **problem**, the **decomposition** into combinators, the
//! **laws** that justify each step (all of them live in [`crate::laws`]
//! and run in the test suite), and the **cost** in word operations.
//! Snippets are doctests, so the text cannot drift from the code.
//!
//! # Reading a sentence
//!
//! A register is a container of `BITS` cells. Each instruction is one
//! of five shapes, and a kernel is a short sentence in them:
//!
//! | shape | combinators | what it does to the container |
//! |---|---|---|
//! | map | `and`, `or`, `xor`, `not`, `delta_swap` | every cell independently, or a fixed permutation |
//! | scan | `wrapping_add`, `prefix_xor`, `prefix_or`, `fill_up` | information flows along the word (the carry chain is the scan primitive) |
//! | filter | `compact`, `expand` | keep the cells a mask selects, in order |
//! | reduce | `count_ones`, `trailing_zeros`, `select`, `longest_run` | one number out |
//! | window | `run_starts`, `zero_bytes`, `bytes_eq` | a predicate over each cell's neighbourhood, all cells at once |
//!
//! The economics that make sentences worth writing: one cache miss is
//! worth ~300 chained word operations, and a modern core retires four
//! to six independent ones per cycle. A sentence of twelve operations
//! that replaces a loop over sixty-four cells changes the complexity
//! class per word.
//!
//! # Verifying your own
//!
//! Write the law before the kernel. A law is a `fn(...) -> bool` over
//! `W: Word` comparing your combinator against a bit-by-bit loop (see
//! `laws::reference`). Run it with proptest on `u64`, and exhaustively
//! on `u8` / `u16`, every kernel in this crate that had a bug was
//! caught by the exhaustive sweep, not by proptest. Then run the whole
//! thing with and without `-C target-feature=+bmi2,+pclmulqdq`; the
//! hardware and portable paths must agree bit for bit.
//!
//! ---
//!
//! # 1. The inside-string mask (simdjson)
//!
//! **Problem.** Given the positions of `"` and `\` in a block of text,
//! which bytes are inside a string literal? Quotes preceded by an odd
//! number of backslashes are escaped and do not count.
//!
//! **Decomposition.** Escaped positions are the ends of odd-length
//! backslash runs: [`crate::Bits::find_escaped`], a carry-chain trick, runs
//! starting on even and on odd positions are added separately, so each
//! run's end parity falls out of an alternating mask. Real quotes are
//! `quotes & !escaped`. Each real quote *toggles* the in-string state,
//! and a toggle mask becomes a region mask by [`crate::Bits::prefix_xor`]
//! (PCLMULQDQ by all-ones, or a smear).
//!
//! ```
//! use hakmem::Bits;
//!
//! // text:    a " b \ " c " d
//! // index:   0 1 2 3 4 5 6 7
//! let quotes: u8 = 0b0101_0010;
//! let backslashes: u8 = 0b0000_1000;
//! let (escaped, _carry) = backslashes.find_escaped(false);
//! let real = quotes & !escaped; // the quote at 4 is escaped
//! let inside = real.prefix_xor();
//! assert_eq!(inside, 0b0011_1110); // bytes 1..=5 are in the string
//! ```
//!
//! **Laws.** `find_escaped_matches_reference` (parity walk, carry
//! included); `prefix_xor_after_delta_is_identity` and its converse:
//! toggles and regions are inverse views, so nothing is lost either
//! way; `xor_linear_combinators`, `prefix_xor` distributes over XOR,
//! which is why two independent toggle masks can be combined first and
//! scanned once.
//!
//! **Cost.** `find_escaped` is ~14 operations with two additions;
//! `prefix_xor` is one instruction with PCLMULQDQ, six otherwise. Per
//! 64 bytes of text.
//!
//! # 2. First fit of a run across words
//!
//! **Problem.** In a bitmap of free slots, find the lowest position
//! where `k` consecutive slots are free, when the run may straddle a
//! word boundary.
//!
//! **Decomposition.** Inside one word, [`crate::Bits::run_starts`] (Hacker's
//! Delight fig. 6-5): repeat `x &= x >> s` with `s` halving; after
//! `⌈log₂ k⌉` steps bit `p` is set iff bits `p..p + k` were all set.
//! Across the boundary there is exactly one candidate: the word's top
//! run of ones (`leading_ones`) continued by the next word's bottom run
//! (`trailing_ones`). No wider window is needed; an earlier version
//! of this kernel widened to `u128` and was 1.6–2.2× slower for it.
//! Shipped as [`crate::slice::find_run`].
//!
//! ```
//! use hakmem::slice::find_run;
//!
//! // 60 free slots, then 4 taken, then all free: a 6-run must straddle.
//! let words = [u64::MAX >> 4, u64::MAX];
//! assert_eq!(find_run(&words, 6), Some(0));
//! let words = [0xFFu64 << 56, u64::MAX];
//! assert_eq!(find_run(&words, 16), Some(56));
//! ```
//!
//! **Laws.** `run_starts_composes`: `run_starts(a) ∘ run_starts(b) =
//! run_starts(a + b − 1)`. This is *why* the halving chain is
//! correct, each halving step is a composition. `run_starts_shrinks`
//! (longer runs start at a subset of positions) is what makes binary
//! search for [`crate::Bits::longest_run`] valid. `slice_ops_match_reference`
//! covers the boundary case.
//!
//! **Cost.** ≤ 6 operations per word for `k ≤ 64` plus two count-ones
//! at a boundary; the scan is memory-bound, not ALU-bound.
//!
//! # 3. The Myers column step
//!
//! **Problem.** Levenshtein distance is a dynamic-programming table;
//! compute it without touching cells one at a time.
//!
//! **Decomposition.** Adjacent cells in a DP column differ by −1, 0 or
//! +1, so a column of `m` cells is two `m`-bit words: `Pv` (delta +1)
//! and `Mv` (delta −1). Matches of the next text character are a mask
//! `Eq`. The step is
//!
//! ```text
//! Xh = ((Eq & Pv) + Pv) ^ Pv | Eq     // matches propagate down the column
//! Ph = Mv | !(Xh | Pv)                // horizontal deltas out
//! Mh = Pv & Xh
//! score += top bit of Ph, −= top bit of Mh
//! Pv = (Mh << 1) | !(Xv | (Ph << 1))  // shift deltas one row down
//! Mv = (Ph << 1) & Xv
//! ```
//!
//! The addition is the whole trick: a carry chain is a scan, and the
//! DP's "a match at row `i` lets the score at row `i+1` stay flat"
//! is exactly carry propagation. The DP boundary `D[0][j] = j` is one
//! bit shifted into `Ph` (global) or not (semi-global search). Shipped
//! as [`crate::myers`], written against [`crate::Word`] so that
//! [`crate::wide::Wide`] runs it unchanged for long patterns.
//!
//! ```
//! use hakmem::myers::{edit_distance, search};
//!
//! assert_eq!(edit_distance::<u64>(b"kitten", b"sitting"), Some(3));
//! let ends: Vec<_> = search::<u64>(b"lo", b"hello lo", 0).unwrap().collect();
//! assert_eq!(ends, [(5, 0), (8, 0)]);
//! ```
//!
//! **Laws.** None of the exported ones, the kernel is verified against
//! the textbook DP by property tests on `u8`, `u64`, `u128` and
//! `Wide<4>` (`tests/myers.rs`). That is the honest way to say "this
//! sentence has no local algebraic proof, only a global one".
//!
//! **Cost.** ~12 operations per text byte, independent of pattern
//! length up to the word width; `N` limbs for `Wide<N>`.
//!
//! # 4. Morton neighbours without decoding
//!
//! **Problem.** Cells of a 2D grid are stored in Z-order (Morton
//! codes). Step to the east neighbour without splitting the code into
//! `(x, y)` and back.
//!
//! **Decomposition.** A Morton code interleaves `x` and `y`, so `x`
//! lives in the even bits, a *dilated* integer. Adding one to a
//! dilated integer must carry across the zero gaps: fill the gaps with
//! ones first, add, mask back. [`crate::Dilated::incr`] does exactly that,
//! and [`crate::Dilated`] has no `+` on purpose, plain integer addition on
//! a dilated value is the bug this type makes uncompilable.
//!
//! ```
//! use hakmem::Morton2;
//!
//! let m = Morton2::<u32>::encode(3, 5);
//! assert_eq!(m.step_x().decode(), (4, 5));
//! assert_eq!(m.step_y().decode(), (3, 6));
//! ```
//!
//! **Laws.** `dilated_incr_is_add_one`, `dilated_add_is_add`
//! (dilation is a homomorphism of addition modulo the width),
//! `morton_steps_are_unit_moves`, `morton_aligned_block_is_contiguous`
//! (aligned power-of-two blocks are contiguous code ranges, the
//! reason a 1D allocator can run over a 2D texture).
//!
//! **Cost.** `incr` is three operations; `encode` / `decode` are two
//! PDEP / PEXT, or a five-step spread without BMI2.
//!
//! # 5. `strlen` in one word
//!
//! **Problem.** Find the first zero byte of a word, the inner loop of
//! `strlen`, `memchr` and every delimiter scan.
//!
//! **Decomposition.** [`crate::Bits::zero_bytes`] sets the high bit of every
//! zero lane, exactly: `!((x & 0x7f…) + 0x7f… | x | 0x7f…)`. The
//! classic `(x − 0x01…) & !x & 0x80…` is one operation shorter and
//! *wrong* above the first zero byte (a borrow leaks into the next
//! lane); it is fine for "is there any", not for "which". Then
//! `trailing_zeros / 8` names the lane. [`crate::Bits::bytes_eq`] is the
//! same sentence after `x ^ splat(b)`.
//!
//! ```
//! use hakmem::Bits;
//!
//! let word = u64::from_le_bytes(*b"hello\0!!");
//! assert_eq!(word.first_zero_byte(), Some(5));
//! assert_eq!(word.bytes_eq(b'l').count_ones(), 2);
//! ```
//!
//! **Laws.** `swar_lanes_match_reference` (every lane predicate against
//! a byte loop, exhaustive over all bytes for `u8`/`u16`);
//! `zero_bytes_turns_or_into_and` (a lane is zero in `x | y` iff zero
//! in both), the De Morgan shape that lets you test two words with
//! one scan.
//!
//! **Cost.** Four operations for the mask, one `trailing_zeros` for
//! the index; eight bytes per iteration instead of one.
//!
//! # 6. Sliding-piece attacks (Kogge–Stone)
//!
//! **Problem.** On an 8×8 bitboard, which squares does a rook attack
//! northward, given the occupied squares?
//!
//! **Decomposition.** "Slide until blocked" is a prefix OR along a
//! stride of 8 through a propagation mask (the empty squares):
//! [`crate::Bits::fill_up`]. Kogge–Stone doubles the stride each round
//! (`gen |= prop & gen << s; prop &= prop << s`), three rounds for
//! eight ranks. The attacked set includes the blocker: shift the fill
//! once more.
//!
//! ```
//! use hakmem::Bits;
//!
//! let rook = 1u64; // a1
//! let occupied = 1u64 << 32; // a5
//! let attacks = rook.fill_up(!occupied, 8) << 8;
//! assert_eq!(attacks & 0x0101_0101_0101_0101, 0x0000_0001_0101_0100); // a2..a5
//! ```
//!
//! **Laws.** `fills_match_reference` (the doubling rounds equal
//! iterating single steps), `fills_are_closure_operators` (idempotent
//! and extensive, filling twice is filling once, which is what makes
//! "occupancy changed only here" updates safe). With a full mask and
//! stride 1, `fill_up` *is* `prefix_or`.
//!
//! **Cost.** Three rounds of three operations per direction; the
//! eight directions of a queen are 72 operations, no table.
//!
//! # 7. Not built here: banded Myers over `Wide<N>`
//!
//! For two long strings that differ in few edits, `triple_accel`
//! beats recipe 3 on `Wide<8>` by about 2× (see the README table),
//! because it only computes a diagonal band. This crate does not ship
//! that variant, deliberately. Everything needed is here.
//!
//! A `Wide<N>` column is `N` limbs. Run the step of recipe 3 per limb
//! with an explicit horizontal delta between limbs: the top bit of
//! `Ph` / `Mh` leaving limb `i` is the bit shifted into limb `i + 1`
//! (this is exactly what [`crate::Word::wrapping_add`] does across limbs
//! implicitly, the banded version makes the carry explicit so it can
//! stop). Keep the score of the last active limb's bottom cell; while
//! it exceeds `k + (limbs left) × 64`, the limbs below it cannot come
//! back under `k` and need not be updated (Ukkonen's cut-off, Hyyrö's
//! block formulation). Cost drops from `N` limbs per text byte to the
//! active band, typically one or two. The laws you would write:
//! agreement with recipe 3 on the same inputs when `k` is large, and
//! `None`-agreement with the DP when it is small. Both are one
//! proptest each against `tests/myers.rs`'s reference. Do that and you
//! have your microkernel; it will be shorter than this paragraph.
//!
//! # 8. Every byte map is one instruction, or two lookups
//!
//! **Problem.** Shift, rotate or reverse the bits of every byte in a
//! register. SSE has no 8-bit shift at all; NEON has the shifts and
//! `rbit` but no rotate; and a kernel usually wants three of these in
//! a row.
//!
//! **Decomposition.** Each of those is XOR-linear in the bits of the
//! byte, so each is an 8×8 matrix over GF(2), an
//! [`Affine8`](crate::affine::Affine8); NOT adds a constant. GFNI's
//! `gf2p8affineqb` applies one to sixteen bytes in one instruction,
//! and matrix multiplication ([`then`](crate::affine::Affine8::then))
//! composes them, so a chain of byte tricks is one constant and one
//! instruction. Without GFNI, linearity gives `A·x = A·hi ⊕ A·lo`: two
//! nibble lookups ([`lut16`](crate::lanes::Lanes::lut16)) and an XOR,
//! the tables folded at compile time. The named maps are the identity
//! matrix moved: a left shift by `n` is the identity shifted right by
//! `8 n` bits, the arithmetic shift adds `0x80` rows for the vacated
//! bits, the reversal reverses the rows (Wunkolo, 2020).
//!
//! ```
//! use hakmem::affine::Affine8;
//! use hakmem::lanes::{Lanes, U8x16};
//!
//! let x = U8x16::load(b"0123456789abcdef");
//! // Reverse the bits, then shift left by one, as one map.
//! let map = Affine8::REVERSE.then(Affine8::shl(1));
//! assert_eq!(x.affine(map), x.reverse_bits().shl(1));
//! // The matrices are the ones in Wunkolo's posts.
//! assert_eq!(Affine8::shl(1).matrix(), 0x0001_0204_0810_2040);
//! assert_eq!(Affine8::sra(2).matrix(), 0x0408_1020_4080_8080);
//! ```
//!
//! **Laws.** `affine_matches_reference` (the scalar, the SWAR word,
//! the nibble tables and the lanes against the bit-by-bit definition),
//! `affine_composes` (`then` is `apply` after `apply`; the identity is
//! neutral), `affine_named_maps_match_ops` (each named map is the `u8`
//! operation, for every shift count), `lane_maps_match_reference`.
//!
//! **Cost.** One instruction with GFNI (latency 3, one every other
//! cycle on Ice Lake, per uops.info); five without (two PSHUFB, a
//! shift, an AND, an XOR); on the SWAR carrier eight rounds of seven
//! operations.
//!
//! # 9. The truth table is the function at `(0xF0, 0xCC, 0xAA)`
//!
//! **Problem.** VPTERNLOG computes any Boolean function of three
//! registers from an 8-bit immediate. Which immediate? And the
//! question it is usually asked for: signed saturation on lanes that
//! have no signed saturating add.
//!
//! **Decomposition.** Bit `k` of `0xF0`, `0xCC`, `0xAA` is bit 2, 1, 0
//! of `k`, so across their bit positions the three bytes enumerate
//! the eight inputs; the function evaluated once on them is its table
//! ([`truth_table`](crate::bits::truth_table)), and
//! [`ternary`](crate::Bits::ternary) reads the table back. Signed
//! overflow needs only sign bits (Hacker's Delight 2-13): the operands
//! agree and the sum disagrees, `!(x ^ y) & (x ^ s)`. Its table is
//! `0x42`; the subtraction's, `(x ^ y) & (x ^ d)`, is `0x18`. Those are
//! the immediates in Wunkolo's saturation kernel, derived instead of
//! looked up. [`signed_add_overflows`](crate::Bits::signed_add_overflows)
//! is the scalar form.
//!
//! ```
//! use hakmem::Bits;
//! use hakmem::bits::truth_table;
//!
//! assert_eq!(truth_table(|a, b, s| !(a ^ b) & (a ^ s)), 0x42);
//! assert_eq!(truth_table(|a, b, d| (a ^ b) & (a ^ d)), 0x18);
//! let (a, b) = (100u8, 100u8);
//! let sum = a.wrapping_add(b);
//! assert_eq!(
//!     a.ternary(b, sum, 0x42) & 0x80 != 0,
//!     a.signed_add_overflows(b)
//! );
//! assert!(a.signed_add_overflows(b)); // 100 + 100 does not fit an i8
//! ```
//!
//! **Laws.** `ternary_is_truth_table` (against the bit-by-bit reading
//! of the table), `truth_table_names_the_function` (mux, majority,
//! XOR3 and the two overflow tests come back as themselves, with
//! `0x42` and `0x18`), `signed_overflow_matches_sign_test`; and
//! `i8::checked_add` on every pair of bytes as the outside witness.
//!
//! **Cost.** One VPTERNLOG per register with AVX-512; the provided
//! expansion is at most ten operations, fewer once a constant table
//! folds.
//!
//! # 10. Constants from registers
//!
//! **Problem.** `splat(0x80)`, the sign mask, is a load, like every
//! lane constant. In a loop that is a port and a cache line; and SSE
//! cannot build `0x80` from all-ones in registers, having no byte
//! shift.
//!
//! **Decomposition.** All-ones is a compare of a register with itself.
//! The rounding average of `0x00` and `0xFF` is `(0 + 255 + 1) >> 1 =
//! 0x80` (PAVGB; Wunkolo, 2022), and
//! [`avg_round`](crate::lanes::Lanes::avg_round) is that instruction,
//! with Hacker's Delight 2-5 as the definition where there is none:
//! `(x | y) − ((x ^ y) >> 1)` cannot overflow.
//!
//! ```
//! use hakmem::lanes::{Lanes, U8x16};
//!
//! let ones = U8x16::zero().not(); // pcmpeqb x, x
//! assert_eq!(U8x16::zero().avg_round(ones), U8x16::splat(0x80)); // pavgb
//! assert_eq!(U8x16::splat(7).avg_round(U8x16::splat(8)), U8x16::splat(8));
//! assert_eq!(U8x16::splat(7).avg_floor(U8x16::splat(8)), U8x16::splat(7));
//! ```
//!
//! **Laws.** `lane_maps_match_reference`: both averages against 16-bit
//! arithmetic on every lane, and the sign mask from the average.
//!
//! **Cost.** Two instructions, no memory.
//!
//! # 11. Every occupancy of a mask, in order
//!
//! **Problem.** Fill a table indexed by the occupancy of a set of
//! squares (a magic-bitboard table, a Kindergarten row), or search
//! every subset of a mask for a constant that works. Walking `0..2^k`
//! and depositing each with PDEP is one way; the carry-rippler is
//! the way without PDEP.
//!
//! **Decomposition.** `(x − mask) & mask` is the next subset of `mask`
//! after `x`: subtracting the mask borrows through the selected bits
//! the way adding one carries through contiguous ones, so the step
//! is `+ 1` in the compacted domain without ever compacting.
//! [`next_subset`](crate::Bits::next_subset) is the step,
//! [`subsets`](crate::Bits::subsets) the iterator from zero to the mask.
//!
//! ```
//! use hakmem::prelude::*;
//!
//! let relevant = 0b1010_0100u8;
//! let occupancies: Vec<u8> = relevant.subsets().collect();
//! assert_eq!(occupancies.len(), 8);
//! assert_eq!(occupancies[..4], [0, 0b100, 0b10_0000, 0b10_0100]);
//! // The same walk through PEXT / PDEP.
//! for (i, s) in relevant.subsets().enumerate() {
//!     assert_eq!(u8::try_from(i).unwrap().expand(relevant), s);
//! }
//! ```
//!
//! **Laws.** `next_subset_is_increment_in_mask` (the step is
//! `expand(compact(x) + 1)`), `subsets_enumerate_each_once` (`2^k`
//! items, consecutive in the compacted domain, all inside the mask).
//!
//! **Cost.** Two operations per subset, no table, no PDEP.
//!
//! # 12. A column as a byte, by one multiply (Kindergarten)
//!
//! **Problem.** The occupancy of a file, a diagonal or an antidiagonal
//! of a bitboard as a small integer, to index a table: the bits are
//! eight or nine apart and a table wants them adjacent.
//!
//! **Decomposition.** A multiply by a constant is the sum of the
//! operand shifted left by each set bit of the constant. Choose one
//! shift per selected bit, the distance from where it is to where it
//! should land, and if no two of the shifted copies ever put a bit on
//! the same position the sum is an OR and the product holds the
//! gathered bits: PEXT by arithmetic. [`gather`](crate::Bits::gather)
//! is the multiply, mask and shift; [`gather_factor`](crate::Bits::gather_factor)
//! derives the constant, and it derives the ones in the literature: a
//! file's factor is the identity matrix of recipe 8, a diagonal's
//! read by column is the a-file `0x0101…01`. For bits `stride` apart
//! with `stride ≥ k` the copies cannot meet
//! (`strided_gather_is_exact`), which covers files and diagonals;
//! anything else is one brute-force run of `gather_is_exact` over the
//! `2^k` occupancies of recipe 11.
//!
//! ```
//! use hakmem::prelude::*;
//!
//! // The a1-h8 diagonal, read by column onto the top rank.
//! let diagonal = 0x8040_2010_0804_0201u64;
//! let factor = u64::gather_factor(diagonal, 56).unwrap();
//! assert_eq!(factor, 0x0101_0101_0101_0101);
//! let occupied = diagonal & 0xFFFFu64; // ranks 1 and 2
//! assert_eq!(occupied.gather(diagonal, factor, 56), 0b11);
//! assert!(hakmem::laws::gather_is_exact(diagonal, factor, 56));
//! ```
//!
//! **Laws.** `gather_is_exact_by` (agreement with `compact` on every
//! occupancy, for any placement), `strided_gather_is_exact` (the
//! theorem that makes files and diagonals safe without the sweep),
//! and `kindergarten_gathers_are_exact` in `tests/laws.rs`, every
//! file, diagonal and antidiagonal of the board.
//!
//! **Cost.** One multiply, one shift, one AND; a 64-entry table per
//! line instead of a magic-bitboard's per-square tables.
//!
//! # 13. Hilbert: parities one way, the carry chain two bits wide the other
//!
//! **Problem.** A 2D Hilbert index to and from `(x, y)`, for the
//! locality a Z-order does not give: consecutive indices are always
//! neighbouring cells.
//!
//! **Decomposition.** A Hilbert index has the shape of a Morton code,
//! one pair of bits per level, top level first, and each level is read
//! in one of four frames (base, transposed, reflected, both) that the
//! levels above it chose. Which frame is a parity of what the pairs
//! above did: transpose when the pair is `0` or `3`, reflect when it
//! is `3`. Parities from the top are [`suffix_xor`](crate::Bits::suffix_xor),
//! one per flag, over the even lane of the index word; the odd lane of
//! the same scan holds the parity over the levels *strictly* above,
//! the exclusive scan for free. Two scans, a handful of lane
//! operations, then [`Morton2::decode`](crate::Morton2::decode).
//!
//! ```
//! use hakmem::prelude::*;
//!
//! // Order 2, the 4 × 4 curve: 0 1 2 3 climb the left column's
//! // first quadrant, 15 sits at (3, 0).
//! let cells: Vec<(u8, u8)> = (0..16u8)
//!     .map(|h| Hilbert2::from_index(h).decode_order(2))
//!     .collect();
//! assert_eq!(cells[..4], [(0, 0), (1, 0), (1, 1), (0, 1)]);
//! assert_eq!(cells[15], (3, 0));
//! // Every step is one cell.
//! for w in cells.windows(2) {
//!     let ((x0, y0), (x1, y1)) = (w[0], w[1]);
//!     assert_eq!(x0.abs_diff(x1) + y0.abs_diff(y1), 1);
//! }
//! assert_eq!(Hilbert2::<u8>::encode_order(3, 0, 2).index(), 15);
//! ```
//!
//! **The encode, and why it costs more.** With the coordinates as
//! input the frame at each level is an affine map of the frame above
//! it, `(a, c) ↦ (a ^ ¬(c ^ x), c)` when `x = y` and
//! `(a, c) ↦ (c ^ x, a ^ x)` when `x ≠ y`. The two linear parts do
//! not commute; they generate `GL(2, 2) ≅ S₃`, and with the
//! translations `AGL(2, 2) ≅ S₄`. So no parity computes the frames
//! and no adder does either. But the maps themselves depend only on
//! the input, and affine maps compose associatively, which is all a
//! Kogge–Stone scan needs: [`Hilbert2::from_morton`](crate::Hilbert2::from_morton)
//! keeps each level's map bit-sliced in six words (`M ^ I` and `b`,
//! one bit per level), composes every window with the window `s`
//! levels above it for `s = 1, 2, 4, …`, and reads the frames off the
//! vector parts, since the top frame is zero. This is exactly the
//! adder's carry chain, whose maps are the affine maps of one bit
//! (`kill`, `propagate`, `generate`, composed as `(g, p)` in three
//! operations), one bit wider: `Aff(2, 2)` instead of `Aff(1, 2)`,
//! thirty-six operations a round instead of three, and no
//! instruction for it. The decode gets away with two parities because
//! its transitions are functions of the *output* pairs, which the scan
//! has in hand from the start; the encode's are functions of the
//! output it is computing, and the affine product is what solving that
//! recurrence in log depth looks like.
//!
//! The representation is what decides the cost. Six words for a
//! general element of `Aff(2, 2)` cost 36 operations a round. But the
//! two frame maps are involutions, and a product of two involutions
//! from a group of order six has order one or three: pair the levels
//! and every window's linear part lies in the cyclic group of order
//! three, which is GF(4)*, with the translations in GF(4). Four words,
//! a GF(4) multiplication per word pair, about 24 operations a round,
//! one round fewer. This is rawrunprotected's construction (2016);
//! the six-word form is the same scan before anyone noticed the
//! pairing.
//!
//! The general pattern: a finite-state machine over a word is a prefix
//! over its transition monoid; it is a broadword kernel when the
//! monoid has a low-dimensional representation with a cheap
//! composition. `{K, P, G}` is the adder (`Aff(1, GF(2))`), this
//! recipe is `Aff(1, GF(4))`, a permutation of sixteen states is
//! [`Lanes::shuffle`](crate::lanes::Lanes::shuffle) composing tables.
//! The four-state loop this replaces is kept as
//! `laws::reference::hilbert_index_machine`. The rule is a theorem for
//! the aperiodic and modular cases (Serre 2004; Paperman, Salvati and
//! Soyez-Martin 2023) and open for groups.
//!
//! **Laws.** `hilbert_matches_reference` (both directions against the
//! textbook `xy2d` / `d2xy` loops, every cell of every order up to 8
//! on `u16`), `hilbert_roundtrip`, `hilbert_consecutive_are_adjacent`
//! (the path property, every index of the 256 × 256 curve),
//! `hilbert_order_laws` (the order-`n` curve is the first quadrant of
//! the order-`n + 1` curve transposed, and runs corner to corner).
//!
//! **Cost.** Decode: two suffix XORs (one CLMUL each with
//! `+pclmulqdq`, six operations each without) and about ten lane
//! operations, then a Morton decode; 1.6 ns per `u64` with CLMUL,
//! 3 ns without, against 50 to 80 ns for the `d2xy` loop. Encode:
//! a pairing round and four rounds of about 24 operations for a
//! `u64`, all independent within a round; 11 ns per word portable,
//! 5 ns with `+bmi2`, against 40 ns for the four-state loop and 33 ns
//! for `xy2d`. The loop is
//! bound by a three-operation carried chain per level; the scan is
//! bound by how many independent operations the core retires. Against
//! the crates people use (`benches/hilbert.rs`): `fast_hilbert`, a
//! 512-byte transition table walked three levels a step, decodes in
//! 12.5 ns and encodes in 11 ns; `lindel` (Skilling, one bit a step)
//! takes about 70 ns either way. The decode here wins by four to
//! eight times, the encode by 5 % portable and twice with `+bmi2`;
//! the table is a chain of eleven dependent loads, the scan is about
//! 120 independent operations.
//!
//! # 13. Hilbert decode is two scans; Hilbert encode is not
//!
//! **Problem.** A 2D Hilbert index to and from `(x, y)`, for the
//! locality a Z-order does not give: consecutive indices are always
//! neighbouring cells.
//!
//! **Decomposition.** A Hilbert index has the shape of a Morton code,
//! one pair of bits per level, top level first, and each level is read
//! in one of four frames (base, transposed, reflected, both) that the
//! levels above it chose. Which frame is a parity of what the pairs
//! above did: transpose when the pair is `0` or `3`, reflect when it
//! is `3`. Parities from the top are [`suffix_xor`](crate::Bits::suffix_xor),
//! one per flag, over the even lane of the index word; the odd lane of
//! the same scan holds the parity over the levels *strictly* above,
//! the exclusive scan for free. Two scans, a handful of lane
//! operations, then [`Morton2::decode`](crate::Morton2::decode).
//!
//! ```
//! use hakmem::prelude::*;
//!
//! // Order 2, the 4 × 4 curve: 0 1 2 3 fill the lower-left quadrant,
//! // 15 sits at (3, 0).
//! let cells: Vec<(u8, u8)> = (0..16u8)
//!     .map(|h| Hilbert2::from_index(h).decode_order(2))
//!     .collect();
//! assert_eq!(cells[..4], [(0, 0), (1, 0), (1, 1), (0, 1)]);
//! assert_eq!(cells[15], (3, 0));
//! // Every step is one cell.
//! for w in cells.windows(2) {
//!     let ((x0, y0), (x1, y1)) = (w[0], w[1]);
//!     assert_eq!(x0.abs_diff(x1) + y0.abs_diff(y1), 1);
//! }
//! assert_eq!(Hilbert2::<u8>::encode_order(3, 0, 2).index(), 15);
//! ```
//!
//! **Why the encode is a loop.** With the coordinates as input the
//! frame at each level is an affine map of the frame above it,
//! `(a, c) ↦ (a ^ ¬(c ^ x), c)` when `x = y` and
//! `(a, c) ↦ (c ^ x, a ^ x)` when `x ≠ y`. The two linear parts do
//! not commute; they generate `GL(2, 2) ≅ S₃`, and with the
//! translations `AGL(2, 2) ≅ S₄`. A prefix over a non-abelian group is
//! not a parity and not a carry chain, so
//! [`Hilbert2::from_morton`](crate::Hilbert2::from_morton) is
//! `BITS / 2` steps of a four-state machine, branch-free, three
//! operations on the loop-carried chain per level. The decode escapes
//! because its transitions are functions of the *output* pairs, which
//! are known before the scan starts; the encode's are functions of the
//! output it is still computing. A log-depth encode exists in
//! principle: a prefix over `S₄` as a composition of 4-entry tables,
//! which is what [`Lanes::shuffle`](crate::lanes::Lanes::shuffle)
//! does to sixteen of them at once. It is not built here.
//!
//! **Laws.** `hilbert_matches_reference` (both directions against the
//! textbook `xy2d` / `d2xy` loops, every cell of every order up to 8
//! on `u16`), `hilbert_roundtrip`, `hilbert_consecutive_are_adjacent`
//! (the path property, every index of the 256 × 256 curve),
//! `hilbert_order_laws` (the order-`n` curve is the first quadrant of
//! the order-`n + 1` curve transposed, and runs corner to corner).
//!
//! **Cost.** Decode: two suffix XORs (one CLMUL each with
//! `+pclmulqdq`, six operations each without) and about ten lane
//! operations, then a Morton decode; 1.6 ns per `u64` with CLMUL,
//! 3 ns without, against 50 to 80 ns for the `d2xy` loop. Encode:
//! 32 steps for a `u64`, about 40 ns, within 20 % of the `xy2d` loop
//! it is checked against; the Morton encode it starts from is 1.6 ns.

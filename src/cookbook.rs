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

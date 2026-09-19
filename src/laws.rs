//! The algebra's laws as exported property functions.
//!
//! Each function returns `true` when the law holds for its inputs.
//! The crate tests them against the `reference` module's bit-loop model for
//! `u32`, `u64` and `u128`, with and without the BMI2 fast paths (see
//! `tests/laws.rs`); downstream carriers and backends run the same
//! functions over their own types. A law that fails is a bug in the
//! backend, never a caveat in the docs.

use crate::affine::Affine8;
use crate::bits::truth_table;
use crate::lanes::{Lanes, U8x8, U8x16};
use crate::permute::board8::{self, Dir};
use crate::prelude::*;
use crate::rank9::Rank9;

// --- runs -------------------------------------------------------------

/// `run_starts(1)` is the identity.
#[must_use]
pub fn run_starts_one_is_identity<W: Word>(x: W) -> bool {
    x.run_starts(1) == x
}

/// Starts of `k + 1`-runs are a subset of starts of `k`-runs.
/// `k` in `1..BITS`.
#[must_use]
pub fn run_starts_shrinks<W: Word>(x: W, k: u32) -> bool {
    let (longer, shorter) = (x.run_starts(k + 1), x.run_starts(k));
    longer.and(shorter) == longer
}

/// `run_starts` agrees with the per-bit definition. `k` in `1..=BITS`.
#[must_use]
pub fn run_starts_matches_reference<W: Word>(x: W, k: u32) -> bool {
    x.run_starts(k) == reference::run_starts(x, k)
}

// --- scan -------------------------------------------------------------

/// `prefix_xor` undoes `delta_xor`.
#[must_use]
pub fn prefix_xor_after_delta_is_identity<W: Word>(x: W) -> bool {
    x.delta_xor().prefix_xor() == x
}

/// `delta_xor` undoes `prefix_xor`.
#[must_use]
pub fn delta_after_prefix_xor_is_identity<W: Word>(x: W) -> bool {
    x.prefix_xor().delta_xor() == x
}

/// `prefix_or` is all-ones from the lowest set bit upward.
#[must_use]
pub fn prefix_or_is_smear_from_first_set<W: Word>(x: W) -> bool {
    let expect = x.first_set().map_or(W::ZERO, |p| W::ONES.shl(p));
    x.prefix_or() == expect
}

/// `prefix_xor` agrees with the per-bit definition.
#[must_use]
pub fn prefix_xor_matches_reference<W: Word>(x: W) -> bool {
    x.prefix_xor() == reference::prefix_xor(x)
}

// --- set view ---------------------------------------------------------

/// `rank` is monotone in its bound. `i` in `0..BITS`.
#[must_use]
pub fn rank_is_monotone<W: Word>(x: W, i: u32) -> bool {
    x.rank(i) <= x.rank(i + 1)
}

/// `rank(BITS)` is the popcount.
#[must_use]
pub fn rank_full_is_popcount<W: Word>(x: W) -> bool {
    x.rank(W::BITS) == x.count_ones()
}

/// `first_set` and `last_set` bracket every set bit.
#[must_use]
pub fn first_last_bracket_set_bits<W: Word>(x: W) -> bool {
    match (x.first_set(), x.last_set()) {
        (None, None) => x.is_zero(),
        (Some(lo), Some(hi)) => lo <= hi && x.rank(lo) == 0 && x.rank(hi + 1) == x.count_ones(),
        _ => false,
    }
}

/// `select(k)` lands on a set bit whose rank is `k`; `None` exactly
/// when `k >= popcount`. `k` in `0..BITS`.
#[must_use]
pub fn select_is_rank_inverse<W: Word>(x: W, k: u32) -> bool {
    x.select(k).map_or_else(
        || k >= x.count_ones(),
        |p| k < x.count_ones() && x.bit(p) && x.rank(p) == k,
    )
}

/// `select` agrees with the per-bit definition. `k` in `0..BITS`.
#[must_use]
pub fn select_matches_reference<W: Word>(x: W, k: u32) -> bool {
    x.select(k) == reference::select(x, k)
}

// --- compact / expand -------------------------------------------------

/// `compact` agrees with the per-bit definition.
#[must_use]
pub fn compact_matches_reference<W: Word>(x: W, m: W) -> bool {
    x.compact(m) == reference::pext(x, m)
}

/// `expand` agrees with the per-bit definition.
#[must_use]
pub fn expand_matches_reference<W: Word>(x: W, m: W) -> bool {
    x.expand(m) == reference::pdep(x, m)
}

/// `expand ∘ compact` under the same mask keeps exactly the masked bits.
#[must_use]
pub fn compact_expand_roundtrip<W: Word>(x: W, m: W) -> bool {
    x.compact(m).expand(m) == x.and(m)
}

/// `compact ∘ expand` under the same mask keeps exactly the low
/// `count_ones(m)` bits.
#[must_use]
pub fn expand_compact_roundtrip<W: Word>(x: W, m: W) -> bool {
    x.expand(m).compact(m) == x.and(W::low_ones(m.count_ones()))
}

/// `compact` preserves the count of selected bits.
#[must_use]
pub fn compact_preserves_popcount<W: Word>(x: W, m: W) -> bool {
    x.compact(m).count_ones() == x.and(m).count_ones()
}

/// Composition: extracting under `n` from an extract under `m` is one
/// extract under `n` deposited into `m`. Folklore (doc 13 §9.1).
#[must_use]
pub fn compact_composes<W: Word>(x: W, m: W, n: W) -> bool {
    x.compact(m).compact(n) == x.compact(n.expand(m))
}

// --- dilated / Morton -------------------------------------------------

/// Dilating then un-dilating is the identity on values that fit.
#[must_use]
pub fn dilated_roundtrip<W: Word, const D: u32>(x: W) -> bool {
    let x = x.and(W::low_ones(Dilated::<W, D>::width()));
    Dilated::<W, D>::from_int(x).into_int() == x
}

/// `incr` in dilated space is `+ 1` modulo the dilated width.
#[must_use]
pub fn dilated_incr_is_add_one<W: Word, const D: u32>(x: W) -> bool {
    let width = W::low_ones(Dilated::<W, D>::width());
    let x = x.and(width);
    Dilated::<W, D>::from_int(x).incr().into_int() == x.wrapping_add(W::ONE).and(width)
}

/// `decr` undoes `incr`.
#[must_use]
pub fn dilated_decr_after_incr_is_identity<W: Word, const D: u32>(x: W) -> bool {
    let d = Dilated::<W, D>::from_int(x.and(W::low_ones(Dilated::<W, D>::width())));
    d.incr().decr() == d
}

/// `wrapping_add` in dilated space is `+` modulo the dilated width.
#[must_use]
pub fn dilated_add_is_add<W: Word, const D: u32>(a: W, b: W) -> bool {
    let width = W::low_ones(Dilated::<W, D>::width());
    let (a, b) = (a.and(width), b.and(width));
    let (da, db) = (Dilated::<W, D>::from_int(a), Dilated::<W, D>::from_int(b));
    da.wrapping_add(db).into_int() == a.wrapping_add(b).and(width)
}

/// Encoding then decoding a Morton code returns the coordinates.
#[must_use]
pub fn morton_roundtrip<W: Word>(x: W, y: W) -> bool {
    let half = W::low_ones(W::BITS / 2);
    let (x, y) = (x.and(half), y.and(half));
    Morton2::encode(x, y).decode() == (x, y)
}

/// `step_x` / `step_y` move one cell along the axis, wrapping.
#[must_use]
pub fn morton_steps_are_unit_moves<W: Word>(x: W, y: W) -> bool {
    let half = W::low_ones(W::BITS / 2);
    let (x, y) = (x.and(half), y.and(half));
    let m = Morton2::encode(x, y);
    m.step_x() == Morton2::encode(x.wrapping_add(W::ONE).and(half), y)
        && m.step_y() == Morton2::encode(x, y.wrapping_add(W::ONE).and(half))
}

/// Aligned `2×2` blocks are contiguous code ranges: the block at even
/// `(x, y)` occupies `code..code + 4`.
#[must_use]
pub fn morton_aligned_block_is_contiguous<W: Word>(x: W, y: W) -> bool {
    let half = W::low_ones(W::BITS / 2);
    let even = half.and(W::ONE.not());
    let (x, y) = (x.and(even), y.and(even));
    let base = Morton2::encode(x, y).code();
    let cell = |dx: W, dy: W| Morton2::encode(x.or(dx), y.or(dy)).code();
    cell(W::ZERO, W::ZERO) == base
        && cell(W::ONE, W::ZERO) == base.or(W::ONE)
        && cell(W::ZERO, W::ONE) == base.or(W::ONE.shl(1))
        && cell(W::ONE, W::ONE) == base.or(W::low_ones(2))
}

/// Bit-loop reference semantics. Slow, obviously correct, the thing
/// every combinator is measured against.
pub mod reference {
    use crate::affine::Affine8;
    use crate::word::Word;

    /// Bit `p` set iff bits `p..p + k` of `x` are all set.
    #[must_use]
    pub fn run_starts<W: Word>(x: W, k: u32) -> W {
        let mut out = W::ZERO;
        for p in 0..W::BITS {
            let inside = p + k <= W::BITS;
            if inside && (p..p + k).all(|b| x.bit(b)) {
                out = out.or(W::ONE.shl(p));
            }
        }
        out
    }

    /// Bit `i` = parity of bits `0..=i`.
    #[must_use]
    pub fn prefix_xor<W: Word>(x: W) -> W {
        let mut out = W::ZERO;
        let mut parity = false;
        for i in 0..W::BITS {
            parity ^= x.bit(i);
            if parity {
                out = out.or(W::ONE.shl(i));
            }
        }
        out
    }

    /// Every bit at or below the highest set bit.
    #[must_use]
    pub fn suffix_or<W: Word>(x: W) -> W {
        let mut out = W::ZERO;
        let mut seen = false;
        for i in (0..W::BITS).rev() {
            seen |= x.bit(i);
            if seen {
                out = out.or(W::ONE.shl(i));
            }
        }
        out
    }

    /// Smallest `y > x` with `count_ones(y) == count_ones(x)`, by search.
    #[must_use]
    pub fn next_same_popcount<W: Word>(x: W) -> Option<W> {
        if x.is_zero() {
            return None;
        }
        let n = x.count_ones();
        let mut y = x.wrapping_add(W::ONE);
        while !y.is_zero() {
            if y.count_ones() == n {
                return Some(y);
            }
            y = y.wrapping_add(W::ONE);
        }
        None
    }

    /// `a <= b` without an `Ord` bound: no borrow out of `b - a`.
    fn le<W: Word>(a: W, b: W) -> bool {
        let d = b.wrapping_sub(a);
        let borrow = b.not().and(a).or(b.not().or(a).and(d));
        !borrow.bit(W::BITS - 1)
    }

    /// Smallest power of two `>= x`, `1` for zero, `None` if it does not fit.
    #[must_use]
    pub fn round_up_pow2<W: Word>(x: W) -> Option<W> {
        let mut p = W::ONE;
        loop {
            if le(x, p) {
                return Some(p);
            }
            if p.bit(W::BITS - 1) {
                return None;
            }
            p = p.shl(1);
        }
    }

    /// Bit `i` = parity of bits `i..BITS` (Gray decode).
    #[must_use]
    pub fn prefix_xor_from_top<W: Word>(x: W) -> W {
        let mut out = W::ZERO;
        let mut parity = false;
        for i in (0..W::BITS).rev() {
            parity ^= x.bit(i);
            if parity {
                out = out.or(W::ONE.shl(i));
            }
        }
        out
    }

    /// Position of the `k`-th set bit, by counting.
    #[must_use]
    pub fn select<W: Word>(x: W, k: u32) -> Option<u32> {
        let mut seen = 0;
        for i in 0..W::BITS {
            if x.bit(i) {
                if seen == k {
                    return Some(i);
                }
                seen += 1;
            }
        }
        None
    }

    /// PEXT by walking every position of the word.
    #[must_use]
    pub fn pext<W: Word>(x: W, m: W) -> W {
        let mut out = W::ZERO;
        let mut k = 0;
        for i in 0..W::BITS {
            if m.bit(i) {
                if x.bit(i) {
                    out = out.or(W::ONE.shl(k));
                }
                k += 1;
            }
        }
        out
    }

    /// PDEP by walking every position of the word.
    #[must_use]
    pub fn pdep<W: Word>(x: W, m: W) -> W {
        let mut out = W::ZERO;
        let mut k = 0;
        for i in 0..W::BITS {
            if m.bit(i) {
                if x.bit(k) {
                    out = out.or(W::ONE.shl(i));
                }
                k += 1;
            }
        }
        out
    }

    /// `x ↦ A·x ⊕ b` on one byte, bit by bit: output bit `i` is the XOR
    /// over `j` of `row_i[j] & x[j]`, then bit `i` of the constant.
    #[must_use]
    pub fn affine(map: Affine8, x: u8) -> u8 {
        let mut out = 0u8;
        for i in 0..8 {
            let mut bit = map.add() >> i & 1;
            for j in 0..8 {
                bit ^= (map.row(i) >> j & 1) & (x >> j & 1);
            }
            out |= bit << i;
        }
        out
    }

    /// Bit `i` of the result is bit `4 a_i + 2 b_i + c_i` of `table`.
    #[must_use]
    pub fn ternary<W: Word>(a: W, b: W, c: W, table: u8) -> W {
        let mut out = W::ZERO;
        for i in 0..W::BITS {
            let k = u32::from(a.bit(i)) << 2 | u32::from(b.bit(i)) << 1 | u32::from(c.bit(i));
            if table >> k & 1 == 1 {
                out = out.or(W::ONE.shl(i));
            }
        }
        out
    }
}

// --- reduces ------------------------------------------------------------

/// `parity` is the low bit of the popcount; `trailing_ones` /
/// `leading_ones` count what the reference loop counts.
#[must_use]
pub fn small_reduces_match_reference<W: Word>(x: W) -> bool {
    let trailing = (0..W::BITS).take_while(|&i| x.bit(i)).count();
    let leading = (0..W::BITS).rev().take_while(|&i| x.bit(i)).count();
    x.parity() == (x.count_ones() % 2 == 1)
        && x.trailing_ones() as usize == trailing
        && x.leading_ones() as usize == leading
}

/// `positions()` enumerates exactly the set bits, ascending, and
/// `rev()` descending, with an exact `len()`.
#[must_use]
pub fn positions_enumerate_set_bits<W: Word>(x: W) -> bool {
    let mut fwd = x.positions();
    for i in 0..W::BITS {
        if x.bit(i) && fwd.next() != Some(i) {
            return false;
        }
    }
    let mut back = x.positions().rev();
    for i in (0..W::BITS).rev() {
        if x.bit(i) && back.next() != Some(i) {
            return false;
        }
    }
    fwd.next().is_none() && back.next().is_none() && x.positions().len() == x.count_ones() as usize
}

// --- swar -----------------------------------------------------------------

/// Every SWAR lane predicate agrees with the per-byte definition.
/// `n` in `1..=128`.
#[must_use]
pub fn swar_lanes_match_reference<W: Word>(x: W, b: u8, n: u8) -> bool {
    let lanes = W::BITS / 8;
    let mut zero = W::ZERO;
    let mut eq = W::ZERO;
    let mut ge = W::ZERO;
    let mut sum = 0u32;
    for lane in 0..lanes {
        let byte = x.shr(lane * 8).low_byte();
        let hi = W::ONE.shl(lane * 8 + 7);
        if byte == 0 {
            zero = zero.or(hi);
        }
        if byte == b {
            eq = eq.or(hi);
        }
        if byte >= n {
            ge = ge.or(hi);
        }
        sum += u32::from(byte);
    }
    x.zero_bytes() == zero
        && x.has_zero_byte() != zero.is_zero()
        && x.first_zero_byte() == zero.first_set().map(|p| p / 8)
        && x.bytes_eq(b) == eq
        && x.count_bytes_eq(b) == eq.count_ones()
        && x.bytes_ge(n) == ge
        && x.bytes_lt(n) == ge.xor(W::lanes_hi())
        && (sum >= 256 || x.sum_bytes() == sum)
}

// --- slice ---------------------------------------------------------------

/// Slice `rank`/`select`/`next_set_after`/`find_run` agree with the
/// bit-loop definitions over the concatenated words.
// `p % bits < BITS` fits a u32.
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn slice_ops_match_reference<W: Word>(words: &[W], i: usize, k: u32) -> bool {
    let bits = W::BITS as usize;
    let total = words.len() * bits;
    let bit = |p: usize| words[p / bits].bit((p % bits) as u32);
    let set: alloc_free::Positions = (0..total).filter(|&p| bit(p)).collect();

    let rank_ok = crate::slice::rank(words, i) == set.iter().filter(|&&p| p < i).count();
    let select_ok = crate::slice::select(words, i) == set.get(i).copied();
    let next_ok = crate::slice::next_set_after(words, i) == set.iter().copied().find(|&p| p >= i);
    let run_ref = (0..total).find(|&s| s + k as usize <= total && (s..s + k as usize).all(bit));
    let run_ok = crate::slice::find_run(words, k) == run_ref;
    let positions_ok = crate::slice::positions(words).eq(set.iter().copied());
    rank_ok
        && select_ok
        && next_ok
        && run_ok
        && positions_ok
        && crate::slice::popcount(words) == set.len()
}

/// Tiny fixed-capacity position list so the slice reference stays
/// `no_std` (slices in the laws are at most 4 words wide).
mod alloc_free {
    pub(super) struct Positions {
        buf: [usize; 512],
        len: usize,
    }
    impl Positions {
        pub(super) fn iter(&self) -> core::slice::Iter<'_, usize> {
            self.buf[..self.len].iter()
        }
        pub(super) fn get(&self, i: usize) -> Option<&usize> {
            self.buf[..self.len].get(i)
        }
        pub(super) const fn len(&self) -> usize {
            self.len
        }
    }
    impl FromIterator<usize> for Positions {
        fn from_iter<I: IntoIterator<Item = usize>>(it: I) -> Self {
            let mut p = Self {
                buf: [0; 512],
                len: 0,
            };
            for v in it {
                p.buf[p.len] = v;
                p.len += 1;
            }
            p
        }
    }
}

// --- permute --------------------------------------------------------------

/// `delta_swap` is an involution and moves exactly the masked bits.
/// `shift` in `1..BITS`; `mask` is sanitised to a valid one.
#[must_use]
pub fn delta_swap_is_involution<W: Word>(x: W, mask: W, shift: u32) -> bool {
    // Keep only bits whose partner is in range and not itself masked.
    let mask = mask.and(W::ONES.shr(shift)).and(mask.shl(shift).not());
    let y = x.delta_swap(mask, shift);
    let untouched = mask.or(mask.shl(shift)).not();
    y.delta_swap(mask, shift) == x
        && y.and(untouched) == x.and(untouched)
        && y.and(mask) == x.shr(shift).and(mask)
        && y.and(mask.shl(shift)) == x.and(mask).shl(shift)
}

/// The 8×8 board permutations are correct.
///
/// Each agrees with its coordinate definition on every cell (linearity
/// makes single-cell checks cover every input), and they compose as
/// the group says: `transpose² = id`, `rotate_90_cw⁴ = id`,
/// `rotate_90_ccw ∘ rotate_90_cw = id`, `rotate_180 = flip ∘ mirror`.
#[must_use]
pub fn board8_permutations_are_correct(x: u64) -> bool {
    use crate::permute::board8::{
        flip_vertical, mirror_horizontal, rotate_90_ccw, rotate_90_cw, rotate_180, transpose,
    };
    let cell = |r: u64, c: u64| 1u64 << (8 * r + c);
    let by_cells = |f: fn(u64) -> u64, map: fn(u64, u64) -> (u64, u64)| {
        // Linearity lets single-cell checks cover every input.
        let mut out = 0;
        for r in 0..8 {
            for c in 0..8 {
                if x & cell(r, c) != 0 {
                    let (nr, nc) = map(r, c);
                    out |= cell(nr, nc);
                }
            }
        }
        f(x) == out
    };
    by_cells(transpose, |r, c| (c, r))
        && by_cells(flip_vertical, |r, c| (7 - r, c))
        && by_cells(mirror_horizontal, |r, c| (r, 7 - c))
        && by_cells(rotate_90_cw, |r, c| (7 - c, r))
        && by_cells(rotate_90_ccw, |r, c| (c, 7 - r))
        && by_cells(rotate_180, |r, c| (7 - r, 7 - c))
        && transpose(transpose(x)) == x
        && rotate_90_cw(rotate_90_cw(rotate_90_cw(rotate_90_cw(x)))) == x
        && rotate_90_ccw(rotate_90_cw(x)) == x
        && rotate_180(x) == flip_vertical(mirror_horizontal(x))
}

// --- fill -----------------------------------------------------------------

/// Kogge–Stone fills agree with iterated single steps, and with a full
/// propagation mask and stride 1 they are the prefix / suffix OR.
/// `stride` in `1..BITS`.
#[must_use]
pub fn fills_match_reference<W: Word>(x: W, propagate: W, stride: u32) -> bool {
    let mut up = x;
    let mut down = x;
    for _ in 0..W::BITS {
        up = up.or(propagate.and(up.shl(stride)));
        down = down.or(propagate.and(down.shr(stride)));
    }
    x.fill_up(propagate, stride) == up
        && x.fill_down(propagate, stride) == down
        && x.fill_up(W::ONES, 1) == x.prefix_or()
        && x.fill_down(W::ONES, 1) == reference::suffix_or(x)
}

// --- basics ---------------------------------------------------------------

/// The lowest-bit family agrees with the position-based definitions.
#[must_use]
pub fn basics_match_reference<W: Word>(x: W, y: W, m: W) -> bool {
    let low = x.first_set();
    let lowest_mask = low.map_or(W::ZERO, |p| W::ONE.shl(p));
    let below = low.map_or(W::ONES, |p| W::low_ones(p));
    let up_to = low.map_or(W::ONES, |p| W::low_ones(p + 1));
    let lowest_zero = x.not().first_set().map_or(W::ZERO, |p| W::ONE.shl(p));
    let run_len = low.map_or(0, |p| x.shr(p).trailing_ones());
    let run_mask = low.map_or(W::ZERO, |p| W::low_ones(run_len).shl(p));
    let mut blend = W::ZERO;
    for i in 0..W::BITS {
        if if m.bit(i) { y.bit(i) } else { x.bit(i) } {
            blend = blend.or(W::ONE.shl(i));
        }
    }
    x.lowest_set_mask() == lowest_mask
        && x.below_lowest_set() == below
        && x.up_to_lowest_set() == up_to
        && x.set_lowest_zero() == x.or(lowest_zero)
        && x.clear_lowest_run() == x.and(run_mask.not())
        && x.blend(y, m) == blend
}

/// Gosper's hack yields the smallest larger word with the same
/// popcount, and `None` exactly when none exists.
#[must_use]
pub fn next_same_popcount_matches_reference<W: Word>(x: W) -> bool {
    x.next_same_popcount() == reference::next_same_popcount(x)
}

/// Power-of-two helpers agree with their definitions.
#[must_use]
pub fn pow2_helpers_match_reference<W: Word>(x: W) -> bool {
    let is_pow2 = x.count_ones() == 1;
    let down = x.last_set().map(|p| W::ONE.shl(p));
    let up = reference::round_up_pow2(x);
    let floor = x.last_set();
    let ceil = if x.is_zero() {
        None
    } else if is_pow2 {
        floor
    } else {
        floor.map(|p| p + 1)
    };
    x.is_pow2() == is_pow2
        && x.round_down_pow2() == down
        && x.round_up_pow2() == up
        && x.log2_floor() == floor
        && x.log2_ceil() == ceil
}

// --- runs (continued) ------------------------------------------------------

/// `longest_run` is the largest `k` with a run, and `0` iff zero.
#[must_use]
pub fn longest_run_matches_reference<W: Word>(x: W) -> bool {
    let (mut best, mut cur) = (0, 0);
    for i in 0..W::BITS {
        cur = if x.bit(i) { cur + 1 } else { 0 };
        best = best.max(cur);
    }
    x.longest_run() == best
}

// --- gray / escapes ---------------------------------------------------------

/// Gray code: decode undoes encode, consecutive codes differ in one
/// bit, and decode is the top-down prefix XOR.
#[must_use]
pub fn gray_code_laws<W: Word>(x: W) -> bool {
    let next = x.wrapping_add(W::ONE);
    x.gray_encode().gray_decode() == x
        && x.gray_decode().gray_encode() == x
        && x.gray_encode().xor(next.gray_encode()).count_ones() == 1
        && x.gray_decode() == reference::prefix_xor_from_top(x)
}

/// `find_escaped` over a chain of words agrees with a bit-by-bit
/// parity walk over the concatenation, carry included.
#[must_use]
pub fn find_escaped_matches_reference<W: Word>(words: &[W], prev_ends_odd: bool) -> bool {
    let mut carry = prev_ends_odd;
    let mut run_odd = prev_ends_odd;
    for &w in words {
        let (escaped, next_carry) = w.find_escaped(carry);
        let mut want = W::ZERO;
        for i in 0..W::BITS {
            if run_odd && !w.bit(i) {
                want = want.or(W::ONE.shl(i));
            }
            run_odd = if w.bit(i) { !run_odd } else { false };
        }
        if escaped != want || next_carry != run_odd {
            return false;
        }
        carry = next_carry;
    }
    true
}

// --- algebra: composition and homomorphism laws ------------------------------
//
// The first hand-collected "identity family" (doc 13 §9.2, layer 3):
// how combinators compose with themselves and with the Boolean ring.
// These are the seeds a rewrite engine starts from.

/// Run detection composes additively: detecting `b`-runs among the
/// starts of `a`-runs is detecting `(a + b − 1)`-runs. This is *why*
/// the halving chain works. Requires `a + b − 1 <= BITS`.
#[must_use]
pub fn run_starts_composes<W: Word>(x: W, a: u32, b: u32) -> bool {
    x.run_starts(a).run_starts(b) == x.run_starts(a + b - 1)
}

/// `run_starts` preserves meets: a window is all-set in `x & y` iff it
/// is all-set in both. (It is monotone, not linear.)
#[must_use]
pub fn run_starts_preserves_and<W: Word>(x: W, y: W, k: u32) -> bool {
    x.and(y).run_starts(k) == x.run_starts(k).and(y.run_starts(k))
}

/// Fills are idempotent and extensive (contain their generator).
#[must_use]
pub fn fills_are_closure_operators<W: Word>(x: W, p: W, s: u32) -> bool {
    let up = x.fill_up(p, s);
    let down = x.fill_down(p, s);
    up.fill_up(p, s) == up && down.fill_down(p, s) == down && up.and(x) == x && down.and(x) == x
}

/// The XOR-linear combinators.
///
/// `f(x ^ y) = f(x) ^ f(y)` for prefix scans, Gray code, compact/expand
/// under a fixed mask and delta swaps. Linearity is what lets
/// single-bit checks prove a permutation.
#[must_use]
pub fn xor_linear_combinators<W: Word>(x: W, y: W, m: W, shift: u32) -> bool {
    let z = x.xor(y);
    let mask = m.and(W::ONES.shr(shift)).and(m.shl(shift).not());
    z.prefix_xor() == x.prefix_xor().xor(y.prefix_xor())
        && z.gray_encode() == x.gray_encode().xor(y.gray_encode())
        && z.gray_decode() == x.gray_decode().xor(y.gray_decode())
        && z.compact(m) == x.compact(m).xor(y.compact(m))
        && z.expand(m) == x.expand(m).xor(y.expand(m))
        && z.delta_swap(mask, shift) == x.delta_swap(mask, shift).xor(y.delta_swap(mask, shift))
}

/// The OR-to-AND (De Morgan-shaped) combinators: a lane is zero in
/// `x | y` iff zero in both; a window is all-set in `x & y` iff in both.
#[must_use]
pub fn zero_bytes_turns_or_into_and<W: Word>(x: W, y: W) -> bool {
    x.or(y).zero_bytes() == x.zero_bytes().and(y.zero_bytes())
}

/// `expand` composes dually to `compact` (doc 13 §9.1 folklore,
/// mirrored): depositing into `n` then into `m` is depositing into
/// `pdep(n, m)`.
#[must_use]
pub fn expand_composes<W: Word>(x: W, m: W, n: W) -> bool {
    x.expand(n).expand(m) == x.expand(n.expand(m))
}

/// Delta swaps with a common shift and pairwise disjoint masks (and
/// images) compose to one delta swap with the union mask. Inputs that
/// cannot be made disjoint are skipped (vacuously true).
#[must_use]
pub fn delta_swaps_merge<W: Word>(x: W, mask1: W, mask2: W, shift: u32) -> bool {
    let fit = W::ONES.shr(shift);
    let mask1 = mask1.and(fit).and(mask1.shl(shift).not());
    let taken = mask1.or(mask1.shl(shift));
    let mask2 = mask2.and(fit).and(mask2.shl(shift).not()).and(taken.not());
    if !mask2.shl(shift).and(taken).is_zero() {
        return true;
    }
    x.delta_swap(mask1, shift).delta_swap(mask2, shift) == x.delta_swap(mask1.or(mask2), shift)
}

/// Gray code successor: consecutive codes differ exactly in the
/// lowest set bit of the successor. `x != ONES`.
#[must_use]
pub fn gray_successor_flips_lowest_set<W: Word>(x: W) -> bool {
    let next = x.wrapping_add(W::ONE);
    x.gray_encode().xor(next.gray_encode()) == next.lowest_set_mask()
}

/// Rank and select are inverse on set bits: `select(rank(i)) = i`
/// whenever bit `i` is set. `i < BITS`.
#[must_use]
pub fn select_inverts_rank_on_set_bits<W: Word>(x: W, i: u32) -> bool {
    !x.bit(i) || x.select(x.rank(i)) == Some(i)
}

// --- grid -----------------------------------------------------------------

/// `block_starts` agrees with the per-cell definition, reduces to
/// `run_starts` for `h = 1`, and composes additively in both axes.
/// `rows` at most 8 long; `w`, `h` such that the sums stay in range.
#[must_use]
pub fn block_starts_laws<W: Word>(rows: &[W], w1: u32, h1: u32, w2: u32, h2: u32) -> bool {
    let len = rows.len();
    let mut out = [W::ZERO; 8];
    let out = &mut out[..len];
    crate::grid::block_starts(rows, w1, h1, out);

    // Per-cell reference.
    for (r, &row_out) in out.iter().enumerate() {
        for c in 0..W::BITS {
            let fits = r + h1 as usize <= len && c + w1 <= W::BITS;
            let all = fits && (r..r + h1 as usize).all(|rr| (c..c + w1).all(|cc| rows[rr].bit(cc)));
            if row_out.bit(c) != all {
                return false;
            }
        }
    }

    // h = 1 is the 1D combinator.
    let mut one = [W::ZERO; 8];
    let one = &mut one[..len];
    crate::grid::block_starts(rows, w1, 1, one);
    if one.iter().zip(rows).any(|(&o, &r)| o != r.run_starts(w1)) {
        return false;
    }

    // Composition.
    let mut twice = [W::ZERO; 8];
    let twice = &mut twice[..len];
    crate::grid::block_starts(out, w2, h2, twice);
    let mut direct = [W::ZERO; 8];
    let direct = &mut direct[..len];
    crate::grid::block_starts(rows, w1 + w2 - 1, h1 + h2 - 1, direct);
    twice == direct
}

// --- rank9 ------------------------------------------------------------

/// The directory agrees with the linear scans of [`crate::slice`] it
/// indexes: `rank` and `rank0` at `i`, `select` at `k`, and the total.
#[must_use]
pub fn rank9_matches_slice(dir: &Rank9<'_>, i: usize, k: usize) -> bool {
    let bits = dir.bits();
    let rank = crate::slice::rank(bits, i);
    dir.rank(i) == rank
        && dir.rank0(i) == i.min(dir.len()) - rank
        && dir.select(k) == crate::slice::select(bits, k)
        && dir.count_ones() == crate::slice::popcount(bits)
}

/// `select` inverts `rank` on set bits: for `k < count_ones()`,
/// `rank(select(k)) == k` and the bit at `select(k)` is set; beyond
/// that `select` is `None`.
#[must_use]
pub fn rank9_select_inverts_rank(dir: &Rank9<'_>, k: usize) -> bool {
    dir.select(k).map_or_else(
        || k >= dir.count_ones(),
        |p| dir.rank(p) == k && dir.rank(p + 1) == k + 1,
    )
}

/// `rank` is monotone and steps by exactly the bit at `i`.
#[must_use]
pub fn rank9_rank_steps_by_bit(dir: &Rank9<'_>, i: usize) -> bool {
    // `i % 64 < 64` fits a u32.
    #[allow(clippy::cast_possible_truncation)]
    let bit = i < dir.len() && dir.bits()[i / 64].bit((i % 64) as u32);
    dir.rank(i + 1) == dir.rank(i) + usize::from(bit)
}

// --- board8 -----------------------------------------------------------

/// [`board8::slide`] equals a walk from
/// every piece, one square at a time in `dir`, stopping on the first
/// occupied square (included) or at the edge.
#[must_use]
pub fn board8_slides_match_reference(pieces: u64, empty: u64, dir: Dir) -> bool {
    let (dr, dc): (i32, i32) = match dir {
        Dir::North => (1, 0),
        Dir::South => (-1, 0),
        Dir::East => (0, 1),
        Dir::West => (0, -1),
        Dir::NorthEast => (1, 1),
        Dir::NorthWest => (1, -1),
        Dir::SouthEast => (-1, 1),
        Dir::SouthWest => (-1, -1),
    };
    let mut expect = 0u64;
    for sq in 0..64u32 {
        if pieces >> sq & 1 == 0 {
            continue;
        }
        let (mut r, mut c) = ((sq / 8).cast_signed(), (sq % 8).cast_signed());
        loop {
            r += dr;
            c += dc;
            if !(0..8).contains(&r) || !(0..8).contains(&c) {
                break;
            }
            // `r` and `c` are in `0..8`.
            #[allow(clippy::cast_sign_loss)]
            let bit = 1u64 << (8 * r + c) as u32;
            expect |= bit;
            if empty & bit == 0 {
                break;
            }
        }
    }
    board8::slide(pieces, empty, dir) == expect
}

// --- lanes ------------------------------------------------------------

/// Every [`Lanes`] operation against its per-lane scalar definition.
///
/// On `lhs` and `rhs` with shift `n` and lookup `table`; `rhs` doubles as
/// the index vector of `shuffle` and the weights of `mul_add_pairs`, and
/// `concat_shift` is checked for every offset.
#[must_use]
pub fn lanes_match_reference<L: Lanes>(lhs: L, rhs: L, n: u32, table: [u8; 16]) -> bool {
    let lanes = L::LANES;
    let mask = |c: bool| if c { 0xFF } else { 0 };
    let lut = |v: u8| {
        if v & 0x80 == 0 {
            table[usize::from(v & 15)]
        } else {
            0
        }
    };
    let bits = lhs.to_bits();
    let lanewise = (0..lanes).all(|i| {
        let (x, y) = (lhs.lane(i), rhs.lane(i));
        let lane_bit = u32::try_from(i).is_ok_and(|i| bits.bit(i));
        lhs.and(rhs).lane(i) == x & y
            && lhs.or(rhs).lane(i) == x | y
            && lhs.xor(rhs).lane(i) == x ^ y
            && lhs.not().lane(i) == !x
            && lhs.add(rhs).lane(i) == x.wrapping_add(y)
            && lhs.sub(rhs).lane(i) == x.wrapping_sub(y)
            && lhs.shl(n).lane(i) == if n >= 8 { 0 } else { x << n }
            && lhs.shr(n).lane(i) == if n >= 8 { 0 } else { x >> n }
            && lhs.cmp_eq(rhs).lane(i) == mask(x == y)
            && lhs.cmp_le(rhs).lane(i) == mask(x <= y)
            && lhs.cmp_lt(rhs).lane(i) == mask(x < y)
            && lhs.cmp_ge(rhs).lane(i) == mask(x >= y)
            && lhs.cmp_gt(rhs).lane(i) == mask(x > y)
            && lhs.min(rhs).lane(i) == x.min(y)
            && lhs.max(rhs).lane(i) == x.max(y)
            && lhs.blend(rhs, lhs.cmp_le(rhs)).lane(i) == if x <= y { y } else { x }
            && lhs.lut16(table).lane(i) == lut(x)
            && lane_bit == (x & 0x80 != 0)
            && lhs.add_sat(rhs).lane(i) == x.saturating_add(y)
            && lhs.sub_sat(rhs).lane(i) == x.saturating_sub(y)
            && lhs.shuffle(rhs).lane(i)
                == if y & 0x80 == 0 {
                    lhs.lane(usize::from(y) % lanes)
                } else {
                    0
                }
    });
    let concat = (0..=2 * lanes + 1).all(|k| {
        let r = lhs.concat_shift(rhs, k);
        (0..lanes).all(|i| {
            let j = i + k;
            r.lane(i)
                == if j < lanes {
                    lhs.lane(j)
                } else if j < 2 * lanes {
                    rhs.lane(j - lanes)
                } else {
                    0
                }
        })
    });
    let (lo, hi) = (lhs.unpack_lo(rhs), lhs.unpack_hi(rhs));
    let unpack = (0..lanes / 2).all(|k| {
        lo.lane(2 * k) == lhs.lane(k)
            && lo.lane(2 * k + 1) == rhs.lane(k)
            && hi.lane(2 * k) == lhs.lane(lanes / 2 + k)
            && hi.lane(2 * k + 1) == rhs.lane(lanes / 2 + k)
    });
    let sad = lhs.sum_abs_diff(rhs)
        == (0..lanes)
            .map(|i| u32::from(lhs.lane(i).abs_diff(rhs.lane(i))))
            .sum::<u32>();
    let products = lhs.mul_add_pairs(rhs);
    let madd = (0..lanes / 2).all(|k| {
        let term = |j: usize| i32::from(lhs.lane(j)) * i32::from(rhs.lane(j).cast_signed());
        let sum = (term(2 * k) + term(2 * k + 1)).clamp(i32::from(i16::MIN), i32::from(i16::MAX));
        // Clamped into range, so the narrowing is exact.
        #[allow(clippy::cast_possible_truncation)]
        let bytes = (sum as i16).to_le_bytes();
        products.lane(2 * k) == bytes[0] && products.lane(2 * k + 1) == bytes[1]
    });
    lanewise
        && concat
        && unpack
        && sad
        && madd
        && L::splat(0x5A).lane(lanes - 1) == 0x5A
        && L::zero() == L::splat(0)
        && lane_maps_match_reference(lhs, rhs, n, table)
}

/// Two table lookups are one: on lanes with the top bit clear,
/// `lut16(lut16(x, a), b) == lut16(x, b ∘ a)`, the composed table
/// applying PSHUFB's zero-on-top-bit rule to `a`'s entries.
#[must_use]
pub fn lut16_composes<L: Lanes>(x: L, a: [u8; 16], b: [u8; 16]) -> bool {
    let x = x.and(L::splat(0x7F));
    let mut composed = [0u8; 16];
    for (c, &v) in composed.iter_mut().zip(a.iter()) {
        *c = if v & 0x80 == 0 {
            b[usize::from(v & 15)]
        } else {
            0
        };
    }
    x.lut16(a).lut16(b) == x.lut16(composed)
}

/// The lane algebra and the word algebra agree where they meet: a
/// compare folded to bits is the SWAR byte test of [`Bits`], and the
/// count of matching lanes is [`Bits::count_bytes_eq`].
#[must_use]
pub fn lanes_agree_with_bits(x: u64, b: u8) -> bool {
    let lanes = U8x8::new(x);
    let eq = lanes.cmp_eq(U8x8::splat(b)).to_bits();
    eq.count_ones() == x.count_bytes_eq(b)
        && lanes.cmp_eq(U8x8::zero()).to_bits() == U8x8::new(x.zero_bytes()).to_bits()
        && lanes.cmp_lt(U8x8::splat(0x80)).to_bits() == U8x8::new(x.bytes_lt(0x80)).to_bits()
}

/// The sixteen-lane carrier, whatever it compiles to, is two eight-lane
/// SWAR carriers side by side.
#[must_use]
pub fn u8x16_agrees_with_halves(lhs: (u64, u64), rhs: (u64, u64), n: u32, table: [u8; 16]) -> bool {
    let wide = |p: (u64, u64)| U8x16::from_halves(p.0, p.1);
    let narrow = |p: (u64, u64)| (U8x8::new(p.0), U8x8::new(p.1));
    let (x, y) = (wide(lhs), wide(rhs));
    let ((xl, xh), (yl, yh)) = (narrow(lhs), narrow(rhs));
    let pair = |l: U8x8, h: U8x8| (l.bits(), h.bits());
    let mut matrix = [0u8; 8];
    matrix.copy_from_slice(&table[..8]);
    let map = Affine8::new(u64::from_le_bytes(matrix), table[8]);
    x.and(y).halves() == pair(xl.and(yl), xh.and(yh))
        && x.or(y).halves() == pair(xl.or(yl), xh.or(yh))
        && x.xor(y).halves() == pair(xl.xor(yl), xh.xor(yh))
        && x.not().halves() == pair(xl.not(), xh.not())
        && x.add(y).halves() == pair(xl.add(yl), xh.add(yh))
        && x.sub(y).halves() == pair(xl.sub(yl), xh.sub(yh))
        && x.shl(n).halves() == pair(xl.shl(n), xh.shl(n))
        && x.shr(n).halves() == pair(xl.shr(n), xh.shr(n))
        && x.cmp_eq(y).halves() == pair(xl.cmp_eq(yl), xh.cmp_eq(yh))
        && x.cmp_le(y).halves() == pair(xl.cmp_le(yl), xh.cmp_le(yh))
        && x.lut16(table).halves() == pair(xl.lut16(table), xh.lut16(table))
        && x.to_bits() == u16::from(xl.to_bits()) | u16::from(xh.to_bits()) << 8
        && x.add_sat(y).halves() == pair(xl.add_sat(yl), xh.add_sat(yh))
        && x.sub_sat(y).halves() == pair(xl.sub_sat(yl), xh.sub_sat(yh))
        && x.mul_add_pairs(y).halves() == pair(xl.mul_add_pairs(yl), xh.mul_add_pairs(yh))
        && x.sum_abs_diff(y) == xl.sum_abs_diff(yl) + xh.sum_abs_diff(yh)
        && x.affine(map).halves() == pair(xl.affine(map), xh.affine(map))
        && x.reverse_bits().halves() == pair(xl.reverse_bits(), xh.reverse_bits())
        && x.sra(n).halves() == pair(xl.sra(n), xh.sra(n))
        && x.rotl(n).halves() == pair(xl.rotl(n), xh.rotl(n))
        && x.avg_round(y).halves() == pair(xl.avg_round(yl), xh.avg_round(yh))
        && x.avg_floor(y).halves() == pair(xl.avg_floor(yl), xh.avg_floor(yh))
        && x.ternary(y, x.not(), table[9]).halves()
            == pair(
                xl.ternary(yl, xl.not(), table[9]),
                xh.ternary(yh, xh.not(), table[9]),
            )
        && x.lane(3) == xl.lane(3)
        && x.lane(11) == xh.lane(3)
}

// --- ternary and sign-bit tests ---------------------------------------

/// [`Bits::ternary`] agrees with the bit-by-bit reading of the table.
#[must_use]
pub fn ternary_is_truth_table<W: Word>(a: W, b: W, c: W, table: u8) -> bool {
    a.ternary(b, c, table) == reference::ternary(a, b, c, table)
}

/// The table of a function is the function at `(0xF0, 0xCC, 0xAA)`.
///
/// For a handful of named functions, `ternary` with that table is the
/// function, and the overflow tests of Hacker's Delight 2-13 come out
/// as the immediates `0x42` and `0x18`.
#[must_use]
pub fn truth_table_names_the_function<W: Word>(a: W, b: W, c: W) -> bool {
    fn mux<W: Word>(m: W, x: W, y: W) -> W {
        m.and(x).or(m.not().and(y))
    }
    fn majority<W: Word>(a: W, b: W, c: W) -> W {
        a.and(b).or(a.and(c)).or(b.and(c))
    }
    fn xor3<W: Word>(a: W, b: W, c: W) -> W {
        a.xor(b).xor(c)
    }
    fn add_overflow<W: Word>(a: W, b: W, s: W) -> W {
        a.xor(b).not().and(a.xor(s))
    }
    fn sub_overflow<W: Word>(a: W, b: W, d: W) -> W {
        a.xor(b).and(a.xor(d))
    }
    a.ternary(b, c, truth_table(mux::<u8>)) == mux(a, b, c)
        && a.ternary(b, c, truth_table(majority::<u8>)) == majority(a, b, c)
        && a.ternary(b, c, truth_table(xor3::<u8>)) == xor3(a, b, c)
        && truth_table(add_overflow::<u8>) == 0x42
        && truth_table(sub_overflow::<u8>) == 0x18
        && a.ternary(b, c, 0x42) == add_overflow(a, b, c)
        && a.ternary(b, c, 0x18) == sub_overflow(a, b, c)
}

/// Signed overflow from three sign bits agrees with the sign comparison.
///
/// For addition the operands agree in sign and the sum does not; for
/// subtraction they differ and the difference disagrees with the
/// minuend; and both tests are the ternary immediates `0x42` and `0x18`.
#[must_use]
pub fn signed_overflow_matches_sign_test<W: Word>(a: W, b: W) -> bool {
    let top = |w: W| w.bit(W::BITS - 1);
    let (sum, diff) = (a.wrapping_add(b), a.wrapping_sub(b));
    a.signed_add_overflows(b) == (top(a) == top(b) && top(sum) != top(a))
        && a.signed_sub_overflows(b) == (top(a) != top(b) && top(diff) != top(a))
        && a.signed_add_overflows(b) == top(a.ternary(b, sum, 0x42))
        && a.signed_sub_overflows(b) == top(a.ternary(b, diff, 0x18))
}

// --- affine -----------------------------------------------------------

/// An [`Affine8`] map agrees with its bit-by-bit definition: on one
/// byte (`apply`), on every byte of a word (`apply8`), through the
/// nibble tables, and on every lane of the SWAR carrier.
#[must_use]
pub fn affine_matches_reference(map: Affine8, word: u64) -> bool {
    let (lo, hi) = map.tables();
    let by_word = map.apply8(word).to_le_bytes();
    let by_lanes = U8x8::new(word).affine(map);
    word.to_le_bytes().iter().enumerate().all(|(i, &x)| {
        let want = reference::affine(map, x);
        map.apply(x) == want
            && by_word[i] == want
            && by_lanes.lane(i) == want
            && lo[usize::from(x & 15)] ^ hi[usize::from(x >> 4)] == want
    })
}

/// Composition is matrix multiplication: `a.then(b)` applied is `b`
/// after `a`, `compose` is the same map written the other way round,
/// and the identity is neutral on both sides.
#[must_use]
pub fn affine_composes(a: Affine8, b: Affine8, x: u8) -> bool {
    let splat = u64::from(x) * 0x0101_0101_0101_0101;
    a.then(b).apply(x) == b.apply(a.apply(x))
        && b.compose(a) == a.then(b)
        && Affine8::IDENTITY.then(a) == a
        && a.then(Affine8::IDENTITY) == a
        && a.then(b).apply8(splat) == b.apply8(a.apply8(splat))
}

/// The named maps are the byte operations they are named after, for
/// every shift count including those past the width.
#[must_use]
pub fn affine_named_maps_match_ops(x: u8, n: u32) -> bool {
    let shl = if n >= 8 { 0 } else { x << n };
    let shr = if n >= 8 { 0 } else { x >> n };
    let sra = (x.cast_signed() >> n.min(7)).cast_unsigned();
    let parity = if x.count_ones() & 1 == 1 { 0xFF } else { 0 };
    Affine8::IDENTITY.apply(x) == x
        && Affine8::NOT.apply(x) == !x
        && Affine8::ZERO.apply(x) == 0
        && Affine8::REVERSE.apply(x) == x.reverse_bits()
        && Affine8::PARITY.apply(x) == parity
        && Affine8::shl(n).apply(x) == shl
        && Affine8::shr(n).apply(x) == shr
        && Affine8::sra(n).apply(x) == sra
        && Affine8::rotl(n).apply(x) == x.rotate_left(n)
        && Affine8::rotr(n).apply(x) == x.rotate_right(n)
        && Affine8::from_rows([1, 2, 4, 8, 16, 32, 64, 128], 0) == Affine8::IDENTITY
        && (0..8).all(|i| Affine8::IDENTITY.row(i) == 1 << i)
}

/// The byte maps of [`Lanes`] against their scalar definitions, on
/// `lhs` and `rhs` with shift `n`; the map under test is built from
/// `table`, and so is the ternary truth table.
#[must_use]
pub fn lane_maps_match_reference<L: Lanes>(lhs: L, rhs: L, n: u32, table: [u8; 16]) -> bool {
    let mut matrix = [0u8; 8];
    matrix.copy_from_slice(&table[..8]);
    let map = Affine8::new(u64::from_le_bytes(matrix), table[8]);
    let truth = table[9];
    let third = lhs.add(rhs);
    (0..L::LANES).all(|i| {
        let (x, y, z) = (lhs.lane(i), rhs.lane(i), third.lane(i));
        lhs.affine(map).lane(i) == map.apply(x)
            && lhs.reverse_bits().lane(i) == x.reverse_bits()
            && lhs.sra(n).lane(i) == (x.cast_signed() >> n.min(7)).cast_unsigned()
            && lhs.rotl(n).lane(i) == x.rotate_left(n)
            && lhs.rotr(n).lane(i) == x.rotate_right(n)
            && u16::from(lhs.avg_round(rhs).lane(i)) == (u16::from(x) + u16::from(y) + 1) >> 1
            && u16::from(lhs.avg_floor(rhs).lane(i)) == u16::midpoint(u16::from(x), u16::from(y))
            && lhs.ternary(rhs, third, truth).lane(i) == x.ternary(y, z, truth)
    }) && L::zero().avg_round(L::zero().not()) == L::splat(0x80)
}

// --- carry-rippler and gather ------------------------------------------

/// The carry-rippler is `+ 1` in the compacted domain: the next subset
/// of `mask` after `x` is `expand(compact(x) + 1)`, and `None` exactly
/// after the mask itself. Bits of `x` outside the mask are ignored.
#[must_use]
pub fn next_subset_is_increment_in_mask<W: Word>(x: W, mask: W) -> bool {
    let s = x.and(mask);
    let want = if s == mask {
        None
    } else {
        Some(s.compact(mask).wrapping_add(W::ONE).expand(mask))
    };
    s.next_subset(mask) == want && x.next_subset(mask) == want
}

/// [`Bits::subsets`] enumerates every subset of `mask` exactly once,
/// consecutive in the compacted domain, from zero to the mask. `mask`
/// with at most 16 set bits;
/// wider masks pass vacuously.
#[must_use]
pub fn subsets_enumerate_each_once<W: Word>(mask: W) -> bool {
    let k = mask.count_ones();
    if k > 16 {
        return true;
    }
    let mut count = 0u32;
    let mut previous = None;
    for s in mask.subsets() {
        count += 1;
        let consecutive =
            previous.is_none_or(|p: W| p.compact(mask).wrapping_add(W::ONE) == s.compact(mask));
        if !consecutive || s.and(mask) != s {
            return false;
        }
        previous = Some(s);
    }
    count == 1 << k && previous == Some(mask)
}

/// A gather triple with placement `place` agrees with [`Bits::compact`].
///
/// On every subset of the mask the result carries bit `i` of the
/// compacted subset at bit `place(i) − target` and nothing else; since
/// [`Bits::gather`] masks its input first, that is every input. `mask`
/// with at most 16 set bits; wider masks pass vacuously.
#[must_use]
pub fn gather_is_exact_by<W: Word>(
    mask: W,
    factor: W,
    target: u32,
    place: impl Fn(u32) -> u32,
) -> bool {
    if mask.count_ones() > 16 {
        return true;
    }
    mask.subsets().all(|s| {
        let packed = s.compact(mask);
        let mut want = W::ZERO;
        for i in 0..mask.count_ones() {
            if packed.bit(i) {
                want = want.or(W::ONE.shl(place(i) - target));
            }
        }
        s.gather(mask, factor, target) == want
    })
}
/// An order-preserving gather triple agrees with [`Bits::compact`].
///
/// [`gather_is_exact_by`] with `place(i) = target + i`. `mask` with at
/// most 16 set bits; wider masks pass vacuously.
#[must_use]
pub fn gather_is_exact<W: Word>(mask: W, factor: W, target: u32) -> bool {
    gather_is_exact_by(mask, factor, target, |i| target + i)
}

/// Bits `stride` apart with `stride >= k` gather exactly.
///
/// The order-preserving factor exists iff every bit moves left and the
/// window fits, and then the gather agrees with `compact`: the partial
/// products `target + i + stride (j − i)` are distinct for distinct
/// `(i, j)`, so nothing is ever added. Parameter sets the theorem does
/// not cover pass vacuously.
#[must_use]
pub fn strided_gather_is_exact<W: Word>(
    x: W,
    start: u32,
    stride: u32,
    k: u32,
    target: u32,
) -> bool {
    if k == 0 || stride < k || start + stride * (k - 1) >= W::BITS {
        return true;
    }
    let mut mask = W::ZERO;
    for i in 0..k {
        mask = mask.or(W::ONE.shl(start + stride * i));
    }
    let fits = target + k <= W::BITS && target >= start + (stride - 1) * (k - 1);
    W::gather_factor(mask, target).map_or(!fits, |factor| {
        fits && x.gather(mask, factor, target) == x.compact(mask)
    })
}

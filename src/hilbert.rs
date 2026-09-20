//! Hilbert curves as two scans over the Morton code: parities one way,
//! the carry chain two bits wide the other.
//!
//! A 2D Hilbert index has the shape of a Morton code: one pair of bits
//! per level, most significant level first. The difference is that
//! each level is read in a frame the levels above it chose, one of
//! four: the base curve, transposed, point-reflected, or both. Which
//! frame is a function of the pairs above, and of nothing else, so
//! decoding is two suffix XORs over the levels
//! ([`Bits::suffix_xor`]; Hacker's Delight 16-2 in its parallel-prefix
//! form) and then a Morton decode: straight-line, about a dozen
//! operations, on any [`Word`].
//!
//! Encoding is a scan too, of a heavier kind. Read with the coordinates
//! as input, the frame at each level is an affine map of the frame
//! above it, `(a, c) ↦ (a ^ ¬(c ^ x), c)` when `x = y` and
//! `(a, c) ↦ (c ^ x, a ^ x)` when `x ≠ y`. The linear parts do not
//! commute (they generate `GL(2, 2) ≅ S₃`), so no parity and no adder
//! computes the product, but every map is known before any frame is,
//! and affine maps compose associatively, so Kogge–Stone does. Two
//! involutions multiply to an element of order three, so once levels
//! are paired every window is `v ↦ m·v + t` over GF(4) with `m` in
//! GF(4)*: four bit-sliced words, one GF(4) multiplication per word
//! pair per round, `⌈log₂(BITS / 2)⌉ − 1` rounds after the pairing.
//! That is the adder's carry chain, whose maps are `c ↦ p·c + g` over
//! GF(2), with GF(4) in place of GF(2). The construction is
//! rawrunprotected's (*2D Hilbert curves in O(log n)*,
//! threadlocalmutex.com, 2016, public domain); [`Hilbert2::from_morton`]
//! is it on any [`Word`], in the Morton layout, with laws. Both
//! directions are checked against the textbook `xy2d` / `d2xy` loops
//! and the four-state machine in [`crate::laws::reference`].
//!
//! The curve of order `n` starts at `(0, 0)`, ends at `(2^n − 1, 0)`,
//! and visits the quadrants lower-left, upper-left, upper-right,
//! lower-right, the order of Hacker's Delight figure 16-1 and of the
//! `xy2d` / `d2xy` loops everyone copies. Consecutive indices are
//! adjacent cells (`hilbert_consecutive_are_adjacent` in
//! [`crate::laws`]).
//!
//! ```
//! use hakmem::prelude::*;
//!
//! // Order 2: the 4 × 4 curve, index 12 is the cell (3, 1).
//! let h = Hilbert2::<u8>::encode_order(3, 1, 2);
//! assert_eq!(h.index(), 12);
//! assert_eq!(h.decode_order(2), (3, 1));
//! // The full-width curve of a u32 has 16 levels: a 65536 × 65536 grid.
//! let h = Hilbert2::<u32>::encode(40_000, 7);
//! assert_eq!(h.decode(), (40_000, 7));
//! ```

use crate::bits::Bits;
use crate::dilated::{Dilated, Morton2};
use crate::word::Word;

/// A 2D Hilbert index over the full width of `W`: `BITS / 2` levels,
/// a `2^(BITS/2)` square. Ordered as an integer, which is the curve
/// order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Hilbert2<W: Word>(W);

impl<W: Word> Hilbert2<W> {
    /// Levels of the full-width curve: `BITS / 2`.
    pub const LEVELS: u32 = W::BITS / 2;

    /// The index of `(x, y)` on the full-width curve. Coordinates
    /// above [`LEVELS`](Self::LEVELS) bits are dropped.
    #[inline]
    #[must_use]
    pub fn encode(x: W, y: W) -> Self {
        Self::from_morton(Morton2::encode(x, y))
    }

    /// The `(x, y)` of this index on the full-width curve.
    #[inline]
    #[must_use]
    pub fn decode(self) -> (W, W) {
        self.into_morton().decode()
    }

    /// The index of `(x, y)` on the curve of `order` levels, `order`
    /// in `0..=LEVELS`; coordinates must be below `2^order`
    /// (debug-asserted).
    ///
    /// The order-`n` curve is the first `4^n` cells of the full-width
    /// curve, transposed when `LEVELS − n` is odd: every level above
    /// reads a zero pair, and a zero pair transposes the frame below
    /// it (`hilbert_order_laws`). So this is [`encode`](Self::encode)
    /// with the arguments swapped for odd depth.
    #[inline]
    #[must_use]
    pub fn encode_order(x: W, y: W, order: u32) -> Self {
        debug_assert!(order <= Self::LEVELS, "order {order} > {}", Self::LEVELS);
        debug_assert!(
            x.and(W::low_ones(order).not()).is_zero() && y.and(W::low_ones(order).not()).is_zero(),
            "coordinates above 2^{order}"
        );
        if (Self::LEVELS - order) & 1 == 0 {
            Self::encode(x, y)
        } else {
            Self::encode(y, x)
        }
    }

    /// The `(x, y)` of this index on the curve of `order` levels; the
    /// index must be below `4^order` (debug-asserted).
    #[inline]
    #[must_use]
    pub fn decode_order(self, order: u32) -> (W, W) {
        debug_assert!(order <= Self::LEVELS, "order {order} > {}", Self::LEVELS);
        debug_assert!(
            self.0.and(W::low_ones(2 * order).not()).is_zero(),
            "index above 4^{order}"
        );
        let (x, y) = self.decode();
        if (Self::LEVELS - order) & 1 == 0 {
            (x, y)
        } else {
            (y, x)
        }
    }

    /// Wraps an index.
    #[inline]
    #[must_use]
    pub const fn from_index(index: W) -> Self {
        Self(index)
    }

    /// The raw index.
    #[inline]
    #[must_use]
    pub const fn index(self) -> W {
        self.0
    }

    /// From the Morton code of the same cell: the levels re-read in
    /// their frames, all at once.
    ///
    /// Per level, with `x`, `y` the coordinate bits and `lo = x ^ y`, the
    /// frame map is `(a, c) ↦ (c ^ x, a ^ x)` when `lo` is set, else
    /// `(a, c) ↦ (a ^ ¬(c ^ x), c)`. Windows of two levels have linear
    /// parts in GF(4)* and translations in GF(4); the suffix product of
    /// the windows is a Kogge–Stone scan with one GF(4) multiplication
    /// per word pair per round, and the frame at a level is the
    /// translation of the product above it, the top frame being zero.
    /// rawrunprotected's construction (2016), in the dilated layout.
    #[must_use]
    pub fn from_morton(morton: Morton2<W>) -> Self {
        let code = morton.code();
        let lanes = Dilated::<W, 2>::mask();
        let x = code.and(lanes);
        let y = code.shr(1).and(lanes);
        let lo = x.xor(y);
        let not_lo = lo.xor(lanes);
        let neither = x.or(y).xor(lanes);
        let x_only = x.and(y.xor(lanes));
        // Windows of two levels, composed by hand: their linear parts
        // are products of two involutions, which lie in the cyclic
        // group of order three, GF(4)*. From here on every window is
        // `v ↦ m·v + t` over GF(4), `m` in `(ma, mb)`, `t` in `(ta, tb)`,
        // and a round is one multiplication in GF(4) per word pair.
        let mut ma = lo.or(not_lo.shr(2));
        let mut mb = lo.shr(2).xor(lo);
        let mut ta = neither.shr(2).xor(not_lo.and(x_only.shr(2))).xor(neither);
        let mut tb = lo.and(neither.shr(2)).xor(x_only.shr(2)).xor(x_only);
        let mut step = 4;
        while step < W::BITS {
            let (pa, pb, pc, pd) = (ma, mb, ta, tb);
            let (qa, qb, qc, qd) = (pa.shr(step), pb.shr(step), pc.shr(step), pd.shr(step));
            // `t ^= m · t'`: `(a + bω)(c' + d'ω)` with `ω² = ω + 1`.
            ta = pc.xor(pa.and(qc)).xor(pb.and(qd));
            tb = pd.xor(pb.and(qc)).xor(pa.xor(pb).and(qd));
            // `m ·= m'`, not needed after the last round.
            if step * 2 < W::BITS {
                ma = pa.and(qa).xor(pb.and(qb));
                mb = pa.and(qb).xor(pb.and(qa.xor(qb)));
            }
            step <<= 1;
        }
        // The frames, read off the translations of the products above.
        let swap_like = ta.xor(ta.shr(2));
        let flip_like = tb.xor(tb.shr(2));
        let hi = flip_like.or(lo.or(swap_like).xor(lanes));
        Self(lo.or(hi.shl(1)))
    }

    /// To the Morton code of the same cell: the levels read back out
    /// of their frames, all at once.
    ///
    /// Per level, with `s` the pair: the frame below it transposes
    /// when `s ∈ {0, 3}` and reflects when `s = 3`, so `swap` is the
    /// parity of `¬(s_hi ^ s_lo)` and `flip` the parity of
    /// `s_hi & s_lo` over the levels above, two suffix XORs. Then
    /// `x = s_hi ^ swap·s_lo ^ flip` and `y = x ^ s_lo`.
    #[must_use]
    pub fn into_morton(self) -> Morton2<W> {
        let lanes = Dilated::<W, 2>::mask();
        let s_lo = self.0.and(lanes);
        let s_hi = self.0.shr(1).and(lanes);
        // Even bits of a suffix XOR over one lane hold the inclusive
        // parity; odd bit `2i + 1` the parity over the levels strictly
        // above `i`, the exclusive scan for free.
        let swap = s_hi.xor(s_lo).xor(lanes).suffix_xor().shr(1).and(lanes);
        let flip = s_hi.and(s_lo).suffix_xor().shr(1).and(lanes);
        let x = flip.xor(s_hi).xor(swap.and(s_lo));
        let y = x.xor(s_lo);
        Morton2::from_code(x.or(y.shl(1)))
    }
}

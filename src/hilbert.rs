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

use core::ops::RangeBounds;

use crate::bits::Bits;
use crate::cover::{Masks, Quadrants, Rect, small};
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
        self.to_morton().decode()
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
    pub fn to_morton(self) -> Morton2<W> {
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

/// One level of the encode machine of `crate::laws::reference`, for the
/// batch table: frame `(swap, flip)` as bits 0 and 1, coordinate bits
/// `x`, `y`; returns the digit and the frame below.
const fn level(frame: u8, x: u8, y: u8) -> (u8, u8) {
    let (swap, flip) = (frame & 1, (frame >> 1) & 1);
    let lo = x ^ y;
    let hi = x ^ flip ^ (swap & lo);
    let swap_below = swap ^ 1 ^ hi ^ lo;
    let flip_below = flip ^ (hi & lo);
    ((hi << 1) | lo, swap_below | (flip_below << 1))
}

/// `L` levels of the encode machine a byte, modulo the reflection that
/// flips both axes, for the batch kernels: [`level`] in the frame
/// `(swap, flip)` is [`level`] in `(swap, 0)` on the cell moved by
/// `x ^ flip, y ^ flip`, with `flip` added below, so `flip` is a XOR
/// mask and `swap` alone is tabled. In the cell basis `(x, x ^ y)` the
/// reflection moves `x` only. The entry at `swap << 2L | cells`, `L`
/// cells of that basis from the top, holds the high digit bits at their
/// places (the odd bits), the `flip` gained below at every `x` place
/// (the even bits), and the `swap` gained at bit `2L`: all of the state
/// relative, so the next state is `(state ^ entry)` masked and the next
/// index `cells ^ state`. The low digit bits are `x ^ y`, the input
/// itself. `N = 2 · 4^L` entries, half of the unreduced table.
#[cfg_attr(
    not(any(
        all(target_arch = "x86_64", not(feature = "portable")),
        all(
            target_arch = "aarch64",
            target_feature = "neon",
            not(feature = "portable")
        )
    )),
    allow(dead_code)
)]
const fn reduced<const N: usize>() -> [u8; N] {
    let levels = N.trailing_zeros() / 2;
    let mut table = [0u8; N];
    let mut i = 0;
    while i < N {
        #[allow(clippy::cast_possible_truncation)]
        let (swap, cells) = ((i >> (2 * levels)) as u8, i as u8);
        let mut frame = swap;
        let mut entry = 0u8;
        let mut j = levels;
        while j > 0 {
            j -= 1;
            let x = (cells >> (2 * j)) & 1;
            let y = x ^ ((cells >> (2 * j + 1)) & 1);
            let (digit, below) = level(frame, x, y);
            entry |= (digit >> 1) << (2 * j + 1);
            frame = below;
        }
        // The flip gained, at every `x` place.
        entry |= (frame >> 1) * (0x55 >> (8 - 2 * levels));
        table[i] = entry | (((frame & 1) ^ swap) << (2 * levels));
        i += 1;
    }
    table
}

/// Two levels a byte, 32 entries: two NEON registers for `tbl`.
#[cfg_attr(
    not(all(
        target_arch = "aarch64",
        target_feature = "neon",
        not(feature = "portable")
    )),
    allow(dead_code)
)]
const TABLE4: [u8; 32] = reduced();

/// Three levels a byte, 128 entries: two AVX-512 registers for
/// `vpermi2b`.
#[cfg_attr(
    not(all(target_arch = "x86_64", not(feature = "portable"))),
    allow(dead_code)
)]
const TABLE6: [u8; 128] = reduced();

macro_rules! hilbert2_batch {
    ($($w:ty => $kernel:ident),* $(,)?) => {$(
        impl Hilbert2<$w> {
            /// [`from_morton`](Self::from_morton) over a slice of keys, in
            /// place: each Morton code becomes the Hilbert index of the
            /// same cell.
            ///
            /// With AVX-512 VBMI a step is one `vpermi2b` through a
            /// 128-entry table per register of keys, three levels at a
            /// time: the frame modulo the reflection of both axes, which
            /// rides as a XOR mask on the cells, three `vpternlog` around
            /// the lookup. With NEON the same reduction two levels at a
            /// time, 32 entries in two registers for `tbl`. Otherwise, and
            /// for the keys past the
            /// last whole batch, it is [`from_morton`](Self::from_morton)
            /// per key. Coordinates never enter: fill the slice with
            /// [`Morton2::encode`] from whatever layout the points are in.
            pub fn from_morton_in_place(keys: &mut [$w]) {
                let done = batch::$kernel(keys);
                for key in &mut keys[done..] {
                    *key = Self::from_morton(Morton2::from_code(*key)).index();
                }
            }

            /// [`to_morton`](Self::to_morton) over a slice of keys, in
            /// place: each Hilbert index becomes the Morton code of the
            /// same cell. Per key; the decode is two suffix XORs and has no
            /// table to batch.
            pub fn to_morton_in_place(keys: &mut [$w]) {
                for key in keys {
                    *key = Self::from_index(*key).to_morton().code();
                }
            }

            /// [`from_morton_in_place`](Self::from_morton_in_place) on the
            /// curve of `order` levels, as [`encode_order`](Self::encode_order)
            /// per key; `order` in `0..=LEVELS`, codes below `4^order`
            /// (debug-asserted). The order-`n` curve is the full-width one
            /// with `x` and `y` swapped when `LEVELS − n` is odd: a swap of
            /// the Morton lanes per key, then the full-width batch.
            pub fn from_morton_in_place_order(keys: &mut [$w], order: u32) {
                debug_assert!(order <= Self::LEVELS, "order {order} > {}", Self::LEVELS);
                debug_assert!(
                    keys.iter().all(|&k| k.checked_shr(2 * order).unwrap_or(0) == 0),
                    "codes above 4^{order}"
                );
                if (Self::LEVELS - order) & 1 == 1 {
                    let even = Dilated::<$w, 2>::mask();
                    for key in keys.iter_mut() {
                        *key = ((*key & even) << 1) | ((*key >> 1) & even);
                    }
                }
                Self::from_morton_in_place(keys);
            }

            /// [`to_morton_in_place`](Self::to_morton_in_place) on the
            /// curve of `order` levels, as [`decode_order`](Self::decode_order)
            /// per key; indices below `4^order` (debug-asserted).
            pub fn to_morton_in_place_order(keys: &mut [$w], order: u32) {
                debug_assert!(order <= Self::LEVELS, "order {order} > {}", Self::LEVELS);
                debug_assert!(
                    keys.iter().all(|&k| k.checked_shr(2 * order).unwrap_or(0) == 0),
                    "indices above 4^{order}"
                );
                Self::to_morton_in_place(keys);
                if (Self::LEVELS - order) & 1 == 1 {
                    let even = Dilated::<$w, 2>::mask();
                    for key in keys.iter_mut() {
                        *key = ((*key & even) << 1) | ((*key >> 1) & even);
                    }
                }
            }
        }
    )*};
}

hilbert2_batch!(u32 => from_morton_u32, u64 => from_morton_u64);

macro_rules! hilbert2_columns {
    ($($w:ty),* $(,)?) => {$(
        impl Hilbert2<$w> {
            /// [`encode`](Self::encode) over two columns of coordinates:
            /// `out[i]` is the index of `(xs[i], ys[i])`, the shape
            /// coordinates arrive in from geodata (quantised longitude and
            /// latitude). Coordinates above `LEVELS` bits are dropped.
            ///
            /// [`Morton2::encode_columns`] into `out`, then
            /// [`from_morton_in_place`](Self::from_morton_in_place): both
            /// batches choose their kernels at run time, so a build
            /// without flags gets them. The Morton step is about a tenth
            /// of the whole; skipping it, as `Hilbert3` does, would need
            /// byte planes for 2D, which are not worth the code here.
            ///
            /// # Panics
            ///
            /// If the three slices differ in length.
            pub fn encode_columns(xs: &[$w], ys: &[$w], out: &mut [$w]) {
                Morton2::<$w>::encode_columns(xs, ys, out);
                Self::from_morton_in_place(out);
            }

            /// [`decode`](Self::decode) of a column of indices into two
            /// columns of coordinates, the inverse of
            /// [`encode_columns`](Self::encode_columns). The indices go
            /// to Morton codes per key ([`to_morton`](Self::to_morton),
            /// two suffix XORs) in blocks of 256 on the stack, and each
            /// block through [`Morton2::decode_columns`].
            ///
            /// # Panics
            ///
            /// If the three slices differ in length.
            pub fn decode_columns(keys: &[$w], xs: &mut [$w], ys: &mut [$w]) {
                let n = keys.len();
                assert!(
                    xs.len() == n && ys.len() == n,
                    "columns of {} and {} for {n} keys",
                    xs.len(),
                    ys.len()
                );
                let mut codes = [0 as $w; 256];
                for ((k, x), y) in keys.chunks(256).zip(xs.chunks_mut(256)).zip(ys.chunks_mut(256)) {
                    let codes = &mut codes[..k.len()];
                    for (c, &h) in codes.iter_mut().zip(k) {
                        *c = Self::from_index(h).to_morton().code();
                    }
                    Morton2::<$w>::decode_columns(codes, x, y);
                }
            }
        }
    )*};
}

hilbert2_columns!(u32, u64);

/// The Hilbert curve for [`crate::cover`]: [`level`] read backwards, the
/// quadrant of each digit in each of the four frames.
pub(crate) struct HilbertQuadrants;

/// `DIGITS[frame][x | y << 1]` is [`level`].
const DIGITS: [[(u8, u8); 4]; 4] = {
    let mut t = [[(0, 0); 4]; 4];
    let mut f = 0;
    while f < 4 {
        let mut q = 0;
        while q < 4 {
            t[f as usize][q as usize] = level(f, q & 1, q >> 1);
            q += 1;
        }
        f += 1;
    }
    t
};

static H_MASKS: Masks<4> = Masks::build(DIGITS);

/// A step of the curve between two quadrants of a node, as a side of a
/// rectangle sees it. `axis` 0 crosses a vertical line (a step in `x`),
/// 1 a horizontal one; `up` whether toward the upper half. On the other
/// axis the step sits in half `half`, at the corner cell `corner` of it
/// (0 the first, 1 the last): the curve leaves a quadrant at the corner
/// where the next one starts.
#[derive(Clone, Copy)]
struct Step {
    axis: u8,
    up: u8,
    half: u8,
    corner: u8,
}

/// The three steps of a node in each frame.
const STEPS: [[Step; 3]; 4] = {
    let mut t = [[Step {
        axis: 0,
        up: 0,
        half: 0,
        corner: 0,
    }; 3]; 4];
    let mut f = 0;
    while f < 4 {
        let mut d = 0;
        while d < 3 {
            let (ax, ay, ag) = CHILDREN[f][d];
            let (bx, by, _) = CHILDREN[f][d + 1];
            // Where the first quadrant ends: the cell of its digit 3.
            let (ex, ey, _) = CHILDREN[ag as usize][3];
            t[f][d] = if ax == bx {
                Step {
                    axis: 1,
                    up: by,
                    half: ax,
                    corner: ex,
                }
            } else {
                Step {
                    axis: 0,
                    up: bx,
                    half: ay,
                    corner: ey,
                }
            };
            d += 1;
        }
        f += 1;
    }
    t
};

/// `CHILDREN[frame][digit]`: one level of [`level`] read backwards, for
/// the tables built at compile time.
#[allow(clippy::cast_possible_truncation)]
const CHILDREN: [[(u8, u8, u8); 4]; 4] = {
    let mut t = [[(0, 0, 0); 4]; 4];
    let mut f = 0;
    while f < 4 {
        let mut q = 0;
        while q < 4 {
            let (d, below) = DIGITS[f][q];
            t[f][d as usize] = ((q & 1) as u8, (q >> 1) as u8, below);
            q += 1;
        }
        f += 1;
    }
    t
};

/// `PER[axis][up][frame]`: how many steps of a node cross that way.
const PER: [[[u8; 4]; 2]; 2] = {
    let mut t = [[[0; 4]; 2]; 2];
    let mut f = 0;
    while f < 4 {
        let mut d = 0;
        while d < 3 {
            let s = STEPS[f][d];
            t[s.axis as usize][s.up as usize][f] += 1;
            d += 1;
        }
        f += 1;
    }
    t
};

/// `AT[axis][up][frame][half << 1 | corner]`: the same steps by where
/// they sit along the side, `h` the half of the node: at `0`, `h - 1`,
/// `h` or `2h - 1` from its first cell. Counting them is then four
/// compares and no branch on the frame.
const AT: [[[[u8; 4]; 4]; 2]; 2] = {
    let mut t = [[[[0; 4]; 4]; 2]; 2];
    let mut f = 0;
    while f < 4 {
        let mut d = 0;
        while d < 3 {
            let s = STEPS[f][d];
            t[s.axis as usize][s.up as usize][f][(s.half << 1 | s.corner) as usize] += 1;
            d += 1;
        }
        f += 1;
    }
    t
};

/// `NEXT[axis][frame][fixed][free]`: the frame of the child whose bit on
/// the axis across the side is `fixed` and on the other `free`.
#[allow(clippy::cast_possible_truncation)]
const NEXT: [[[[u8; 2]; 2]; 4]; 2] = {
    let mut t = [[[[0; 2]; 2]; 4]; 2];
    let mut f = 0;
    while f < 4 {
        let mut a = 0;
        while a < 2 {
            let mut b = 0;
            while b < 2 {
                t[0][f][a][b] = level(f as u8, a as u8, b as u8).1;
                t[1][f][a][b] = level(f as u8, b as u8, a as u8).1;
                b += 1;
            }
            a += 1;
        }
        f += 1;
    }
    t
};

/// The frame of the node at level `l` holding the cell of index `h`: the
/// frames of the decoder form the Klein group, so it is the parity of the
/// digits above `l` that swap (0 and 3) and of those that flip (3).
fn frame<W: Word>(h: W, l: u32) -> u8 {
    if l >= W::BITS / 2 {
        return 0;
    }
    let even = W::splat_byte(0x55);
    let (hi, lo) = (h.shr(1).and(even), h.and(even));
    let above = W::low_ones(2 * l).not().and(even);
    let swap = hi.xor(lo).not().and(above).parity();
    let flip = hi.and(lo).and(above).parity();
    u8::from(swap) | u8::from(flip) << 1
}

/// All ones when `b`.
#[inline]
fn mask<W: Word>(b: bool) -> W {
    W::ZERO.wrapping_sub(if b { W::ONE } else { W::ZERO })
}

/// The steps entering the rectangle across one side, the line between
/// cells `line - 1` and `line` on `axis`, over `lo..=hi` on the other,
/// on the grid coarsened by `s`, `levels` levels left. `up` is the
/// direction that enters. `corners` are the indices of the rectangle's
/// corners at the ends of the side, which name the frames.
///
/// Only nodes at level `tz(line) + 1` straddle the line, one column of
/// them, and what a node contributes depends only on its frame. The
/// ones strictly between the ends are counted by frame: the weight of a
/// frame below `j` levels is the steps under it, and the count below a
/// node number `c` is a sum along the path of `c`, both built from the
/// bottom level up in four lanes. The two end nodes are counted cell
/// by cell.
#[allow(clippy::too_many_arguments, clippy::many_single_char_names)]
fn side<W: Word>(
    levels: u32,
    s: u32,
    axis: usize,
    line: W,
    up: u8,
    (lo, hi): (W, W),
    (lo_corner, hi_corner): (W, W),
) -> W {
    if line.is_zero() || !line.shr(levels).is_zero() {
        return W::ZERO;
    }
    let lvl = line.trailing_zeros() + 1;
    let node = line.shr(lvl);
    let (c0, c1) = (lo.shr(lvl), hi.shr(lvl));
    let next = &NEXT[axis];
    let mut n = W::ZERO;
    let first = c0.wrapping_add(W::ONE);
    if c1 > first {
        let depth = crate::cover::bitlen(c0.xor(c1));
        let mut weight = PER[axis][up as usize].map(|v| small::<W>(v.into()));
        let (mut to_last, mut to_first) = ([W::ZERO; 4], [W::ZERO; 4]);
        for j in 0..depth {
            let fixed = usize::from(node.bit(j));
            let (a, b) = (c1.bit(j), first.bit(j));
            let (mut w, mut l, mut f) = ([W::ZERO; 4], [W::ZERO; 4], [W::ZERO; 4]);
            // Masked, so the lanes are selects and the indices need no
            // bounds check.
            let (ma, mb) = (mask::<W>(a), mask::<W>(b));
            for g in 0..4 {
                let [zero, one] = next[g][fixed].map(|v| usize::from(v) & 3);
                w[g] = weight[zero].wrapping_add(weight[one]);
                let (la, lb) = (if a { one } else { zero }, if b { one } else { zero });
                l[g] = weight[zero].and(ma).wrapping_add(to_last[la]);
                f[g] = weight[zero].and(mb).wrapping_add(to_first[lb]);
            }
            (weight, to_last, to_first) = (w, l, f);
        }
        let top = frame(lo_corner, lvl + depth + s) as usize;
        n = to_last[top].wrapping_sub(to_first[top]);
    }
    let h = W::ONE.shl(lvl - 1);
    let offsets = [
        W::ZERO,
        h.wrapping_sub(W::ONE),
        h,
        h.shl(1).wrapping_sub(W::ONE),
    ];
    let end = |c: W, corner: W| {
        let at = AT[axis][usize::from(up)][frame(corner, lvl + s) as usize];
        let origin = c.shl(lvl);
        let mut n = W::ZERO;
        for (count, offset) in at.into_iter().zip(offsets) {
            let pos = origin.wrapping_add(offset);
            let steps = if lo <= pos && pos <= hi { count } else { 0 };
            n = n.wrapping_add(small::<W>(steps.into()));
        }
        n
    };
    n = n.wrapping_add(end(c0, lo_corner));
    if c1 != c0 {
        n = n.wrapping_add(end(c1, hi_corner));
    }
    n
}

impl Quadrants for HilbertQuadrants {
    /// The indices of `(x0, y0)`, `(x1, y0)`, `(x0, y1)`, `(x1, y1)`.
    type Context<W> = [W; 4];

    #[inline]
    fn digit(frame: u8, dx: u8, dy: u8) -> (u8, u8) {
        level(frame, dx, dy)
    }

    #[inline]
    fn child(k: u32, frame: u8, p: u32) -> (u8, u8, u8) {
        H_MASKS.child(k, frame, p)
    }

    #[inline]
    fn cells(k: u32, frame: u8, cols: usize, rows: usize) -> u64 {
        H_MASKS.cells(k, frame, cols, rows)
    }

    fn context<W: Word>(r: &Rect<W>) -> [W; 4] {
        let h = |x, y| Hilbert2::<W>::encode(x, y).index();
        [h(r.x0, r.y0), h(r.x1, r.y0), h(r.x0, r.y1), h(r.x1, r.y1)]
    }

    /// The curve is continuous, so a run starts where a step enters the
    /// rectangle across a side, or at the origin; four sides.
    fn runs<W: Word>(levels: u32, r: &Rect<W>, s: u32, corners: &[W; 4]) -> W {
        let levels = levels - s;
        let (x0, x1, y0, y1) = (r.x0.shr(s), r.x1.shr(s), r.y0.shr(s), r.y1.shr(s));
        let top = W::low_ones(levels);
        let [h00, h10, h01, h11] = *corners;
        let mut n = if x0.is_zero() && y0.is_zero() {
            W::ONE
        } else {
            W::ZERO
        };
        n = n.wrapping_add(side(levels, s, 0, x0, 1, (y0, y1), (h00, h01)));
        n = n.wrapping_add(side(levels, s, 1, y0, 1, (x0, x1), (h00, h10)));
        if x1 < top {
            let line = x1.wrapping_add(W::ONE);
            n = n.wrapping_add(side(levels, s, 0, line, 0, (y0, y1), (h10, h11)));
        }
        if y1 < top {
            let line = y1.wrapping_add(W::ONE);
            n = n.wrapping_add(side(levels, s, 1, line, 0, (x0, x1), (h01, h11)));
        }
        n
    }
}

impl<W: Word> Hilbert2<W> {
    /// The keys of the rectangle `x` × `y` as at most `out.len()` ranges
    /// of indices: the part of `out` it filled.
    ///
    /// The ranges are inclusive, sorted, disjoint and not touching, and
    /// every cell of the rectangle has its index in one of them. When
    /// the budget allows the exact cover, the ranges hold those cells
    /// and no others; otherwise they hold more, the least any budget's
    /// worth of ranges of the deepest cover that fits can hold (design
    /// notes, section 5). A scan of keys sorted on the curve seeks once
    /// per range. `x` and `y` are any ranges, `..` included; past the
    /// grid they are clipped, and an empty one gives no ranges. The
    /// rest of `out` is scratch.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// // The 2 × 2 block at the origin is the first quadrant of the
    /// // bottom level: indices 0..=3, one range.
    /// let mut out = [(0u8, 0u8); 4];
    /// assert_eq!(Hilbert2::<u8>::cover(0..=1, 0..2, &mut out), [(0, 3)]);
    /// ```
    ///
    /// # Panics
    ///
    /// If the rectangle is not empty and `out` is.
    pub fn cover(x: impl RangeBounds<W>, y: impl RangeBounds<W>, out: &mut [(W, W)]) -> &[(W, W)] {
        crate::cover::cover_ranges::<W, HilbertQuadrants>(Self::LEVELS, x, y, out)
    }

    /// Whether some cell of the rectangle `x` × `y` has its index in
    /// `keys`: the test that prunes a block of data sorted on the curve
    /// by its least and greatest key (a granule, a Parquet row group, a
    /// file), as `ClickHouse` does for its sparse index. Answered by one
    /// descent of the quadtree, where a node is both an interval of
    /// indices and a square of cells; only the nodes on the paths of the
    /// two ends are partly in the interval, so it visits a few nodes a
    /// step, three levels at a time, and decodes nothing.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// // Indices 0..=3 are the 2 × 2 block at the origin.
    /// assert!(Hilbert2::<u8>::intersects(0..=3, 1..=5, 1..=5));
    /// assert!(!Hilbert2::<u8>::intersects(0..4, 2..=5, ..));
    /// ```
    #[must_use]
    pub fn intersects(
        keys: impl RangeBounds<W>,
        x: impl RangeBounds<W>,
        y: impl RangeBounds<W>,
    ) -> bool {
        crate::cover::intersects_ranges::<W, HilbertQuadrants>(Self::LEVELS, keys, x, y)
    }
}

/// The batch kernels. Each returns how many keys from the front it
/// converted, a whole number of batches; the caller finishes the rest.
/// The intrinsics are `unsafe` solely because they require the target
/// feature: on `x86_64` a kernel carries its features in
/// `target_feature` and is called only where `crate::cpu` found them;
/// on `aarch64` NEON is a compile-time fact.
mod batch {
    #[cfg(all(target_arch = "x86_64", not(feature = "portable")))]
    #[allow(unsafe_code)]
    mod vbmi {
        use core::arch::x86_64::{
            _mm512_loadu_si512, _mm512_permutex2var_epi8, _mm512_set1_epi8, _mm512_set1_epi32,
            _mm512_set1_epi64, _mm512_setzero_si512, _mm512_slli_epi32, _mm512_slli_epi64,
            _mm512_srli_epi32, _mm512_srli_epi64, _mm512_srlv_epi32, _mm512_srlv_epi64,
            _mm512_storeu_si512, _mm512_ternarylogic_epi64,
        };

        use super::super::TABLE6;

        // `vpternlog` truth tables over `(a, b, c) = (0xF0, 0xCC, 0xAA)`.
        /// `a ^ (b & c)`
        const XOR_AND: i32 = 0x78;
        /// `(a & c) ^ b`
        const AND_XOR: i32 = 0x6C;
        /// `a | (b & c)`
        const OR_AND: i32 = 0xF8;
        /// `(a ^ b) & c`
        const XOR_THEN_AND: i32 = 0x28;

        macro_rules! kernel {
            (
                $name:ident, $w:ty, $lanes:literal, $set1:ident, $srli:ident, $slli:ident,
                $srlv:ident
            ) => {
                /// Batches of 64 keys, `64 / LANES` registers in flight.
                #[target_feature(enable = "avx512f,avx512bw,avx512vbmi")]
                pub(in crate::hilbert) fn $name(keys: &mut [$w]) -> usize {
                    const REGS: usize = 64 / $lanes;
                    const LEVELS: u32 = <$w>::BITS / 2;
                    #[allow(clippy::cast_possible_truncation)]
                    const STEPS: u8 = LEVELS.div_ceil(3) as u8;
                    // Levels above the key read as cell 0; two of them
                    // are the identity, so an odd number starts swapped.
                    #[allow(clippy::cast_possible_truncation)]
                    const START: u8 = (((3 * STEPS as u32 - LEVELS) & 1) << 6) as u8;
                    let (chunks, _) = keys.as_chunks_mut::<64>();
                    let done = chunks.len() * 64;
                    for chunk in chunks {
                        // SAFETY: AVX-512 F, BW and VBMI are enabled on this function;
                        // every load and store stays inside the 64 keys of
                        // `chunk` or the 128 bytes of the table.
                        unsafe {
                            let lo = _mm512_loadu_si512(TABLE6.as_ptr().cast());
                            let hi = _mm512_loadu_si512(TABLE6.as_ptr().add(64).cast());
                            let cells = $set1(63);
                            let high_digits = $set1(0x2A);
                            let state_bits = $set1(0x55);
                            #[allow(clippy::cast_possible_wrap)]
                            let odd = _mm512_set1_epi8(0xAA_u8 as i8);
                            let even = _mm512_set1_epi8(0x55);
                            let mut code = [_mm512_setzero_si512(); REGS];
                            let mut acc = [_mm512_setzero_si512(); REGS];
                            let mut state = [$set1(START.into()); REGS];
                            for (r, c) in code.iter_mut().enumerate() {
                                let m = _mm512_loadu_si512(chunk.as_ptr().add($lanes * r).cast());
                                // Each level's `(x, y)` to `(x, x ^ y)`.
                                *c = _mm512_ternarylogic_epi64::<XOR_AND>(m, $slli::<1>(m), odd);
                            }
                            let mut step = STEPS;
                            while step > 0 {
                                step -= 1;
                                let shift = $set1((6 * step).into());
                                for r in 0..REGS {
                                    let index = _mm512_ternarylogic_epi64::<AND_XOR>(
                                        $srlv(code[r], shift),
                                        state[r],
                                        cells,
                                    );
                                    let entry = _mm512_permutex2var_epi8(lo, index, hi);
                                    acc[r] = _mm512_ternarylogic_epi64::<OR_AND>(
                                        $slli::<6>(acc[r]),
                                        entry,
                                        high_digits,
                                    );
                                    state[r] = _mm512_ternarylogic_epi64::<XOR_THEN_AND>(
                                        state[r], entry, state_bits,
                                    );
                                }
                            }
                            for (r, a) in acc.iter().enumerate() {
                                // The low digit bits: `x ^ y`, the odd bits
                                // of the rewritten code.
                                let a = _mm512_ternarylogic_epi64::<OR_AND>(
                                    *a,
                                    $srli::<1>(code[r]),
                                    even,
                                );
                                _mm512_storeu_si512(chunk.as_mut_ptr().add($lanes * r).cast(), a);
                            }
                        }
                    }
                    done
                }
            };
        }

        kernel!(
            from_morton_u32,
            u32,
            16,
            _mm512_set1_epi32,
            _mm512_srli_epi32,
            _mm512_slli_epi32,
            _mm512_srlv_epi32
        );
        kernel!(
            from_morton_u64,
            u64,
            8,
            _mm512_set1_epi64,
            _mm512_srli_epi64,
            _mm512_slli_epi64,
            _mm512_srlv_epi64
        );
    }

    /// `x86_64` chooses the kernel once a call, as in `hilbert3`: at
    /// compile time when the build has the features, else at run time.
    #[cfg(all(target_arch = "x86_64", not(feature = "portable")))]
    #[allow(unsafe_code)]
    mod dispatch {
        use super::vbmi;
        use crate::cpu;

        macro_rules! keys {
            ($($name:ident: $w:ty;)*) => {$(
                pub(in crate::hilbert) fn $name(keys: &mut [$w]) -> usize {
                    if cpu::avx512vbmi() {
                        // SAFETY: AVX-512 F, BW and VBMI are present.
                        unsafe { vbmi::$name(keys) }
                    } else {
                        0
                    }
                }
            )*};
        }

        keys! {
            from_morton_u32: u32;
            from_morton_u64: u64;
        }
    }

    #[cfg(all(target_arch = "x86_64", not(feature = "portable")))]
    pub(super) use dispatch::{from_morton_u32, from_morton_u64};

    #[cfg(all(
        target_arch = "aarch64",
        target_feature = "neon",
        not(feature = "portable")
    ))]
    #[allow(unsafe_code)]
    mod neon {
        use core::arch::aarch64::{
            vandq_u32, vandq_u64, vbslq_u32, vbslq_u64, vdupq_n_s32, vdupq_n_s64, vdupq_n_u8,
            vdupq_n_u32, vdupq_n_u64, veorq_u32, veorq_u64, vld1q_u8_x2, vld1q_u32, vld1q_u64,
            vqtbl2q_u8, vreinterpretq_u8_u32, vreinterpretq_u8_u64, vreinterpretq_u32_u8,
            vreinterpretq_u64_u8, vshlq_n_u32, vshlq_n_u64, vshlq_u32, vshlq_u64, vshrq_n_u32,
            vshrq_n_u64, vsliq_n_u32, vsliq_n_u64, vst1q_u32, vst1q_u64,
        };

        use super::super::TABLE4;

        macro_rules! kernel {
            (
                $name:ident, $w:ty, $per_reg:literal, $dup:ident, $dup_s:ident, $s:ty,
                $ld:ident, $st:ident, $and:ident, $bsl:ident, $eor:ident, $shl:ident,
                $shl_n:ident, $shr_n:ident, $sli_n:ident, $to_u8:ident, $from_u8:ident
            ) => {
                /// Batches of eight registers.
                pub(in crate::hilbert) fn $name(keys: &mut [$w]) -> usize {
                    // Two levels a step, not three: three is 128 entries in
                    // eight registers beside the 24 the keys hold, and NEON
                    // stops counting at 32. An even number of levels, so no
                    // padding and the top frame is the identity.
                    #[allow(clippy::cast_possible_wrap, clippy::cast_lossless)]
                    const STEPS: $s = (<$w>::BITS / 4) as $s;
                    let (chunks, _) = keys.as_chunks_mut::<{ 8 * $per_reg }>();
                    let done = chunks.len() * 8 * $per_reg;
                    for chunk in chunks {
                        // SAFETY: NEON is enabled by cfg; every load and
                        // store stays inside `chunk` or the 32 bytes of
                        // `TABLE4`.
                        unsafe {
                            let table = vld1q_u8_x2(TABLE4.as_ptr());
                            let cells = $dup(15);
                            let state_bits = $dup(0x15);
                            let odd = $from_u8(vdupq_n_u8(0xAA));
                            let mut code = [$dup(0); 8];
                            let mut acc = [$dup(0); 8];
                            let mut state = [$dup(0); 8];
                            for (r, c) in code.iter_mut().enumerate() {
                                let m = $ld(chunk.as_ptr().add($per_reg * r));
                                // Each level's `(x, y)` to `(x, x ^ y)`.
                                *c = $eor(m, $and($shl_n::<1>(m), odd));
                            }
                            let mut step = STEPS;
                            while step > 0 {
                                step -= 1;
                                let shift = $dup_s(-4 * step);
                                for r in 0..8 {
                                    let index = $eor($and($shl(code[r], shift), cells), state[r]);
                                    let entry = $from_u8(vqtbl2q_u8(table, $to_u8(index)));
                                    // The low four bits of the entry, flips and all; the
                                    // flips sit where the low digit bits go.
                                    acc[r] = $sli_n::<4>(entry, acc[r]);
                                    state[r] = $and($eor(state[r], entry), state_bits);
                                }
                            }
                            for (r, a) in acc.iter().enumerate() {
                                // The low digit bits over the flips: `x ^ y`,
                                // the odd bits of the rewritten code.
                                let a = $bsl(odd, *a, $shr_n::<1>(code[r]));
                                $st(chunk.as_mut_ptr().add($per_reg * r), a);
                            }
                        }
                    }
                    done
                }
            };
        }

        kernel!(
            from_morton_u64,
            u64,
            2,
            vdupq_n_u64,
            vdupq_n_s64,
            i64,
            vld1q_u64,
            vst1q_u64,
            vandq_u64,
            vbslq_u64,
            veorq_u64,
            vshlq_u64,
            vshlq_n_u64,
            vshrq_n_u64,
            vsliq_n_u64,
            vreinterpretq_u8_u64,
            vreinterpretq_u64_u8
        );
        kernel!(
            from_morton_u32,
            u32,
            4,
            vdupq_n_u32,
            vdupq_n_s32,
            i32,
            vld1q_u32,
            vst1q_u32,
            vandq_u32,
            vbslq_u32,
            veorq_u32,
            vshlq_u32,
            vshlq_n_u32,
            vshrq_n_u32,
            vsliq_n_u32,
            vreinterpretq_u8_u32,
            vreinterpretq_u32_u8
        );
    }

    #[cfg(all(
        target_arch = "aarch64",
        target_feature = "neon",
        not(feature = "portable")
    ))]
    pub(super) use neon::{from_morton_u32, from_morton_u64};

    /// No batch path: the caller converts every key.
    #[cfg(not(any(
        all(target_arch = "x86_64", not(feature = "portable")),
        all(
            target_arch = "aarch64",
            target_feature = "neon",
            not(feature = "portable")
        )
    )))]
    pub(super) const fn from_morton_u32(_: &mut [u32]) -> usize {
        0
    }

    #[cfg(not(any(
        all(target_arch = "x86_64", not(feature = "portable")),
        all(
            target_arch = "aarch64",
            target_feature = "neon",
            not(feature = "portable")
        )
    )))]
    pub(super) const fn from_morton_u64(_: &mut [u64]) -> usize {
        0
    }
}

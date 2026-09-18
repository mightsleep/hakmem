//! Dilated integers and Morton (Z-order) codes.
//!
//! A *dilated* integer keeps its bits at every `D`-th position, zeros
//! between. Interleaving `D` of them gives a Morton code, the address
//! space where a `2^n × 2^n` grid's aligned power-of-two blocks are
//! contiguous ranges, the reason a 1D allocator can run over a 2D
//! texture.
//!
//! Arithmetic on dilated values does not go through `+`: the gaps must
//! be filled with ones so the carry tunnels across them
//! (`((a | !mask) + b) & mask`, Wise & Raman). [`Dilated`] therefore
//! has no `Add` impl, only [`incr`](Dilated::incr) / [`wrapping_add`](Dilated::wrapping_add)
//! Plain integer addition on it does not compile.

use crate::word::Word;

/// An integer whose bits sit at positions `0, D, 2D, …` of `W`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Dilated<W: Word, const D: u32>(W);

impl<W: Word, const D: u32> Dilated<W, D> {
    /// The positions this dilation occupies: bits `0, D, 2D, …`.
    #[inline]
    #[must_use]
    pub fn mask() -> W {
        let mut m = W::ZERO;
        let mut i = 0;
        while i < W::BITS {
            m = m.or(W::ONE.shl(i));
            i += D;
        }
        m
    }

    /// Number of integer bits that fit: `ceil(BITS / D)`.
    #[inline]
    #[must_use]
    pub const fn width() -> u32 {
        W::BITS.div_ceil(D)
    }

    /// Dilates a plain integer. Bits above [`width`](Self::width) are
    /// dropped.
    #[inline]
    #[must_use]
    pub fn from_int(x: W) -> Self {
        Self(x.pdep(Self::mask()))
    }

    /// Undoes [`from_int`](Self::from_int).
    #[inline]
    #[must_use]
    pub fn into_int(self) -> W {
        self.0.pext(Self::mask())
    }

    /// Wraps an already-dilated bit pattern. Bits outside the mask must
    /// be zero (debug-asserted).
    #[inline]
    #[must_use]
    pub fn from_bits(bits: W) -> Self {
        debug_assert!(
            bits.and(Self::mask().not()).is_zero(),
            "bits outside dilation mask"
        );
        Self(bits)
    }

    /// The raw dilated bit pattern.
    #[inline]
    #[must_use]
    pub const fn bits(self) -> W {
        self.0
    }

    /// `self + 1` in dilated space, wrapping at [`width`](Self::width)
    /// bits: fill the gaps with ones so the carry tunnels through them.
    #[inline]
    #[must_use]
    pub fn incr(self) -> Self {
        let m = Self::mask();
        Self(self.0.or(m.not()).wrapping_add(W::ONE).and(m))
    }

    /// `self - 1` in dilated space, wrapping: the borrow tunnels through
    /// the zero gaps on its own.
    #[inline]
    #[must_use]
    pub fn decr(self) -> Self {
        Self(self.0.wrapping_sub(W::ONE).and(Self::mask()))
    }

    /// `self + other` in dilated space, wrapping.
    #[inline]
    #[must_use]
    pub fn wrapping_add(self, other: Self) -> Self {
        let m = Self::mask();
        Self(self.0.or(m.not()).wrapping_add(other.0).and(m))
    }
}

/// A 2D Morton code: `x` dilated by 2 in the even bits, `y` in the odd.
///
/// ```
/// use hakmem::prelude::*;
///
/// let m = Morton2::<u32>::encode(3, 5);
/// assert_eq!(m.decode(), (3, 5));
/// assert_eq!(m.step_x().decode(), (4, 5));
/// assert_eq!(m.step_y().decode(), (3, 6));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Morton2<W: Word>(W);

impl<W: Word> Morton2<W> {
    /// Interleaves `x` (even bits) and `y` (odd bits). Coordinates
    /// above `BITS / 2` bits are dropped.
    #[inline]
    #[must_use]
    pub fn encode(x: W, y: W) -> Self {
        let xd = Dilated::<W, 2>::from_int(x).bits();
        let yd = Dilated::<W, 2>::from_int(y).bits();
        Self(xd.or(yd.shl(1)))
    }

    /// Splits the code back into `(x, y)`.
    #[inline]
    #[must_use]
    pub fn decode(self) -> (W, W) {
        (self.x().into_int(), self.y().into_int())
    }

    /// Wraps an existing code.
    #[inline]
    #[must_use]
    pub const fn from_code(code: W) -> Self {
        Self(code)
    }

    /// The raw code.
    #[inline]
    #[must_use]
    pub const fn code(self) -> W {
        self.0
    }

    /// The `x` coordinate, still dilated (for arithmetic in place).
    #[inline]
    #[must_use]
    pub fn x(self) -> Dilated<W, 2> {
        Dilated::from_bits(self.0.and(Dilated::<W, 2>::mask()))
    }

    /// The `y` coordinate, still dilated.
    #[inline]
    #[must_use]
    pub fn y(self) -> Dilated<W, 2> {
        Dilated::from_bits(self.0.shr(1).and(Dilated::<W, 2>::mask()))
    }

    /// Replaces the `x` coordinate.
    #[inline]
    #[must_use]
    pub fn with_x(self, x: Dilated<W, 2>) -> Self {
        Self(self.0.and(Dilated::<W, 2>::mask().not()).or(x.bits()))
    }

    /// Replaces the `y` coordinate.
    #[inline]
    #[must_use]
    pub fn with_y(self, y: Dilated<W, 2>) -> Self {
        Self(self.0.and(Dilated::<W, 2>::mask()).or(y.bits().shl(1)))
    }

    /// The code of `(x + 1, y)`, wrapping.
    #[inline]
    #[must_use]
    pub fn step_x(self) -> Self {
        self.with_x(self.x().incr())
    }

    /// The code of `(x, y + 1)`, wrapping.
    #[inline]
    #[must_use]
    pub fn step_y(self) -> Self {
        self.with_y(self.y().incr())
    }
}

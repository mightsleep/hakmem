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

use crate::cover::{Masks, Quadrants, Rect};
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
    /// Levels: `BITS / 2`.
    pub const LEVELS: u32 = W::BITS / 2;

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

/// A 3D Morton code: `x` dilated by 3 in bits `0, 3, 6, …`, `y` in
/// `1, 4, 7, …`, `z` in `2, 5, 8, …`.
///
/// `BITS / 3` levels; when `BITS` is not a multiple of three the top
/// one or two bits are unused, and coordinates above `BITS / 3` bits
/// are dropped.
///
/// ```
/// use hakmem::prelude::*;
///
/// let m = Morton3::<u32>::encode(3, 5, 6);
/// assert_eq!(m.decode(), (3, 5, 6));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Morton3<W: Word>(W);

impl<W: Word> Morton3<W> {
    /// Levels: `BITS / 3`.
    pub const LEVELS: u32 = W::BITS / 3;

    /// Interleaves three coordinates. Bits above `LEVELS` are dropped.
    #[inline]
    #[must_use]
    pub fn encode(x: W, y: W, z: W) -> Self {
        let side = W::low_ones(Self::LEVELS);
        let d = |v: W| Dilated::<W, 3>::from_int(v.and(side)).bits();
        Self(d(x).or(d(y).shl(1)).or(d(z).shl(2)))
    }

    /// Splits the code back into `(x, y, z)`.
    #[inline]
    #[must_use]
    pub fn decode(self) -> (W, W, W) {
        let lane = Dilated::<W, 3>::mask();
        let u = |v: W| Dilated::<W, 3>::from_bits(v.and(lane)).into_int();
        (u(self.0), u(self.0.shr(1)), u(self.0.shr(2)))
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
}

macro_rules! morton2_columns {
    ($($w:ty => $encode:ident, $decode:ident),* $(,)?) => {$(
        impl Morton2<$w> {
            /// [`encode`](Self::encode) over two columns of coordinates:
            /// `out[i]` is the code of `(xs[i], ys[i])`. Coordinates above
            /// `BITS / 2` bits are dropped, as in `encode`.
            ///
            /// On `x86_64` with AVX2, chosen at run time, 32 bytes of
            /// points a step without PDEP: the coordinate's bytes widen
            /// to 16 bits, each nibble lands in a byte, and one PSHUFB
            /// through a 16-entry table spreads it over the even bits
            /// (the odd ones for `y`). Otherwise, and past the last
            /// batch, `encode` per point.
            ///
            /// # Panics
            ///
            /// If the three slices differ in length.
            pub fn encode_columns(xs: &[$w], ys: &[$w], out: &mut [$w]) {
                let n = out.len();
                assert!(
                    xs.len() == n && ys.len() == n,
                    "columns of {} and {} for {n} codes",
                    xs.len(),
                    ys.len()
                );
                let done = batch::$encode(xs, ys, out);
                for ((o, &x), &y) in out[done..].iter_mut().zip(&xs[done..]).zip(&ys[done..]) {
                    *o = Self::encode(x, y).code();
                }
            }

            /// [`decode`](Self::decode) of a column of codes into two
            /// columns of coordinates, the inverse of
            /// [`encode_columns`](Self::encode_columns). With AVX2 two
            /// PSHUFB per axis compact a nibble's two bits of it, and a
            /// shift inside 16 bits and a PSHUFB pack the nibbles back
            /// into bytes.
            ///
            /// # Panics
            ///
            /// If the three slices differ in length.
            pub fn decode_columns(codes: &[$w], xs: &mut [$w], ys: &mut [$w]) {
                let n = codes.len();
                assert!(
                    xs.len() == n && ys.len() == n,
                    "columns of {} and {} for {n} codes",
                    xs.len(),
                    ys.len()
                );
                let done = batch::$decode(codes, xs, ys);
                for ((&c, x), y) in codes[done..].iter().zip(&mut xs[done..]).zip(&mut ys[done..]) {
                    (*x, *y) = Self::from_code(c).decode();
                }
            }
        }
    )*};
}

/// Z-order for [`crate::cover`]: digit `d` is `x` in bit 0, `y` in bit
/// 1, one frame.
pub(crate) struct ZQuadrants;

static Z_MASKS: Masks<1> = Masks::build([[(0, 0), (1, 0), (2, 0), (3, 0)]]);

impl Quadrants for ZQuadrants {
    type Context<W> = ();

    #[inline]
    fn digit(_: u8, dx: u8, dy: u8) -> (u8, u8) {
        (dx | dy << 1, 0)
    }

    #[inline]
    fn child(k: u32, _: u8, p: u32) -> (u8, u8, u8) {
        Z_MASKS.child(k, 0, p)
    }

    #[inline]
    fn cells(k: u32, _: u8, cols: usize, rows: usize) -> u64 {
        Z_MASKS.cells(k, 0, cols, rows)
    }

    fn context<W: Word + Ord>(_: &Rect<W>) {}

    fn runs<W: Word + Ord>(levels: u32, r: &Rect<W>, s: u32, (): &()) -> W {
        z_runs(
            (r.x0.shr(s), r.x1.shr(s)),
            (r.y0.shr(s), r.y1.shr(s)),
            levels - s,
        )
    }
}

/// The runs of the exact Z-order cover of `x0..=x1` × `y0..=y1` on
/// `levels` levels, without a node: a run starts at a cell whose
/// predecessor is outside, and the predecessor of a code is a borrow.
/// When the lowest set bit is bit `t` of `x`, the low `t` bits of both
/// are zero and the borrow gives `(x - 1, y | low(t))`; when it is bit
/// `t` of `y`, `(x | low(t + 1), y - 1)`. The predecessor then leaves by
/// the near side (`x = x0`, or `y = y0`), or by the far side of the other
/// axis, which only the last aligned block before `x1` (or `y1`) can do.
/// Each is a count of an arithmetic progression, two per level.
fn z_runs<W: Word + Ord>((x0, x1): (W, W), (y0, y1): (W, W), levels: u32) -> W {
    // `v` in `a..=b` with `v ≡ r` mod `2^m`.
    let progression = |(a, b): (W, W), r: W, m: u32| {
        let upto = |v: W| {
            if v < r {
                W::ZERO
            } else {
                v.wrapping_sub(r).shr(m).wrapping_add(W::ONE)
            }
        };
        let before = if a.is_zero() {
            W::ZERO
        } else {
            upto(a.wrapping_sub(W::ONE))
        };
        upto(b).wrapping_sub(before)
    };
    // Whether the last multiple of `2^m` in `a..=b` sticks out past `b`
    // once its low `m` bits are filled.
    let spills = |(a, b): (W, W), m: u32| {
        let low = W::low_ones(m);
        b.and(low.not()) >= a && b.and(low) != low
    };
    let one = |c: bool| if c { W::ONE } else { W::ZERO };
    let mut n = one(x0.is_zero() && y0.is_zero());
    for t in 0..levels {
        let half = W::ONE.shl(t);
        // Lowest set bit in x, at t: the columns with that bit, times
        // the rows with none below it.
        let near = x0.trailing_zeros() == t;
        let cols = progression((x0, x1), half, t + 1).wrapping_sub(one(near));
        let rows = progression((y0, y1), W::ZERO, t);
        n = n.wrapping_add(if near { rows } else { W::ZERO });
        n = n.wrapping_add(if spills((y0, y1), t) { cols } else { W::ZERO });
        // Lowest set bit in y, at t.
        let near = y0.trailing_zeros() == t;
        let rows = progression((y0, y1), half, t + 1).wrapping_sub(one(near));
        let cols = progression((x0, x1), W::ZERO, t + 1);
        n = n.wrapping_add(if near { cols } else { W::ZERO });
        n = n.wrapping_add(if spills((x0, x1), t + 1) {
            rows
        } else {
            W::ZERO
        });
    }
    n
}

impl<W: Word + Ord> Morton2<W> {
    /// The codes of the rectangle `x.0..=x.1` × `y.0..=y.1` as at most
    /// `out.len()` ranges; returns how many it wrote. Inclusive, sorted,
    /// disjoint, not touching, covering every cell, exact when the
    /// budget allows, as [`Hilbert2::cover`](crate::hilbert::Hilbert2::cover).
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// // Two columns of four cells: two quadrants, two ranges.
    /// let mut out = [(0u8, 0u8); 8];
    /// let n = Morton2::<u8>::cover((0, 1), (0, 3), &mut out);
    /// assert_eq!(&out[..n], &[(0, 3), (8, 11)]);
    /// ```
    ///
    /// # Panics
    ///
    /// If the rectangle is not empty and `out` is.
    #[must_use = "only `out[..n]` holds ranges; the rest is scratch"]
    pub fn cover(x: (W, W), y: (W, W), out: &mut [(W, W)]) -> usize {
        crate::cover::cover::<W, ZQuadrants>(Self::LEVELS, x, y, out)
    }

    /// Whether some cell of the rectangle has its code in
    /// `keys.0..=keys.1`, as [`Hilbert2::intersects`](crate::hilbert::Hilbert2::intersects).
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// // Codes 4..=11 miss the 2 × 2 block at the origin, codes 0..=3.
    /// assert!(!Morton2::<u8>::intersects((4, 11), (0, 1), (0, 1)));
    /// assert!(Morton2::<u8>::intersects((4, 11), (0, 2), (0, 1)));
    /// ```
    #[must_use]
    pub fn intersects(keys: (W, W), x: (W, W), y: (W, W)) -> bool {
        crate::cover::intersects::<W, ZQuadrants>(Self::LEVELS, keys, x, y)
    }
}

morton2_columns!(
    u32 => encode_u32, decode_u32,
    u64 => encode_u64, decode_u64,
);

/// The Morton batch kernels; each returns how many points from the front
/// it converted, a whole number of batches.
mod batch {
    #[cfg(all(target_arch = "x86_64", not(feature = "portable")))]
    // `inline(always)` on the table load: a helper without the kernel's
    // `target_feature` must be inlined into it for the intrinsics to be.
    #[allow(unsafe_code, clippy::inline_always)]
    mod avx2 {
        use core::arch::x86_64::{
            __m256i, _mm_loadu_si128, _mm256_and_si256, _mm256_broadcastsi128_si256,
            _mm256_loadu_si256, _mm256_or_si256, _mm256_set1_epi8, _mm256_shuffle_epi8,
            _mm256_slli_epi64, _mm256_srli_epi16, _mm256_srli_epi64, _mm256_storeu_si256,
        };

        /// PSHUFB's zero: bit 7 of the index.
        const Z: u8 = 0x80;

        /// A nibble's four bits at the even bits of a byte, or the odd.
        const fn spread(odd: bool) -> [u8; 16] {
            let mut t = [0u8; 16];
            let mut n = 0;
            while n < 16 {
                let s = (n & 1) | (n & 2) << 1 | (n & 4) << 2 | (n & 8) << 3;
                t[n as usize] = if odd { s << 1 } else { s };
                n += 1;
            }
            t
        }

        /// The two even (or odd) bits of a nibble, packed, at bits 0 and
        /// 1 of the result (`high`: at 2 and 3).
        const fn compact(odd: bool, high: bool) -> [u8; 16] {
            let mut t = [0u8; 16];
            let mut n = 0u8;
            while n < 16 {
                let m = if odd { n >> 1 } else { n };
                let c = (m & 1) | (m >> 1 & 2);
                t[n as usize] = if high { c << 2 } else { c };
                n += 1;
            }
            t
        }

        const SPREAD_X: [u8; 16] = spread(false);
        const SPREAD_Y: [u8; 16] = spread(true);
        const X_LO: [u8; 16] = compact(false, false);
        const X_HI: [u8; 16] = compact(false, true);
        const Y_LO: [u8; 16] = compact(true, false);
        const Y_HI: [u8; 16] = compact(true, true);

        /// A 16-byte table in both halves of a register.
        #[inline(always)]
        unsafe fn table(t: &[u8; 16]) -> __m256i {
            // SAFETY: AVX2 on the caller; the load reads the 16 bytes.
            unsafe { _mm256_broadcastsi128_si256(_mm_loadu_si128(t.as_ptr().cast())) }
        }

        macro_rules! kernels {
            ($encode:ident, $decode:ident, $w:ty, $widen:expr, $gather:expr) => {
                /// Batches of four registers.
                #[target_feature(enable = "avx2")]
                pub(in crate::dilated) fn $encode(xs: &[$w], ys: &[$w], out: &mut [$w]) -> usize {
                    const PER: usize = 32 / size_of::<$w>();
                    let (out, _) = out.as_chunks_mut::<{ 4 * PER }>();
                    let (xs, _) = xs.as_chunks::<{ 4 * PER }>();
                    let (ys, _) = ys.as_chunks::<{ 4 * PER }>();
                    let done = out.len() * 4 * PER;
                    // SAFETY: AVX2 is enabled on this function; every load
                    // and store stays inside a chunk or a 16-byte table.
                    unsafe {
                        let widen = table(&$widen);
                        let (sx, sy) = (table(&SPREAD_X), table(&SPREAD_Y));
                        let low = _mm256_set1_epi8(0x0F);
                        // Byte `i` of the coordinate to byte `2i`, then its
                        // high nibble to byte `2i + 1`: a nibble a byte.
                        let nibbles = |v: __m256i| {
                            let w = _mm256_shuffle_epi8(v, widen);
                            _mm256_and_si256(_mm256_or_si256(w, _mm256_slli_epi64::<4>(w)), low)
                        };
                        for ((o, x), y) in out.iter_mut().zip(xs).zip(ys) {
                            for r in 0..4 {
                                let xv = _mm256_loadu_si256(x.as_ptr().add(PER * r).cast());
                                let yv = _mm256_loadu_si256(y.as_ptr().add(PER * r).cast());
                                let code = _mm256_or_si256(
                                    _mm256_shuffle_epi8(sx, nibbles(xv)),
                                    _mm256_shuffle_epi8(sy, nibbles(yv)),
                                );
                                _mm256_storeu_si256(o.as_mut_ptr().add(PER * r).cast(), code);
                            }
                        }
                    }
                    done
                }

                /// Batches of four registers.
                #[target_feature(enable = "avx2")]
                pub(in crate::dilated) fn $decode(
                    codes: &[$w],
                    xs: &mut [$w],
                    ys: &mut [$w],
                ) -> usize {
                    const PER: usize = 32 / size_of::<$w>();
                    let (codes, _) = codes.as_chunks::<{ 4 * PER }>();
                    let (xs, _) = xs.as_chunks_mut::<{ 4 * PER }>();
                    let (ys, _) = ys.as_chunks_mut::<{ 4 * PER }>();
                    let done = codes.len() * 4 * PER;
                    // SAFETY: as in the encode.
                    unsafe {
                        let gather = table(&$gather);
                        let (xl, xh) = (table(&X_LO), table(&X_HI));
                        let (yl, yh) = (table(&Y_LO), table(&Y_HI));
                        let low = _mm256_set1_epi8(0x0F);
                        // Nibbles `2i` and `2i + 1` into byte `i`: within a
                        // 16-bit word `w | w >> 4`, then the even bytes.
                        let pack = |n: __m256i| {
                            _mm256_shuffle_epi8(
                                _mm256_or_si256(n, _mm256_srli_epi16::<4>(n)),
                                gather,
                            )
                        };
                        for ((c, x), y) in codes.iter().zip(xs).zip(ys) {
                            for r in 0..4 {
                                let v = _mm256_loadu_si256(c.as_ptr().add(PER * r).cast());
                                let lo = _mm256_and_si256(v, low);
                                let hi = _mm256_and_si256(_mm256_srli_epi64::<4>(v), low);
                                let xn = _mm256_or_si256(
                                    _mm256_shuffle_epi8(xl, lo),
                                    _mm256_shuffle_epi8(xh, hi),
                                );
                                let yn = _mm256_or_si256(
                                    _mm256_shuffle_epi8(yl, lo),
                                    _mm256_shuffle_epi8(yh, hi),
                                );
                                _mm256_storeu_si256(x.as_mut_ptr().add(PER * r).cast(), pack(xn));
                                _mm256_storeu_si256(y.as_mut_ptr().add(PER * r).cast(), pack(yn));
                            }
                        }
                    }
                    done
                }
            };
        }

        kernels!(
            encode_u64,
            decode_u64,
            u64,
            [0, Z, 1, Z, 2, Z, 3, Z, 8, Z, 9, Z, 10, Z, 11, Z],
            [0, 2, 4, 6, Z, Z, Z, Z, 8, 10, 12, 14, Z, Z, Z, Z]
        );
        kernels!(
            encode_u32,
            decode_u32,
            u32,
            [0, Z, 1, Z, 4, Z, 5, Z, 8, Z, 9, Z, 12, Z, 13, Z],
            [0, 2, Z, Z, 4, 6, Z, Z, 8, 10, Z, Z, 12, 14, Z, Z]
        );
    }

    /// `x86_64` chooses once a call, as the Hilbert batches do.
    #[cfg(all(target_arch = "x86_64", not(feature = "portable")))]
    #[allow(unsafe_code)]
    mod dispatch {
        use super::avx2;
        use crate::cpu;

        macro_rules! pick {
            ($($name:ident($a:ident: $ta:ty, $b:ident: $tb:ty, $c:ident: $tc:ty);)*) => {$(
                pub(in crate::dilated) fn $name($a: $ta, $b: $tb, $c: $tc) -> usize {
                    if cpu::avx2() {
                        // SAFETY: AVX2 is present.
                        unsafe { avx2::$name($a, $b, $c) }
                    } else {
                        0
                    }
                }
            )*};
        }

        pick! {
            encode_u32(xs: &[u32], ys: &[u32], out: &mut [u32]);
            encode_u64(xs: &[u64], ys: &[u64], out: &mut [u64]);
            decode_u32(codes: &[u32], xs: &mut [u32], ys: &mut [u32]);
            decode_u64(codes: &[u64], xs: &mut [u64], ys: &mut [u64]);
        }
    }

    #[cfg(all(target_arch = "x86_64", not(feature = "portable")))]
    pub(super) use dispatch::{decode_u32, decode_u64, encode_u32, encode_u64};

    /// No batch path: the caller converts every point.
    #[cfg(not(all(target_arch = "x86_64", not(feature = "portable"))))]
    mod none {
        pub(in crate::dilated) const fn encode_u32(_: &[u32], _: &[u32], _: &mut [u32]) -> usize {
            0
        }
        pub(in crate::dilated) const fn encode_u64(_: &[u64], _: &[u64], _: &mut [u64]) -> usize {
            0
        }
        pub(in crate::dilated) const fn decode_u32(
            _: &[u32],
            _: &mut [u32],
            _: &mut [u32],
        ) -> usize {
            0
        }
        pub(in crate::dilated) const fn decode_u64(
            _: &[u64],
            _: &mut [u64],
            _: &mut [u64],
        ) -> usize {
            0
        }
    }

    #[cfg(not(all(target_arch = "x86_64", not(feature = "portable"))))]
    pub(super) use none::{decode_u32, decode_u64, encode_u32, encode_u64};
}

//! Byte lanes: the SIMD half of the algebra, on stable Rust.
//!
//! [`Word`] treats a register as one number, so a carry runs the whole
//! width. [`Lanes`] treats it as `LANES` independent bytes: nothing
//! crosses a lane. The two meet at [`Lanes::to_bits`], which turns a
//! lane mask into a [`Word`] with one bit per lane; from there the carry
//! tricks of [`Bits`] take over. That boundary is where simdjson's first
//! stage hands its masks to its second.
//!
//! Two carriers. [`U8x8`] is eight lanes in a `u64` by SWAR, always
//! available and the one the exhaustive tests run on. [`U8x16`] is
//! sixteen lanes in a vector register, SSSE3 on x86-64 and NEON on
//! aarch64, chosen at compile time by `target_feature` like every other
//! hardware path in this crate, and two `U8x8` halves otherwise. The
//! intrinsic calls are the crate's only `unsafe`, as in [`Word`]: sound
//! because the `cfg` makes the feature a compile-time fact.
//!
//! The one lane instruction that has no scalar sibling is the 16-entry
//! table lookup ([`Lanes::lut16`], PSHUFB / `tbl`): two of them, one on
//! each nibble, classify a byte into any set in three operations.
//! simdjson's structural and whitespace masks are exactly that:
//!
//! ```
//! use hakmem::lanes::{Lanes, U8x16};
//!
//! // Bit per high nibble on the right, per low nibble on the left; a
//! // byte is in the set iff the two tables share a bit. Bits 0..4 mark
//! // the structural characters `{ } [ ] : ,`, bits 4..6 whitespace.
//! const LO: [u8; 16] = [16, 0, 0, 0, 0, 0, 0, 0, 0, 32, 34, 12, 1, 44, 0, 0];
//! const HI: [u8; 16] = [32, 0, 17, 2, 0, 4, 0, 8, 0, 0, 0, 0, 0, 0, 0, 0];
//!
//! fn classify(block: U8x16) -> (u16, u16) {
//!     let class = block.lut16_nibbles(LO, HI);
//!     let structural = class.and(U8x16::splat(15)).cmp_eq(U8x16::zero()).not();
//!     let whitespace = class.and(U8x16::splat(48)).cmp_eq(U8x16::zero()).not();
//!     (structural.to_bits(), whitespace.to_bits())
//! }
//!
//! let (structural, whitespace) = classify(U8x16::load(b"{\"a\": [1, 2]}   "));
//! assert_eq!(structural, 0b0001_1001_0101_0001);
//! assert_eq!(whitespace, 0b1110_0010_0010_0000);
//!
//! // The tables are right for every byte, not just these sixteen.
//! for c in 0..=255u8 {
//!     let (s, w) = classify(U8x16::splat(c));
//!     assert_eq!(s == u16::MAX, b"{}[]:,".contains(&c), "{c:#04x}");
//!     assert_eq!(w == u16::MAX, b" \t\n\r".contains(&c), "{c:#04x}");
//! }
//! ```
//!
//! The second lane instruction without a scalar sibling is GFNI's
//! `gf2p8affineqb`, an 8×8 bit matrix applied to every byte:
//! [`Lanes::affine`] with an [`Affine8`], the map that every byte
//! shift, rotate, reversal, and any composition of them, is an instance
//! of. Without GFNI the same map is two nibble lookups by linearity,
//! which is why `lut16` is the primitive and `affine` is provided.

use crate::affine::Affine8;
use crate::bits::Bits;
use crate::word::Word;

/// The provided methods' scratch, checked against [`Lanes::LANES`] at
/// compile time.
const fn scratch<L: Lanes>() -> [u8; 64] {
    const { assert!(L::LANES <= 64, "Lanes: more than 64 lanes") };
    [0; 64]
}

/// `LANES` independent 8-bit lanes in one register.
///
/// Required methods are the instructions a carrier has; the provided
/// ones are compositions with the same laws everywhere. Compares yield
/// masks, `0xFF` or `0x00` per lane, so they chain with `and` / `or` /
/// `not` and fold into a [`Word`] with [`to_bits`](Self::to_bits).
pub trait Lanes: Copy + Eq + core::fmt::Debug {
    /// Number of lanes, at most 64: the provided methods work in a
    /// 64-byte scratch. AVX-512 fits, and so does a compile error.
    const LANES: usize;
    /// The word [`to_bits`](Self::to_bits) produces: one bit per lane.
    type Bits: Word;

    /// `b` in every lane.
    #[must_use]
    fn splat(b: u8) -> Self;
    /// Lanes from `LANES` bytes, lane 0 first.
    ///
    /// # Panics
    ///
    /// If `bytes.len() != LANES`.
    #[must_use]
    fn load(bytes: &[u8]) -> Self;
    /// Lane `i`, for `i < LANES`.
    #[must_use]
    fn lane(self, i: usize) -> u8;

    /// Bitwise AND.
    #[must_use]
    fn and(self, other: Self) -> Self;
    /// Bitwise OR.
    #[must_use]
    fn or(self, other: Self) -> Self;
    /// Bitwise XOR.
    #[must_use]
    fn xor(self, other: Self) -> Self;
    /// Bitwise NOT.
    #[must_use]
    fn not(self) -> Self;
    /// Wrapping add per lane; no carry leaves a lane.
    #[must_use]
    fn add(self, other: Self) -> Self;
    /// Wrapping subtract per lane; no borrow leaves a lane.
    #[must_use]
    fn sub(self, other: Self) -> Self;
    /// Shift every lane left by `n`; `n >= 8` clears every lane.
    #[must_use]
    fn shl(self, n: u32) -> Self;
    /// Shift every lane right by `n`; `n >= 8` clears every lane.
    #[must_use]
    fn shr(self, n: u32) -> Self;

    /// Mask of lanes where `self == other`.
    #[must_use]
    fn cmp_eq(self, other: Self) -> Self;
    /// Mask of lanes where `self <= other`, unsigned.
    #[must_use]
    fn cmp_le(self, other: Self) -> Self;

    /// Table lookup with PSHUFB's contract: a lane with its top bit
    /// clear becomes `table[lane & 15]`, a lane with its top bit set
    /// becomes zero.
    #[must_use]
    fn lut16(self, table: [u8; 16]) -> Self;

    /// One bit per lane, bit `i` = the top bit of lane `i`. On a compare
    /// mask this is the set of lanes that matched.
    #[must_use]
    fn to_bits(self) -> Self::Bits;

    /// Every lane zero.
    #[inline]
    #[must_use]
    fn zero() -> Self {
        Self::splat(0)
    }

    /// Mask of lanes where `self < other`, unsigned.
    #[inline]
    #[must_use]
    fn cmp_lt(self, other: Self) -> Self {
        self.cmp_le(other).and(self.cmp_eq(other).not())
    }

    /// Mask of lanes where `self >= other`, unsigned.
    #[inline]
    #[must_use]
    fn cmp_ge(self, other: Self) -> Self {
        other.cmp_le(self)
    }

    /// Mask of lanes where `self > other`, unsigned.
    #[inline]
    #[must_use]
    fn cmp_gt(self, other: Self) -> Self {
        other.cmp_lt(self)
    }

    /// Per lane, `other` where `mask` is set and `self` elsewhere.
    #[inline]
    #[must_use]
    fn blend(self, other: Self, mask: Self) -> Self {
        mask.and(other).or(mask.not().and(self))
    }

    /// Unsigned minimum per lane.
    #[inline]
    #[must_use]
    fn min(self, other: Self) -> Self {
        other.blend(self, self.cmp_le(other))
    }

    /// Unsigned maximum per lane.
    #[inline]
    #[must_use]
    fn max(self, other: Self) -> Self {
        self.blend(other, self.cmp_le(other))
    }

    /// The low nibble of every lane looked up in `lo`, the high nibble
    /// in `hi`, combined with AND: the byte classifier of the module docs.
    #[inline]
    #[must_use]
    fn lut16_nibbles(self, lo: [u8; 16], hi: [u8; 16]) -> Self {
        self.and(Self::splat(0x0F))
            .lut16(lo)
            .and(self.shr(4).lut16(hi))
    }

    // --- the five lane-only primitives, portable definitions ----------

    /// Byte permute by data, PSHUFB's contract with `self` as the table:
    /// lane `i` becomes `self[idx_i mod LANES]`, or zero where `idx_i` has
    /// its top bit set. [`lut16`](Self::lut16) is this with a constant
    /// table; this is the same instruction with the table in a register.
    #[inline]
    #[must_use]
    fn shuffle(self, idx: Self) -> Self {
        let mut out = scratch::<Self>();
        for (i, o) in out.iter_mut().enumerate().take(Self::LANES) {
            let j = idx.lane(i);
            *o = if j & 0x80 == 0 {
                self.lane(usize::from(j) % Self::LANES)
            } else {
                0
            };
        }
        Self::load(&out[..Self::LANES])
    }

    /// Lanes `n..n + LANES` of the concatenation `self ++ other`, zeros
    /// past its end: PALIGNR. `n = 0` is `self`, `n = LANES` is `other`,
    /// `n >= 2 LANES` is zero. A window sliding across two registers.
    #[inline]
    #[must_use]
    fn concat_shift(self, other: Self, n: usize) -> Self {
        let mut out = scratch::<Self>();
        for (i, o) in out.iter_mut().enumerate().take(Self::LANES) {
            let j = i + n;
            *o = if j < Self::LANES {
                self.lane(j)
            } else if j < 2 * Self::LANES {
                other.lane(j - Self::LANES)
            } else {
                0
            };
        }
        Self::load(&out[..Self::LANES])
    }

    /// Saturating add per lane: `min(x + y, 255)`.
    #[inline]
    #[must_use]
    fn add_sat(self, other: Self) -> Self {
        self.add(other.min(self.not()))
    }

    /// Saturating subtract per lane: `max(x - y, 0)`.
    #[inline]
    #[must_use]
    fn sub_sat(self, other: Self) -> Self {
        self.sub(self.min(other))
    }

    /// Interleave the low halves: `self[0], other[0], self[1], other[1], ...`
    /// (PUNPCKLBW, `zip1`). With [`unpack_hi`](Self::unpack_hi) a
    /// transpose in `log₂ LANES` rounds, and the widening of bytes to
    /// 16-bit lanes when `other` is zero.
    #[inline]
    #[must_use]
    fn unpack_lo(self, other: Self) -> Self {
        let mut out = scratch::<Self>();
        for k in 0..Self::LANES / 2 {
            out[2 * k] = self.lane(k);
            out[2 * k + 1] = other.lane(k);
        }
        Self::load(&out[..Self::LANES])
    }

    /// Interleave the high halves: `self[LANES/2], other[LANES/2], ...`
    /// (PUNPCKHBW, `zip2`).
    #[inline]
    #[must_use]
    fn unpack_hi(self, other: Self) -> Self {
        let mut out = scratch::<Self>();
        let half = Self::LANES / 2;
        for k in 0..half {
            out[2 * k] = self.lane(half + k);
            out[2 * k + 1] = other.lane(half + k);
        }
        Self::load(&out[..Self::LANES])
    }

    /// Sum over all lanes of `|self - other|` (PSADBW, `uabd` + `addv`):
    /// the horizontal reduce. Against zero it sums the lanes, which is
    /// how a nibble-table popcount is finished.
    #[inline]
    #[must_use]
    fn sum_abs_diff(self, other: Self) -> u32 {
        let d = self.max(other).sub(self.min(other));
        (0..Self::LANES).map(|i| u32::from(d.lane(i))).sum()
    }

    /// PMADDUBSW: for every pair of lanes `k`, the dot product
    /// `self[2k] * w[2k] + self[2k + 1] * w[2k + 1]` with `self` unsigned
    /// and `w` signed, saturated to `i16` and stored little-endian in the
    /// two lanes of the pair. The result is `LANES / 2` sixteen-bit lanes
    /// in the same register, as the instruction leaves them; the parser of
    /// eight decimal digits in three multiplies starts here.
    #[inline]
    #[must_use]
    fn mul_add_pairs(self, weights: Self) -> Self {
        let mut out = scratch::<Self>();
        for k in 0..Self::LANES / 2 {
            let (a0, a1) = (i32::from(self.lane(2 * k)), i32::from(self.lane(2 * k + 1)));
            let (w0, w1) = (
                i32::from(weights.lane(2 * k).cast_signed()),
                i32::from(weights.lane(2 * k + 1).cast_signed()),
            );
            // Saturation is the instruction's, and `i16::try_from` is exact.
            let s = i16::try_from(a0 * w0 + a1 * w1).unwrap_or(if a0 * w0 + a1 * w1 < 0 {
                i16::MIN
            } else {
                i16::MAX
            });
            out[2 * k..2 * k + 2].copy_from_slice(&s.to_le_bytes());
        }
        Self::load(&out[..Self::LANES])
    }

    // --- byte maps: GF(2) affine, and the shapes built on it ------------

    /// Apply an [`Affine8`] map to every lane: `gf2p8affineqb`, one
    /// instruction with GFNI. Without it, linearity splits the map into
    /// two nibble lookups (`A·x = A·hi ⊕ A·lo`), so two
    /// [`lut16`](Self::lut16) and an XOR, the tables folded at compile
    /// time whenever the map is a constant; the SWAR carrier folds
    /// parities instead ([`Affine8::apply8`]).
    #[inline]
    #[must_use]
    fn affine(self, map: Affine8) -> Self {
        let (lo, hi) = map.tables();
        self.and(Self::splat(0x0F))
            .lut16(lo)
            .xor(self.shr(4).lut16(hi))
    }

    /// Reverse the bits of every lane: [`Affine8::REVERSE`]; `rbit` on
    /// NEON.
    #[inline]
    #[must_use]
    fn reverse_bits(self) -> Self {
        self.affine(Affine8::REVERSE)
    }

    /// Arithmetic shift right per lane: the top bit fills the vacated
    /// bits, so `n >= 8` leaves every lane `0x00` or `0xFF`. Without a
    /// signed byte shift (SSE) or GFNI: the logical shift, OR the
    /// sign mask shifted into place.
    #[inline]
    #[must_use]
    fn sra(self, n: u32) -> Self {
        let n = n.min(8);
        let sign = self.cmp_ge(Self::splat(0x80));
        self.shr(n).or(sign.shl(8 - n))
    }

    /// Rotate every lane left by `n mod 8`.
    #[inline]
    #[must_use]
    fn rotl(self, n: u32) -> Self {
        let n = n % 8;
        if n == 0 {
            self
        } else {
            self.shl(n).or(self.shr(8 - n))
        }
    }

    /// Rotate every lane right by `n mod 8`.
    #[inline]
    #[must_use]
    fn rotr(self, n: u32) -> Self {
        self.rotl((8 - n % 8) % 8)
    }

    /// `(x + y + 1) / 2` per lane, no overflow (PAVGB, `urhadd`):
    /// Hacker's Delight 2-5, `(x | y) − ((x ^ y) >> 1)`. Of `0x00` and
    /// `0xFF` it makes `0x80`, the sign mask, in registers and without
    /// a load; SSE has no byte shift to make it any other way.
    #[inline]
    #[must_use]
    fn avg_round(self, other: Self) -> Self {
        self.or(other).sub(self.xor(other).shr(1))
    }

    /// `(x + y) / 2` per lane, rounding down (`uhadd`):
    /// `(x & y) + ((x ^ y) >> 1)`.
    #[inline]
    #[must_use]
    fn avg_floor(self, other: Self) -> Self {
        self.and(other).add(self.xor(other).shr(1))
    }

    /// Any Boolean function of three registers, bit by bit, from its
    /// truth table: VPTERNLOG's contract, as [`Bits::ternary`] on words.
    /// The table of a function `f` is `f(0xF0, 0xCC, 0xAA)`
    /// ([`truth_table`](crate::bits::truth_table)).
    #[inline]
    #[must_use]
    fn ternary(self, b: Self, c: Self, table: u8) -> Self {
        let leaf = |t: u8| match t & 3 {
            0 => Self::zero(),
            1 => c.not(),
            2 => c,
            _ => Self::zero().not(),
        };
        let on_b = |t: u8| b.and(leaf(t >> 2)).or(b.not().and(leaf(t)));
        self.and(on_b(table >> 4)).or(self.not().and(on_b(table)))
    }
}

/// Eight lanes in a `u64` by SWAR: portable, and the carrier the
/// exhaustive tests sweep. Lane `i` is bits `8 i .. 8 i + 8`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct U8x8(u64);

const MSBS_STEP_8: u64 = 0x8080_8080_8080_8080;
const LOW7_STEP_8: u64 = !MSBS_STEP_8;
/// Multiplying the lanes' top bits by this gathers them into bits
/// `56..64`, lane `i` at bit `56 + i`; every partial product lands on
/// its own bit, so nothing carries.
const GATHER_MSBS_8: u64 = 0x0002_0408_1020_4081;

impl U8x8 {
    /// Lanes from the word's bytes, lane 0 in the low byte.
    #[inline]
    #[must_use]
    pub const fn new(word: u64) -> Self {
        Self(word)
    }

    /// The word.
    #[inline]
    #[must_use]
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Top bit of every lane as a full mask, `0xFF` or `0x00`.
    #[inline]
    const fn spread_msbs(msbs: u64) -> u64 {
        ((msbs & MSBS_STEP_8) >> 7) * 0xFF
    }
}

impl Lanes for U8x8 {
    const LANES: usize = 8;
    type Bits = u8;

    #[inline]
    fn splat(b: u8) -> Self {
        Self(u64::splat_byte(b))
    }

    #[inline]
    fn load(bytes: &[u8]) -> Self {
        let mut w = [0; 8];
        w.copy_from_slice(bytes);
        Self(u64::from_le_bytes(w))
    }

    #[inline]
    fn lane(self, i: usize) -> u8 {
        self.0.to_le_bytes()[i]
    }

    #[inline]
    fn and(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    #[inline]
    fn or(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    fn xor(self, other: Self) -> Self {
        Self(self.0 ^ other.0)
    }

    #[inline]
    fn not(self) -> Self {
        Self(!self.0)
    }

    /// Hacker's Delight 2-18: add the low seven bits of every lane, put
    /// the top bits back by XOR, so no carry crosses a lane.
    #[inline]
    fn add(self, other: Self) -> Self {
        let low = (self.0 & LOW7_STEP_8) + (other.0 & LOW7_STEP_8);
        Self(low ^ ((self.0 ^ other.0) & MSBS_STEP_8))
    }

    /// Hacker's Delight 2-18, the subtraction: set every top bit so no
    /// borrow crosses a lane, subtract the low seven bits, fix the tops.
    #[inline]
    fn sub(self, other: Self) -> Self {
        let low = (self.0 | MSBS_STEP_8) - (other.0 & LOW7_STEP_8);
        Self(low ^ ((self.0 ^ !other.0) & MSBS_STEP_8))
    }

    #[inline]
    fn shl(self, n: u32) -> Self {
        if n >= 8 {
            return Self::zero();
        }
        Self((self.0 << n) & u64::splat_byte(0xFFu8 << n))
    }

    #[inline]
    fn shr(self, n: u32) -> Self {
        if n >= 8 {
            return Self::zero();
        }
        Self((self.0 >> n) & u64::splat_byte(0xFF >> n))
    }

    #[inline]
    fn cmp_eq(self, other: Self) -> Self {
        Self(Self::spread_msbs((self.0 ^ other.0).zero_bytes()))
    }

    /// The lane compare of Hacker's Delight 6-1 with full 8-bit lanes:
    /// top bit set iff `self <= other`.
    #[inline]
    fn cmp_le(self, other: Self) -> Self {
        let (x, y) = (self.0, other.0);
        let le = (((y | MSBS_STEP_8) - (x & LOW7_STEP_8)) | (x ^ y)) ^ (x & !y);
        Self(Self::spread_msbs(le))
    }

    /// One lane at a time: the portable definition, correct and slow.
    #[inline]
    fn lut16(self, table: [u8; 16]) -> Self {
        let mut out = [0u8; 8];
        for (o, lane) in out.iter_mut().zip(self.0.to_le_bytes()) {
            *o = if lane & 0x80 == 0 {
                table[usize::from(lane & 0x0F)]
            } else {
                0
            };
        }
        Self(u64::from_le_bytes(out))
    }

    #[inline]
    fn to_bits(self) -> u8 {
        // The product's top byte is the gathered bits; the shift keeps it.
        #[allow(clippy::cast_possible_truncation)]
        let gathered = ((self.0 & MSBS_STEP_8).wrapping_mul(GATHER_MSBS_8) >> 56) as u8;
        gathered
    }

    /// Eight parity folds: [`Affine8::apply8`].
    #[inline]
    fn affine(self, map: Affine8) -> Self {
        Self(map.apply8(self.0))
    }

    /// Reverse the word, then put its bytes back in order.
    #[inline]
    fn reverse_bits(self) -> Self {
        Self(self.0.reverse_bits().swap_bytes())
    }
}

// --- U8x16: SSSE3 -------------------------------------------------------

/// The intrinsics are `unsafe` solely because they require the target
/// feature, and `cfg` makes SSSE3 a compile-time fact of this build. The
/// attribute route is closed: `#[target_feature]` cannot go on safe trait
/// methods, and the build configuration does not count for the check.
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "ssse3",
    not(feature = "portable")
))]
#[allow(unsafe_code)]
mod x16 {
    use core::arch::x86_64::{
        __m128i, _mm_add_epi8, _mm_adds_epu8, _mm_alignr_epi8, _mm_and_si128, _mm_avg_epu8,
        _mm_cmpeq_epi8, _mm_cvtsi128_si64, _mm_maddubs_epi16, _mm_min_epu8, _mm_movemask_epi8,
        _mm_or_si128, _mm_sad_epu8, _mm_set_epi64x, _mm_set1_epi8, _mm_shuffle_epi8,
        _mm_srli_si128, _mm_sub_epi8, _mm_subs_epu8, _mm_unpackhi_epi8, _mm_unpacklo_epi8,
        _mm_xor_si128,
    };
    #[cfg(not(target_feature = "gfni"))]
    use core::arch::x86_64::{_mm_cvtsi32_si128, _mm_sll_epi16, _mm_srl_epi16};
    #[cfg(target_feature = "gfni")]
    use core::arch::x86_64::{_mm_gf2p8affine_epi64_epi8, _mm_set1_epi64x};

    #[cfg(target_feature = "gfni")]
    use super::Affine8;
    use super::{Lanes, U8x8};

    /// Sixteen lanes in an XMM register (SSSE3).
    #[derive(Clone, Copy)]
    pub struct U8x16(__m128i);

    impl U8x16 {
        /// Lanes from two words: lanes `0..8` from `lo`, `8..16` from `hi`.
        #[inline]
        #[must_use]
        pub fn from_halves(lo: u64, hi: u64) -> Self {
            // SAFETY: SSE2 is enabled by cfg (SSSE3 implies it).
            Self(unsafe { _mm_set_epi64x(hi.cast_signed(), lo.cast_signed()) })
        }

        /// The two words, lanes `0..8` and `8..16`.
        #[inline]
        #[must_use]
        pub fn halves(self) -> (u64, u64) {
            // SAFETY: SSE2 is enabled by cfg.
            let (lo, hi) = unsafe {
                (
                    _mm_cvtsi128_si64(self.0),
                    _mm_cvtsi128_si64(_mm_srli_si128::<8>(self.0)),
                )
            };
            (lo.cast_unsigned(), hi.cast_unsigned())
        }
    }

    impl Lanes for U8x16 {
        const LANES: usize = 16;
        type Bits = u16;

        #[inline]
        fn splat(b: u8) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_set1_epi8(b.cast_signed()) })
        }

        #[inline]
        fn load(bytes: &[u8]) -> Self {
            assert!(
                bytes.len() == 16,
                "U8x16::load needs 16 bytes, got {}",
                bytes.len()
            );
            let (lo, hi) = (U8x8::load(&bytes[..8]), U8x8::load(&bytes[8..]));
            Self::from_halves(lo.bits(), hi.bits())
        }

        #[inline]
        fn lane(self, i: usize) -> u8 {
            let (lo, hi) = self.halves();
            if i < 8 {
                U8x8::new(lo).lane(i)
            } else {
                U8x8::new(hi).lane(i - 8)
            }
        }

        #[inline]
        fn and(self, other: Self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_and_si128(self.0, other.0) })
        }

        #[inline]
        fn or(self, other: Self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_or_si128(self.0, other.0) })
        }

        #[inline]
        fn xor(self, other: Self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_xor_si128(self.0, other.0) })
        }

        #[inline]
        fn not(self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_xor_si128(self.0, _mm_set1_epi8(-1)) })
        }

        #[inline]
        fn add(self, other: Self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_add_epi8(self.0, other.0) })
        }

        #[inline]
        fn sub(self, other: Self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_sub_epi8(self.0, other.0) })
        }

        /// No 8-bit shift in SSE: shift 16-bit lanes and mask the bits
        /// that crossed a byte. With GFNI the shift is a matrix, below.
        #[cfg(not(target_feature = "gfni"))]
        #[inline]
        fn shl(self, n: u32) -> Self {
            if n >= 8 {
                return Self::zero();
            }
            // SAFETY: SSE2 is enabled by cfg.
            let shifted = unsafe { _mm_sll_epi16(self.0, _mm_cvtsi32_si128(n.cast_signed())) };
            Self(shifted).and(Self::splat(0xFFu8 << n))
        }

        #[cfg(not(target_feature = "gfni"))]
        #[inline]
        fn shr(self, n: u32) -> Self {
            if n >= 8 {
                return Self::zero();
            }
            // SAFETY: SSE2 is enabled by cfg.
            let shifted = unsafe { _mm_srl_epi16(self.0, _mm_cvtsi32_si128(n.cast_signed())) };
            Self(shifted).and(Self::splat(0xFF >> n))
        }

        #[cfg(target_feature = "gfni")]
        #[inline]
        fn shl(self, n: u32) -> Self {
            self.affine(Affine8::shl(n))
        }

        #[cfg(target_feature = "gfni")]
        #[inline]
        fn shr(self, n: u32) -> Self {
            self.affine(Affine8::shr(n))
        }

        #[inline]
        fn cmp_eq(self, other: Self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_cmpeq_epi8(self.0, other.0) })
        }

        /// `a <= b` iff `min(a, b) == a`.
        #[inline]
        fn cmp_le(self, other: Self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_cmpeq_epi8(_mm_min_epu8(self.0, other.0), self.0) })
        }

        #[inline]
        fn lut16(self, table: [u8; 16]) -> Self {
            // SAFETY: SSSE3 is enabled by cfg.
            Self(unsafe { _mm_shuffle_epi8(Self::load(&table).0, self.0) })
        }

        #[inline]
        fn to_bits(self) -> u16 {
            // SAFETY: SSE2 is enabled by cfg. movemask yields 16 bits in an
            // i32; the truncation keeps them all.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let bits = unsafe { _mm_movemask_epi8(self.0) } as u16;
            bits
        }

        #[inline]
        fn shuffle(self, idx: Self) -> Self {
            // SAFETY: SSSE3 is enabled by cfg.
            Self(unsafe { _mm_shuffle_epi8(self.0, idx.0) })
        }

        /// PALIGNR takes an immediate; the match folds to one instruction
        /// wherever `n` is a constant after inlining.
        #[inline]
        fn concat_shift(self, other: Self, n: usize) -> Self {
            macro_rules! alignr {
                ($($k:literal),*) => {
                    match n {
                        // SAFETY: SSSE3 is enabled by cfg.
                        $($k => Self(unsafe { _mm_alignr_epi8::<$k>(other.0, self.0) }),)*
                        17..=31 => other.concat_shift(Self::zero(), n - 16),
                        _ => Self::zero(),
                    }
                };
            }
            alignr!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16)
        }

        #[inline]
        fn add_sat(self, other: Self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_adds_epu8(self.0, other.0) })
        }

        #[inline]
        fn sub_sat(self, other: Self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_subs_epu8(self.0, other.0) })
        }

        #[inline]
        fn unpack_lo(self, other: Self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_unpacklo_epi8(self.0, other.0) })
        }

        #[inline]
        fn unpack_hi(self, other: Self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_unpackhi_epi8(self.0, other.0) })
        }

        /// PSADBW leaves a sum in each 64-bit half.
        #[inline]
        fn sum_abs_diff(self, other: Self) -> u32 {
            // SAFETY: SSE2 is enabled by cfg.
            let (lo, hi) = Self(unsafe { _mm_sad_epu8(self.0, other.0) }).halves();
            // Each half is at most 8 × 255.
            #[allow(clippy::cast_possible_truncation)]
            let sum = (lo + hi) as u32;
            sum
        }

        #[inline]
        fn mul_add_pairs(self, weights: Self) -> Self {
            // SAFETY: SSSE3 is enabled by cfg.
            Self(unsafe { _mm_maddubs_epi16(self.0, weights.0) })
        }

        #[inline]
        fn avg_round(self, other: Self) -> Self {
            // SAFETY: SSE2 is enabled by cfg.
            Self(unsafe { _mm_avg_epu8(self.0, other.0) })
        }

        /// One `gf2p8affineqb`. Its immediate is the constant term, but
        /// an immediate must be a literal and the map is a value, so the
        /// constant goes in as an XOR after the instruction; a linear
        /// map, which every named one is, needs nothing after it.
        #[cfg(target_feature = "gfni")]
        #[inline]
        fn affine(self, map: Affine8) -> Self {
            // SAFETY: GFNI is enabled by cfg, and SSE2 with it.
            let linear = Self(unsafe {
                _mm_gf2p8affine_epi64_epi8::<0>(self.0, _mm_set1_epi64x(map.matrix().cast_signed()))
            });
            if map.is_linear() {
                linear
            } else {
                linear.xor(Self::splat(map.add()))
            }
        }

        #[cfg(target_feature = "gfni")]
        #[inline]
        fn sra(self, n: u32) -> Self {
            self.affine(Affine8::sra(n))
        }

        #[cfg(target_feature = "gfni")]
        #[inline]
        fn rotl(self, n: u32) -> Self {
            self.affine(Affine8::rotl(n))
        }

        #[cfg(target_feature = "gfni")]
        #[inline]
        fn rotr(self, n: u32) -> Self {
            self.affine(Affine8::rotr(n))
        }
    }

    impl PartialEq for U8x16 {
        fn eq(&self, other: &Self) -> bool {
            self.halves() == other.halves()
        }
    }

    impl Eq for U8x16 {}

    impl core::fmt::Debug for U8x16 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let (lo, hi) = self.halves();
            let mut bytes = [0u8; 16];
            bytes[..8].copy_from_slice(&lo.to_le_bytes());
            bytes[8..].copy_from_slice(&hi.to_le_bytes());
            f.debug_tuple("U8x16").field(&bytes).finish()
        }
    }
}

// --- U8x16: NEON --------------------------------------------------------

/// Same boundary as the SSSE3 module: the intrinsics are `unsafe` only for
/// the feature requirement, and `cfg` settles that at compile time.
#[cfg(all(
    target_arch = "aarch64",
    target_feature = "neon",
    not(feature = "portable")
))]
#[allow(unsafe_code)]
mod x16 {
    use core::arch::aarch64::{
        uint8x16_t, uint8x16x2_t, vabdq_u8, vaddlvq_u8, vaddq_u8, vandq_u8, vceqq_u8, vcleq_u8,
        vcltzq_s8, vcombine_s16, vcombine_u8, vcreate_u8, vdupq_n_s8, vdupq_n_u8, veorq_u8,
        vget_high_s8, vget_high_u8, vget_lane_u64, vget_low_s8, vget_low_s16, vget_low_u8,
        vgetq_lane_u64, vhaddq_u8, vmovl_s8, vmovl_u8, vmull_high_s16, vmull_s16, vmvnq_u8,
        vorrq_u8, vpaddq_s32, vqaddq_u8, vqmovn_s32, vqsubq_u8, vqtbl1q_u8, vqtbl2q_u8, vrbitq_u8,
        vreinterpret_u64_u8, vreinterpretq_s8_u8, vreinterpretq_s16_u16, vreinterpretq_u8_s8,
        vreinterpretq_u8_s16, vreinterpretq_u16_u8, vreinterpretq_u64_u8, vrhaddq_u8, vshlq_s8,
        vshlq_u8, vshrn_n_u16, vsubq_u8, vzip1q_u8, vzip2q_u8,
    };

    use super::{Lanes, U8x8};

    /// Sixteen lanes in a NEON register.
    #[derive(Clone, Copy)]
    pub struct U8x16(uint8x16_t);

    impl U8x16 {
        /// Lanes from two words: lanes `0..8` from `lo`, `8..16` from `hi`.
        #[inline]
        #[must_use]
        pub fn from_halves(lo: u64, hi: u64) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vcombine_u8(vcreate_u8(lo), vcreate_u8(hi)) })
        }

        /// The two words, lanes `0..8` and `8..16`.
        #[inline]
        #[must_use]
        pub fn halves(self) -> (u64, u64) {
            // SAFETY: NEON is enabled by cfg.
            unsafe {
                let v = vreinterpretq_u64_u8(self.0);
                (vgetq_lane_u64::<0>(v), vgetq_lane_u64::<1>(v))
            }
        }
    }

    impl Lanes for U8x16 {
        const LANES: usize = 16;
        type Bits = u16;

        #[inline]
        fn splat(b: u8) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vdupq_n_u8(b) })
        }

        #[inline]
        fn load(bytes: &[u8]) -> Self {
            assert!(
                bytes.len() == 16,
                "U8x16::load needs 16 bytes, got {}",
                bytes.len()
            );
            let (lo, hi) = (U8x8::load(&bytes[..8]), U8x8::load(&bytes[8..]));
            Self::from_halves(lo.bits(), hi.bits())
        }

        #[inline]
        fn lane(self, i: usize) -> u8 {
            let (lo, hi) = self.halves();
            if i < 8 {
                U8x8::new(lo).lane(i)
            } else {
                U8x8::new(hi).lane(i - 8)
            }
        }

        #[inline]
        fn and(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vandq_u8(self.0, other.0) })
        }

        #[inline]
        fn or(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vorrq_u8(self.0, other.0) })
        }

        #[inline]
        fn xor(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { veorq_u8(self.0, other.0) })
        }

        #[inline]
        fn not(self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vmvnq_u8(self.0) })
        }

        #[inline]
        fn add(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vaddq_u8(self.0, other.0) })
        }

        #[inline]
        fn sub(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vsubq_u8(self.0, other.0) })
        }

        /// A negative count shifts right; the lane shift needs no mask.
        #[inline]
        fn shl(self, n: u32) -> Self {
            if n >= 8 {
                return Self::zero();
            }
            // `n < 8` fits an i8.
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let count = n as i8;
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vshlq_u8(self.0, vdupq_n_s8(count)) })
        }

        #[inline]
        fn shr(self, n: u32) -> Self {
            if n >= 8 {
                return Self::zero();
            }
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let count = -(n as i8);
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vshlq_u8(self.0, vdupq_n_s8(count)) })
        }

        #[inline]
        fn cmp_eq(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vceqq_u8(self.0, other.0) })
        }

        #[inline]
        fn cmp_le(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vcleq_u8(self.0, other.0) })
        }

        /// `tbl` returns zero for any index `>= 16`; keeping bit 7 of the
        /// lane in the index gives PSHUFB's contract for free.
        #[inline]
        fn lut16(self, table: [u8; 16]) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vqtbl1q_u8(Self::load(&table).0, vandq_u8(self.0, vdupq_n_u8(0x8F))) })
        }

        /// No movemask on NEON: widen the top bits to full lanes, narrow
        /// every 16-bit pair to a nibble per lane (`shrn` by 4), then
        /// gather one bit of each nibble.
        #[inline]
        fn to_bits(self) -> u16 {
            // SAFETY: NEON is enabled by cfg.
            let mut x = unsafe {
                let full = vcltzq_s8(vreinterpretq_s8_u8(self.0));
                let nibbles = vshrn_n_u16::<4>(vreinterpretq_u16_u8(full));
                vget_lane_u64::<0>(vreinterpret_u64_u8(nibbles))
            } & 0x1111_1111_1111_1111;
            x = (x | x >> 3) & 0x3333_3333_3333_3333;
            x = (x | x >> 6) & 0x0F0F_0F0F_0F0F_0F0F;
            x = (x | x >> 12) & 0x00FF_00FF_00FF_00FF;
            x = (x | x >> 24) & 0x0000_FFFF_0000_FFFF;
            x |= x >> 48;
            // Sixteen gathered bits.
            #[allow(clippy::cast_possible_truncation)]
            let bits = x as u16;
            bits
        }

        #[inline]
        fn shuffle(self, idx: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vqtbl1q_u8(self.0, vandq_u8(idx.0, vdupq_n_u8(0x8F))) })
        }

        /// A two-table lookup over `self ++ other`: `tbl` returns zero for
        /// any index past the 32 bytes, which is the contract's zero fill.
        #[inline]
        fn concat_shift(self, other: Self, n: usize) -> Self {
            if n >= 32 {
                return Self::zero();
            }
            // `n < 32` fits a u8.
            #[allow(clippy::cast_possible_truncation)]
            let start = n as u8;
            let iota = Self::load(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]);
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe {
                vqtbl2q_u8(
                    uint8x16x2_t(self.0, other.0),
                    vaddq_u8(iota.0, vdupq_n_u8(start)),
                )
            })
        }

        #[inline]
        fn add_sat(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vqaddq_u8(self.0, other.0) })
        }

        #[inline]
        fn sub_sat(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vqsubq_u8(self.0, other.0) })
        }

        #[inline]
        fn unpack_lo(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vzip1q_u8(self.0, other.0) })
        }

        #[inline]
        fn unpack_hi(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vzip2q_u8(self.0, other.0) })
        }

        #[inline]
        fn sum_abs_diff(self, other: Self) -> u32 {
            // SAFETY: NEON is enabled by cfg.
            u32::from(unsafe { vaddlvq_u8(vabdq_u8(self.0, other.0)) })
        }

        /// No PMADDUBSW on NEON: widen both sides to 16 bits, multiply to
        /// 32, add the pairs, narrow with saturation.
        #[inline]
        fn mul_add_pairs(self, weights: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe {
                let a_lo = vreinterpretq_s16_u16(vmovl_u8(vget_low_u8(self.0)));
                let a_hi = vreinterpretq_s16_u16(vmovl_u8(vget_high_u8(self.0)));
                let w = vreinterpretq_s8_u8(weights.0);
                let w_lo = vmovl_s8(vget_low_s8(w));
                let w_hi = vmovl_s8(vget_high_s8(w));
                let p0 = vmull_s16(vget_low_s16(a_lo), vget_low_s16(w_lo));
                let p1 = vmull_high_s16(a_lo, w_lo);
                let p2 = vmull_s16(vget_low_s16(a_hi), vget_low_s16(w_hi));
                let p3 = vmull_high_s16(a_hi, w_hi);
                let sums = vcombine_s16(
                    vqmovn_s32(vpaddq_s32(p0, p1)),
                    vqmovn_s32(vpaddq_s32(p2, p3)),
                );
                vreinterpretq_u8_s16(sums)
            })
        }

        #[inline]
        fn reverse_bits(self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vrbitq_u8(self.0) })
        }

        /// A negative count on signed lanes is the arithmetic shift;
        /// past the width it leaves the sign.
        #[inline]
        fn sra(self, n: u32) -> Self {
            // `n.min(8)` fits an i8.
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            let count = -(n.min(8) as i8);
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe {
                vreinterpretq_u8_s8(vshlq_s8(vreinterpretq_s8_u8(self.0), vdupq_n_s8(count)))
            })
        }

        #[inline]
        fn avg_round(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vrhaddq_u8(self.0, other.0) })
        }

        #[inline]
        fn avg_floor(self, other: Self) -> Self {
            // SAFETY: NEON is enabled by cfg.
            Self(unsafe { vhaddq_u8(self.0, other.0) })
        }
    }

    impl PartialEq for U8x16 {
        fn eq(&self, other: &Self) -> bool {
            self.halves() == other.halves()
        }
    }

    impl Eq for U8x16 {}

    impl core::fmt::Debug for U8x16 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            let (lo, hi) = self.halves();
            let mut bytes = [0u8; 16];
            bytes[..8].copy_from_slice(&lo.to_le_bytes());
            bytes[8..].copy_from_slice(&hi.to_le_bytes());
            f.debug_tuple("U8x16").field(&bytes).finish()
        }
    }
}

// --- U8x16: portable ----------------------------------------------------

#[cfg(not(any(
    all(
        target_arch = "x86_64",
        target_feature = "ssse3",
        not(feature = "portable")
    ),
    all(
        target_arch = "aarch64",
        target_feature = "neon",
        not(feature = "portable")
    )
)))]
mod x16 {
    use super::{Affine8, Lanes, U8x8};

    /// Sixteen lanes as two [`U8x8`] halves: the portable definition.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct U8x16(U8x8, U8x8);

    impl U8x16 {
        /// Lanes from two words: lanes `0..8` from `lo`, `8..16` from `hi`.
        #[inline]
        #[must_use]
        pub const fn from_halves(lo: u64, hi: u64) -> Self {
            Self(U8x8::new(lo), U8x8::new(hi))
        }

        /// The two words, lanes `0..8` and `8..16`.
        #[inline]
        #[must_use]
        pub const fn halves(self) -> (u64, u64) {
            (self.0.bits(), self.1.bits())
        }

        #[inline]
        fn map(self, other: Self, f: impl Fn(U8x8, U8x8) -> U8x8) -> Self {
            Self(f(self.0, other.0), f(self.1, other.1))
        }
    }

    impl Lanes for U8x16 {
        const LANES: usize = 16;
        type Bits = u16;

        #[inline]
        fn splat(b: u8) -> Self {
            Self(U8x8::splat(b), U8x8::splat(b))
        }

        #[inline]
        fn load(bytes: &[u8]) -> Self {
            assert!(
                bytes.len() == 16,
                "U8x16::load needs 16 bytes, got {}",
                bytes.len()
            );
            Self(U8x8::load(&bytes[..8]), U8x8::load(&bytes[8..]))
        }

        #[inline]
        fn lane(self, i: usize) -> u8 {
            if i < 8 {
                self.0.lane(i)
            } else {
                self.1.lane(i - 8)
            }
        }

        #[inline]
        fn and(self, other: Self) -> Self {
            self.map(other, U8x8::and)
        }

        #[inline]
        fn or(self, other: Self) -> Self {
            self.map(other, U8x8::or)
        }

        #[inline]
        fn xor(self, other: Self) -> Self {
            self.map(other, U8x8::xor)
        }

        #[inline]
        fn not(self) -> Self {
            Self(self.0.not(), self.1.not())
        }

        #[inline]
        fn add(self, other: Self) -> Self {
            self.map(other, U8x8::add)
        }

        #[inline]
        fn sub(self, other: Self) -> Self {
            self.map(other, U8x8::sub)
        }

        #[inline]
        fn shl(self, n: u32) -> Self {
            Self(self.0.shl(n), self.1.shl(n))
        }

        #[inline]
        fn shr(self, n: u32) -> Self {
            Self(self.0.shr(n), self.1.shr(n))
        }

        #[inline]
        fn cmp_eq(self, other: Self) -> Self {
            self.map(other, U8x8::cmp_eq)
        }

        #[inline]
        fn cmp_le(self, other: Self) -> Self {
            self.map(other, U8x8::cmp_le)
        }

        #[inline]
        fn lut16(self, table: [u8; 16]) -> Self {
            Self(self.0.lut16(table), self.1.lut16(table))
        }

        #[inline]
        fn to_bits(self) -> u16 {
            u16::from(self.0.to_bits()) | u16::from(self.1.to_bits()) << 8
        }

        #[inline]
        fn affine(self, map: Affine8) -> Self {
            Self(self.0.affine(map), self.1.affine(map))
        }

        #[inline]
        fn reverse_bits(self) -> Self {
            Self(self.0.reverse_bits(), self.1.reverse_bits())
        }
    }
}

pub use x16::U8x16;

//! The carrier: a fixed-width word viewed as a container of bits.
//!
//! [`Word`] is open. It exposes exactly the primitive circuits every
//! combinator module is built from, so a new carrier (a SIMD lane, a
//! GPU warp mask) is one `impl` block, and every combinator and every
//! law in the crate lights up for it at once.
//!
//! This module is the **only** place hardware selection happens. Each
//! primitive with a fast path ([`pext`](Word::pext), [`pdep`](Word::pdep),
//! [`select_lowest`](Word::select_lowest), [`xor_scan`](Word::xor_scan),
//! [`xor_scan_down`](Word::xor_scan_down))
//! picks the instruction when the matching `target_feature` is enabled
//! at compile time and the `portable` cargo feature is off; otherwise a
//! broadword definition with the same contract. The laws in
//! [`crate::laws`] are what make the two interchangeable.

/// A fixed-width word of `BITS` bits, bit 0 least significant: the
/// carrier every combinator and every law is written against.
///
/// Open to implementors. A carrier supplies the constants and the
/// required primitives; the provided ones have portable definitions
/// and are overridden only for speed (`u64` routes `pext` to BMI2 and
/// `xor_scan` to PCLMULQDQ). What an implementation must uphold is
/// what the laws in [`crate::laws`] check: run them over the new
/// carrier as its acceptance test, the way `tests/laws.rs` does for
/// [`Wide<N>`](crate::Wide).
///
/// Shift amounts passed to [`shl`](Word::shl) / [`shr`](Word::shr)
/// must be `< BITS`; combinators in this crate uphold that by
/// construction and debug-assert it, and a carrier may assume it.
///
/// `BITS` is a multiple of 8 and at most `2^16`: the byte combinators
/// read bytes, and [`expand_broadword`] keeps one mask per halving
/// round. A wider carrier is a compile error, not a surprise.
///
/// A word is a plain value: ordered as the unsigned integer it spells,
/// printable in binary and hex (`{:b}` is how bits want to be read),
/// hashable, shareable, `'static`. Asked for now because asking later
/// would break every carrier written meanwhile.
pub trait Word:
    Copy
    + Ord
    + core::hash::Hash
    + core::fmt::Debug
    + core::fmt::Binary
    + core::fmt::LowerHex
    + core::fmt::UpperHex
    + Send
    + Sync
    + 'static
{
    /// Width in bits.
    const BITS: u32;
    /// All bits clear.
    const ZERO: Self;
    /// Only bit 0 set.
    const ONE: Self;
    /// All bits set.
    const ONES: Self;

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
    /// Logical shift left by `n < BITS`.
    #[must_use]
    fn shl(self, n: u32) -> Self;
    /// Logical shift right by `n < BITS`.
    #[must_use]
    fn shr(self, n: u32) -> Self;

    /// The carry chain: wrapping addition.
    #[must_use]
    fn wrapping_add(self, other: Self) -> Self;
    /// The borrow chain: wrapping subtraction.
    #[must_use]
    fn wrapping_sub(self, other: Self) -> Self;
    /// Wrapping multiplication: the carry chain applied to every
    /// shifted copy at once; the SWAR way to sum or broadcast lanes.
    #[must_use]
    fn wrapping_mul(self, other: Self) -> Self;
    /// The byte `b` repeated in every 8-bit lane.
    #[must_use]
    fn splat_byte(b: u8) -> Self;
    /// The lowest 8 bits.
    #[must_use]
    fn low_byte(self) -> u8;

    /// Number of set bits (POPCNT).
    #[must_use]
    fn count_ones(self) -> u32;
    /// Index of the lowest set bit; `BITS` when zero (TZCNT).
    #[must_use]
    fn trailing_zeros(self) -> u32;
    /// Number of leading zero bits; `BITS` when zero (LZCNT).
    #[must_use]
    fn leading_zeros(self) -> u32;
    /// Clears the lowest set bit (BLSR); identity on zero.
    #[must_use]
    fn clear_lowest_set(self) -> Self {
        self.and(self.wrapping_sub(Self::ONE))
    }

    /// Parallel bit extract (PEXT): gathers the bits of `self` at the
    /// set positions of `mask` into the low `count_ones(mask)` bits,
    /// preserving order. BMI2: one instruction; portable:
    /// [`compress_broadword`].
    #[must_use]
    fn pext(self, mask: Self) -> Self {
        compress_broadword(self, mask)
    }
    /// Parallel bit deposit (PDEP): scatters the low `count_ones(mask)`
    /// bits of `self` to the set positions of `mask`, preserving order.
    /// BMI2: one instruction; portable: [`expand_broadword`].
    #[must_use]
    fn pdep(self, mask: Self) -> Self {
        expand_broadword(self, mask)
    }

    /// Position of the `k`-th set bit (from 0), or `BITS` when there is
    /// none: the answer `trailing_zeros` gives for zero, which is where
    /// clearing `k` lowest set bits leaves you. The provided loop takes `k`
    /// steps, so a carrier overrides it. BMI2: `trailing_zeros(pdep(1 << k,
    /// self))`; portable: Vigna's broadword select.
    #[must_use]
    fn select_lowest(self, k: u32) -> u32 {
        let mut x = self;
        for _ in 0..k {
            x = x.clear_lowest_set();
        }
        x.trailing_zeros()
    }

    /// Prefix XOR: bit `i` of the result is the parity of bits `0..=i`.
    /// PCLMULQDQ: carry-less multiply by all-ones; portable: log-depth
    /// smear.
    #[must_use]
    fn xor_scan(self) -> Self {
        xor_smear(self)
    }

    /// Suffix XOR: bit `i` of the result is the parity of bits
    /// `i..BITS`. The Gray decode. PCLMULQDQ: the high half of the
    /// carry-less multiply by all-ones is the exclusive suffix parity,
    /// one XOR from the inclusive; portable: log-depth smear downward.
    #[must_use]
    fn xor_scan_down(self) -> Self {
        xor_smear_down(self)
    }

    /// Mask with the `n` lowest bits set, `n <= BITS`.
    #[must_use]
    fn low_ones(n: u32) -> Self {
        if n >= Self::BITS {
            Self::ONES
        } else {
            Self::ONE.shl(n).wrapping_sub(Self::ONE)
        }
    }

    /// `true` when no bit is set.
    #[inline]
    #[must_use]
    fn is_zero(self) -> bool {
        self == Self::ZERO
    }

    /// `true` when bit `i < BITS` is set.
    #[inline]
    #[must_use]
    fn bit(self, i: u32) -> bool {
        !self.shr(i).and(Self::ONE).is_zero()
    }

    /// `true` when an odd number of bits is set.
    #[inline]
    #[must_use]
    fn parity(self) -> bool {
        self.count_ones() & 1 == 1
    }

    /// Number of consecutive set bits from bit 0 upward.
    #[inline]
    #[must_use]
    fn trailing_ones(self) -> u32 {
        self.not().trailing_zeros()
    }

    /// Number of consecutive set bits from the top downward.
    #[inline]
    #[must_use]
    fn leading_ones(self) -> u32 {
        self.not().leading_zeros()
    }
}

// --- portable definitions ----------------------------------------------

/// Portable PEXT: compress by parallel suffix (Hacker's Delight 7-4).
///
/// `log₂ BITS` rounds. Round `i` moves every selected bit right by
/// `2^i` exactly when the number of unselected bits below it has bit
/// `i` set; that count's parity, for every bit at once, is one
/// prefix-XOR scan ([`Word::xor_scan`]) of the "zeros to the right"
/// mask. Constant time, no table, no data-dependent branch: about
/// `log₂ BITS × (9 + cost of a scan)` operations, where a scan is one
/// PCLMULQDQ on targets that have it and a log-depth smear otherwise.
/// `benches/compact.rs` compares it with PEXT and with a loop over the
/// mask's set bits.
///
/// ```
/// use hakmem::word::compress_broadword;
/// assert_eq!(compress_broadword(0b1001u32, 0b1010), 0b10);
/// ```
#[inline]
#[must_use]
pub fn compress_broadword<W: Word>(x: W, mut mask: W) -> W {
    let mut x = x.and(mask);
    let mut mk = mask.not().shl(1); // zeros to the right of each bit
    let mut i = 0;
    while (1u32 << i) < W::BITS {
        let mp = mk.xor_scan(); // parity of those zeros, per bit
        let mv = mp.and(mask); // bits that move this round
        mask = mask.xor(mv).or(mv.shr(1u32 << i));
        let t = x.and(mv);
        x = x.xor(t).or(t.shr(1u32 << i));
        mk = mk.and(mp.not());
        i += 1;
    }
    x
}

/// Portable PDEP: expand by parallel suffix (Hacker's Delight 7-5).
///
/// The move masks of [`compress_broadword`], computed forward and
/// applied in reverse with left shifts. Constant time.
///
/// ```
/// use hakmem::word::expand_broadword;
/// assert_eq!(expand_broadword(0b10u32, 0b1010), 0b1000);
/// ```
#[inline]
#[must_use]
pub fn expand_broadword<W: Word>(x: W, mask: W) -> W {
    // One mask a halving round, 16 rounds for 2^16 bits. Past that the
    // array ran out at run time; now the build does.
    const { assert!(W::BITS <= 1 << 16, "expand_broadword: BITS above 2^16") };
    let mut moves = [W::ZERO; 16];
    let mut m = mask;
    let mut mk = mask.not().shl(1);
    let mut rounds = 0;
    while (1u32 << rounds) < W::BITS {
        let mp = mk.xor_scan();
        let mv = mp.and(m);
        moves[rounds] = mv;
        m = m.xor(mv).or(mv.shr(1u32 << rounds));
        mk = mk.and(mp.not());
        rounds += 1;
    }
    let mut x = x;
    for i in (0..rounds).rev() {
        let mv = moves[i];
        x = x.and(mv.not()).or(x.shl(1u32 << i).and(mv));
    }
    x.and(mask)
}

/// Log-depth XOR smear: `x ^= x << 1; x ^= x << 2; …`.
#[inline]
fn xor_smear<W: Word>(mut x: W) -> W {
    let mut s = 1;
    while s < W::BITS {
        x = x.xor(x.shl(s));
        s <<= 1;
    }
    x
}

/// Log-depth XOR smear downward: `x ^= x >> 1; x ^= x >> 2; …`.
#[inline]
fn xor_smear_down<W: Word>(mut x: W) -> W {
    let mut s = 1;
    while s < W::BITS {
        x = x.xor(x.shr(s));
        s <<= 1;
    }
    x
}

const ONES_STEP_4: u64 = 0x1111_1111_1111_1111;
const ONES_STEP_8: u64 = 0x0101_0101_0101_0101;
const MSBS_STEP_8: u64 = 0x8080_8080_8080_8080;
/// Byte `i` holds `1 << i`.
const INCR_STEP_8: u64 = 0x8040_2010_0804_0201;

/// Position of the `k`-th set bit of a 64-bit word without PDEP.
///
/// Vigna, *Broadword Implementation of Rank/Select Queries* (WEA 2008),
/// with the final in-byte step done by the same compare-and-count
/// trick instead of a table. Constant time, ~30 ALU ops, no memory;
/// see `benches/select.rs` for how it compares with PDEP and with a
/// clear-lowest-bit loop on a given microarchitecture.
///
/// 64 when `k >= count_ones(x)`.
///
/// ```
/// assert_eq!(hakmem::word::select_broadword64(0b1011_0000, 2), 7);
/// assert_eq!(hakmem::word::select_broadword64(0b1011_0000, 3), 64);
/// ```
#[must_use]
pub fn select_broadword64(x: u64, k: u32) -> u32 {
    // Phase 1: per-byte popcounts, then per-byte prefix sums by
    // multiplying with 0x0101…: byte j = popcount of bytes 0..=j.
    let mut s = x - ((x >> 1) & (0x5 * ONES_STEP_4));
    s = (s & (0x3 * ONES_STEP_4)) + ((s >> 2) & (0x3 * ONES_STEP_4));
    s = (s + (s >> 4)) & (0x0F * ONES_STEP_8);
    let byte_sums = s.wrapping_mul(ONES_STEP_8);
    // The top byte is the whole count, so asking past it costs a compare
    // the algorithm had already paid for. Without it the answer is a
    // shift by 64, which Rust calls a panic and x86 calls a shift by 0.
    if u64::from(k) >= byte_sums >> 56 {
        return 64;
    }

    // Phase 2: the byte holding the answer is the number of bytes whose
    // prefix sum is <= k. Compare all eight at once: with the MSB
    // pre-set, `(k | 0x80) - sum` keeps its MSB iff sum <= k.
    let k_step_8 = u64::from(k) * ONES_STEP_8;
    let geq_k = ((k_step_8 | MSBS_STEP_8) - byte_sums) & MSBS_STEP_8;
    let place = geq_k.count_ones() * 8;
    let byte_rank = k - (((byte_sums << 8) >> place) & 0xFF) as u32;

    // Phase 3: the same trick inside the byte. Spread the byte into all
    // eight lanes keeping bit i in lane i, normalise to 0/1, prefix-sum
    // by multiplication, and count lanes whose sum is <= byte_rank.
    let byte = (x >> place) & 0xFF;
    let mut spread = byte.wrapping_mul(ONES_STEP_8) & INCR_STEP_8;
    spread |= spread >> 4;
    spread |= spread >> 2;
    spread |= spread >> 1;
    let bit_sums = (spread & ONES_STEP_8).wrapping_mul(ONES_STEP_8);
    let byte_rank_step_8 = u64::from(byte_rank) * ONES_STEP_8;
    let geq_r = ((byte_rank_step_8 | MSBS_STEP_8) - bit_sums) & MSBS_STEP_8;
    place + geq_r.count_ones()
}

// --- hardware paths -------------------------------------------------------

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "bmi2",
    not(feature = "portable")
))]
#[allow(unsafe_code)]
mod bmi2 {
    //! BMI2 fast paths.
    //!
    //! The intrinsics are `unsafe` solely because they require the
    //! `bmi2` target feature. The `cfg` on this module proves the
    //! feature is enabled for the whole compilation, so the calls are
    //! sound; there is no other precondition.
    use core::arch::x86_64::{_pdep_u32, _pdep_u64, _pext_u32, _pext_u64};

    #[inline]
    pub(super) fn pext32(x: u32, m: u32) -> u32 {
        // SAFETY: bmi2 is statically enabled (module cfg).
        unsafe { _pext_u32(x, m) }
    }
    #[inline]
    pub(super) fn pdep32(x: u32, m: u32) -> u32 {
        // SAFETY: bmi2 is statically enabled (module cfg).
        unsafe { _pdep_u32(x, m) }
    }
    #[inline]
    pub(super) fn pext64(x: u64, m: u64) -> u64 {
        // SAFETY: bmi2 is statically enabled (module cfg).
        unsafe { _pext_u64(x, m) }
    }
    #[inline]
    pub(super) fn pdep64(x: u64, m: u64) -> u64 {
        // SAFETY: bmi2 is statically enabled (module cfg).
        unsafe { _pdep_u64(x, m) }
    }
}

#[cfg(all(
    target_arch = "x86_64",
    target_feature = "pclmulqdq",
    not(feature = "portable")
))]
#[allow(unsafe_code)]
mod clmul {
    //! PCLMULQDQ fast path for prefix XOR: a carry-less multiply by
    //! all-ones XORs every left shift of the operand together, which is
    //! exactly the prefix parity. The high half of the same 128-bit
    //! product holds every right shift combined by XOR, bit `i` the
    //! parity of bits `i + 1..64`: the exclusive suffix parity, one
    //! XOR from the suffix scan. Same soundness argument as `bmi2`.
    use core::arch::x86_64::{
        __m128i, _mm_clmulepi64_si128, _mm_cvtsi128_si64, _mm_set_epi64x, _mm_unpackhi_epi64,
    };

    #[inline]
    pub(super) fn prefix_xor64(x: u64) -> u64 {
        // SAFETY: pclmulqdq (and sse2, implied on x86_64) are
        // statically enabled (module cfg). Reinterpreting the u64 as
        // an i64 lane is a bit copy.
        unsafe {
            let a: __m128i = _mm_set_epi64x(0, x.cast_signed());
            let ones: __m128i = _mm_set_epi64x(0, -1);
            _mm_cvtsi128_si64(_mm_clmulepi64_si128(a, ones, 0)).cast_unsigned()
        }
    }

    #[inline]
    pub(super) fn suffix_xor64(x: u64) -> u64 {
        // SAFETY: as above; `unpackhi` moves the high lane down, a
        // register shuffle.
        unsafe {
            let a: __m128i = _mm_set_epi64x(0, x.cast_signed());
            let ones: __m128i = _mm_set_epi64x(0, -1);
            let p = _mm_clmulepi64_si128(a, ones, 0);
            let exclusive = _mm_cvtsi128_si64(_mm_unpackhi_epi64(p, p)).cast_unsigned();
            exclusive ^ x
        }
    }
}

// --- carrier impls --------------------------------------------------------

macro_rules! impl_word_core {
    ($t:ty) => {
        const BITS: u32 = <$t>::BITS;
        const ZERO: Self = 0;
        const ONE: Self = 1;
        const ONES: Self = <$t>::MAX;

        #[inline]
        fn and(self, other: Self) -> Self {
            self & other
        }
        #[inline]
        fn or(self, other: Self) -> Self {
            self | other
        }
        #[inline]
        fn xor(self, other: Self) -> Self {
            self ^ other
        }
        #[inline]
        fn not(self) -> Self {
            !self
        }

        #[inline]
        fn shl(self, n: u32) -> Self {
            debug_assert!(n < Self::BITS, "shift {n} >= width {}", Self::BITS);
            self << n
        }
        #[inline]
        fn shr(self, n: u32) -> Self {
            debug_assert!(n < Self::BITS, "shift {n} >= width {}", Self::BITS);
            self >> n
        }

        #[inline]
        fn wrapping_add(self, other: Self) -> Self {
            <$t>::wrapping_add(self, other)
        }
        #[inline]
        fn wrapping_sub(self, other: Self) -> Self {
            <$t>::wrapping_sub(self, other)
        }
        #[inline]
        fn wrapping_mul(self, other: Self) -> Self {
            <$t>::wrapping_mul(self, other)
        }
        // Truncation is the point.
        #[allow(clippy::cast_possible_truncation)]
        #[inline]
        fn low_byte(self) -> u8 {
            self as u8
        }
        #[inline]
        fn splat_byte(b: u8) -> Self {
            // 0x0101…01 has one bit per lane; multiplying copies `b` into each.
            Self::from(b).wrapping_mul(<$t>::MAX / 0xFF)
        }

        #[inline]
        fn count_ones(self) -> u32 {
            <$t>::count_ones(self)
        }
        #[inline]
        fn trailing_zeros(self) -> u32 {
            <$t>::trailing_zeros(self)
        }
        #[inline]
        fn leading_zeros(self) -> u32 {
            <$t>::leading_zeros(self)
        }
        #[inline]
        fn clear_lowest_set(self) -> Self {
            self & self.wrapping_sub(1)
        }

        #[inline]
        fn low_ones(n: u32) -> Self {
            debug_assert!(n <= Self::BITS, "mask width {n} > {}", Self::BITS);
            if n >= Self::BITS {
                Self::ONES
            } else {
                (1 << n) - 1
            }
        }
    };
}

impl Word for u64 {
    impl_word_core!(u64);

    #[inline]
    fn pext(self, mask: Self) -> Self {
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "bmi2",
            not(feature = "portable")
        ))]
        {
            bmi2::pext64(self, mask)
        }
        #[cfg(not(all(
            target_arch = "x86_64",
            target_feature = "bmi2",
            not(feature = "portable")
        )))]
        {
            compress_broadword(self, mask)
        }
    }
    #[inline]
    fn pdep(self, mask: Self) -> Self {
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "bmi2",
            not(feature = "portable")
        ))]
        {
            bmi2::pdep64(self, mask)
        }
        #[cfg(not(all(
            target_arch = "x86_64",
            target_feature = "bmi2",
            not(feature = "portable")
        )))]
        {
            expand_broadword(self, mask)
        }
    }
    #[inline]
    fn select_lowest(self, k: u32) -> u32 {
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "bmi2",
            not(feature = "portable")
        ))]
        {
            // Past the last set bit PDEP deposits nothing and TZCNT says 64,
            // as long as `1 << k` is not asked to exist first.
            bmi2::pdep64(1u64.checked_shl(k).unwrap_or(0), self).trailing_zeros()
        }
        #[cfg(not(all(
            target_arch = "x86_64",
            target_feature = "bmi2",
            not(feature = "portable")
        )))]
        {
            select_broadword64(self, k)
        }
    }
    #[inline]
    fn xor_scan(self) -> Self {
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "pclmulqdq",
            not(feature = "portable")
        ))]
        {
            clmul::prefix_xor64(self)
        }
        #[cfg(not(all(
            target_arch = "x86_64",
            target_feature = "pclmulqdq",
            not(feature = "portable")
        )))]
        {
            xor_smear(self)
        }
    }
    #[inline]
    fn xor_scan_down(self) -> Self {
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "pclmulqdq",
            not(feature = "portable")
        ))]
        {
            clmul::suffix_xor64(self)
        }
        #[cfg(not(all(
            target_arch = "x86_64",
            target_feature = "pclmulqdq",
            not(feature = "portable")
        )))]
        {
            xor_smear_down(self)
        }
    }
}

impl Word for u32 {
    impl_word_core!(u32);

    #[inline]
    fn pext(self, mask: Self) -> Self {
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "bmi2",
            not(feature = "portable")
        ))]
        {
            bmi2::pext32(self, mask)
        }
        #[cfg(not(all(
            target_arch = "x86_64",
            target_feature = "bmi2",
            not(feature = "portable")
        )))]
        {
            compress_broadword(self, mask)
        }
    }
    #[inline]
    fn pdep(self, mask: Self) -> Self {
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "bmi2",
            not(feature = "portable")
        ))]
        {
            bmi2::pdep32(self, mask)
        }
        #[cfg(not(all(
            target_arch = "x86_64",
            target_feature = "bmi2",
            not(feature = "portable")
        )))]
        {
            expand_broadword(self, mask)
        }
    }
    #[inline]
    fn select_lowest(self, k: u32) -> u32 {
        // Zero-extended, "none" comes back as 64.
        u64::from(self).select_lowest(k).min(Self::BITS)
    }
    // The prefix of a zero-extended word is the prefix of the word.
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn xor_scan(self) -> Self {
        u64::from(self).xor_scan() as Self
    }
    // The zero extension contributes no parity: the suffix of the
    // extended word, truncated, is the suffix of the word.
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn xor_scan_down(self) -> Self {
        u64::from(self).xor_scan_down() as Self
    }
}

impl Word for u128 {
    impl_word_core!(u128);

    /// Two 64-bit halves: the high half's extract lands above the low
    /// half's `popcount` bits.
    // Truncating casts split the halves on purpose.
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn pext(self, mask: Self) -> Self {
        let (lo, hi) = (self as u64, (self >> 64) as u64);
        let (mlo, mhi) = (mask as u64, (mask >> 64) as u64);
        Self::from(lo.pext(mlo)) | (Self::from(hi.pext(mhi)) << mlo.count_ones())
    }
    /// Two 64-bit halves: the high half consumes source bits after the
    /// low mask's `popcount`.
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn pdep(self, mask: Self) -> Self {
        let (mlo, mhi) = (mask as u64, (mask >> 64) as u64);
        let lo = (self as u64).pdep(mlo);
        let hi = ((self >> mlo.count_ones()) as u64).pdep(mhi);
        Self::from(lo) | (Self::from(hi) << 64)
    }
    /// Pick the half by comparing `k` with the low half's popcount.
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn select_lowest(self, k: u32) -> u32 {
        let (lo, hi) = (self as u64, (self >> 64) as u64);
        let n = lo.count_ones();
        if k < n {
            lo.select_lowest(k)
        } else {
            64 + hi.select_lowest(k - n)
        }
    }
    /// Prefix of each half, with the low half's parity carried into
    /// every bit of the high half.
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn xor_scan(self) -> Self {
        let (lo, hi) = (self as u64, (self >> 64) as u64);
        let carry = 0u64.wrapping_sub(u64::from(lo.count_ones() & 1));
        Self::from(lo.xor_scan()) | (Self::from(hi.xor_scan() ^ carry) << 64)
    }
    /// Suffix of each half, with the high half's parity carried into
    /// every bit of the low half.
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn xor_scan_down(self) -> Self {
        let (lo, hi) = (self as u64, (self >> 64) as u64);
        let carry = 0u64.wrapping_sub(u64::from(hi.count_ones() & 1));
        Self::from(lo.xor_scan_down() ^ carry) | (Self::from(hi.xor_scan_down()) << 64)
    }
}

/// Narrow carriers delegate the hardware-backed primitives to `u32` /
/// `u64`: zero-extending both operands leaves the result inside the
/// low bits, so the fast path is shared. They exist mainly so laws
/// can be checked exhaustively (`tests/exhaustive.rs`).
macro_rules! impl_word_narrow {
    ($($t:ty),*) => {$(
        impl Word for $t {
            impl_word_core!($t);

            // The result fits: it has at most `count_ones(mask)` bits.
            #[allow(clippy::cast_possible_truncation)]
            #[inline]
            fn pext(self, mask: Self) -> Self {
                u32::from(self).pext(u32::from(mask)) as $t
            }
            // Deposited bits land only at set positions of `mask`.
            #[allow(clippy::cast_possible_truncation)]
            #[inline]
            fn pdep(self, mask: Self) -> Self {
                u32::from(self).pdep(u32::from(mask)) as $t
            }
            #[inline]
            fn select_lowest(self, k: u32) -> u32 {
                u64::from(self).select_lowest(k).min(Self::BITS)
            }
            #[allow(clippy::cast_possible_truncation)]
            #[inline]
            fn xor_scan(self) -> Self {
                u64::from(self).xor_scan() as $t
            }
            #[allow(clippy::cast_possible_truncation)]
            #[inline]
            fn xor_scan_down(self) -> Self {
                u64::from(self).xor_scan_down() as $t
            }
        }
    )*};
}

impl_word_narrow!(u8, u16);

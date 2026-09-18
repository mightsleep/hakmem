//! The one trait to import: every combinator, on every carrier.
//!
//! `use hakmem::Bits;` and the whole algebra is available on `u8`,
//! `u16`, `u32`, `u64` and `u128` (the Itertools shape: one trait,
//! many methods, organised below by the circuit each method is).
//! Primitives (the sealed [`Word`]) keep `std` names, `count_ones`,
//! `trailing_zeros`, `leading_zeros`, plus `pext` / `pdep` /
//! `select_lowest` / `xor_scan` for the hardware-backed ones.

use crate::set::Positions;
use crate::word::Word;

/// Every combinator of the algebra as a method. Blanket-implemented
/// for every [`Word`]; never implement it yourself.
///
/// Sections, in source order:
///
/// - **Runs** (Hacker's Delight 6-2 / 6-3): the halving chain `x &= x >> s` turns "is there a run
///   of `k` ones here" into `⌈log₂ k⌉` operations.
/// - **Set view**: rank (POPCNT under a mask), select (`k`-th set bit, PDEP + TZCNT or Vigna's
///   broadword select), first / last, positions.
/// - **Scans**: prefix XOR (PCLMULQDQ or smear; simdjson's inside-string mask), prefix OR (`x |
///   -x`), Gray code, `find_escaped`, Kogge–Stone fills along a stride through a propagation mask.
/// - **Compact / expand** (PEXT / PDEP): the bijection between "bits at the positions of `mask`"
///   and "the low `count_ones(mask)` bits".
/// - **Basics** (HD ch. 2, HAKMEM 175): lowest-set-bit family, blend, powers of two, Gosper's hack.
/// - **SWAR byte lanes** (HD 6-1): exact zero / equal / less-than tests on every byte at once; the
///   `strlen` / `memchr` sentences.
/// - **Permutations**: delta swap, the primitive of every Beneš network (8×8 board permutations
///   live in [`crate::permute::board8`]).
pub trait Bits: Word {
    // ===================================================================
    // Runs: Hacker's Delight 6-2 / 6-3
    // ===================================================================
    /// Bit `p` of the result is set iff bits `p..p + k` of `self` are
    /// all set. `k` must be in `1..=BITS`.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// let x: u64 = 0b0111_0110;
    /// assert_eq!(x.run_starts(1), x); // every set bit
    /// assert_eq!(x.run_starts(2), 0b0011_0010); // starts of 2-runs
    /// assert_eq!(x.run_starts(3), 0b0001_0000);
    /// assert_eq!(x.run_starts(4), 0);
    /// ```
    #[must_use]
    fn run_starts(self, k: u32) -> Self {
        debug_assert!(
            (1..=Self::BITS).contains(&k),
            "run length {k} outside 1..={}",
            Self::BITS
        );
        let mut x = self;
        let mut n = k;
        while n > 1 {
            let s = n >> 1;
            x = x.and(x.shr(s));
            n -= s;
        }
        x
    }

    /// `true` iff `self` contains at least one run of `k` set bits.
    #[inline]
    #[must_use]
    fn has_run(self, k: u32) -> bool {
        !self.run_starts(k).is_zero()
    }

    /// Length of the longest run of set bits (Hacker's Delight 6-3),
    /// by binary search over [`has_run`](Bits::has_run): `log₂ BITS`
    /// probes, no data-dependent loop.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// assert_eq!(0b0111_0110u32.longest_run(), 3);
    /// assert_eq!(0u32.longest_run(), 0);
    /// assert_eq!(u64::MAX.longest_run(), 64);
    /// ```
    #[must_use]
    fn longest_run(self) -> u32 {
        // Invariant: has_run(lo) holds (lo = 0 trivially), has_run(hi + 1) fails.
        let (mut lo, mut hi) = (0, Self::BITS);
        while lo < hi {
            let mid = lo + (hi - lo).div_ceil(2);
            if self.has_run(mid) {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        lo
    }

    // ===================================================================
    // Set view: rank, select, positions
    // ===================================================================
    /// Number of set bits at positions `< i`; `i` must be `<= BITS`.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// let x: u64 = 0b1011;
    /// assert_eq!(x.rank_below(0), 0);
    /// assert_eq!(x.rank_below(2), 2);
    /// assert_eq!(x.rank_below(64), 3);
    /// ```
    #[inline]
    #[must_use]
    fn rank_below(self, i: u32) -> u32 {
        self.and(Self::low_ones(i)).count_ones()
    }

    /// Position of the `k`-th set bit (0-indexed), if `k < popcount`.
    ///
    /// BMI2: `trailing_zeros(pdep(1 << k, self))`. Deposit a lone bit onto the
    /// `k`-th set position, then find it. Portable: Vigna's broadword
    /// select ([`select_broadword64`](crate::word::select_broadword64)),
    /// still O(1). See [`Word::select_lowest`].
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// let x: u64 = 0b1011_0000;
    /// assert_eq!(x.select(0), Some(4));
    /// assert_eq!(x.select(2), Some(7));
    /// assert_eq!(x.select(3), None);
    /// ```
    #[inline]
    #[must_use]
    fn select(self, k: u32) -> Option<u32> {
        if k >= self.count_ones() {
            None
        } else {
            Some(self.select_lowest(k))
        }
    }

    /// Position of the lowest set bit, if any.
    #[inline]
    #[must_use]
    fn first_set(self) -> Option<u32> {
        if self.is_zero() {
            None
        } else {
            Some(self.trailing_zeros())
        }
    }

    /// Position of the highest set bit, if any.
    #[inline]
    #[must_use]
    fn last_set(self) -> Option<u32> {
        if self.is_zero() {
            None
        } else {
            Some(Self::BITS - 1 - self.leading_zeros())
        }
    }
    /// Positions of the set bits, ascending. See [`Positions`].
    #[inline]
    fn positions(self) -> Positions<Self> {
        Positions(self)
    }

    // ===================================================================
    // Scans: prefix XOR / OR, Gray code, escapes, fills
    // ===================================================================
    /// Bit `i` of the result is the OR of bits `0..=i` of `self`:
    /// everything at and above the lowest set bit becomes set.
    #[inline]
    #[must_use]
    fn prefix_or(self) -> Self {
        self.or(Self::ZERO.wrapping_sub(self))
    }

    /// Bit `i` of the result is the XOR (parity) of bits `0..=i`.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// // Toggle at bits 2 and 6 → set on [2, 6).
    /// let toggles: u64 = 0b0100_0100;
    /// assert_eq!(toggles.prefix_xor(), 0b0011_1100);
    /// ```
    #[inline]
    #[must_use]
    fn prefix_xor(self) -> Self {
        self.xor_scan()
    }

    /// Inverse of [`prefix_xor`](Bits::prefix_xor): `x ^ (x << 1)`,
    /// the positions where the prefix parity changes.
    #[inline]
    #[must_use]
    fn delta_xor(self) -> Self {
        self.xor(self.shl(1))
    }
    /// Reflected binary Gray code: `x ^ (x >> 1)`. Consecutive integers
    /// map to words differing in exactly one bit.
    #[inline]
    #[must_use]
    fn gray_encode(self) -> Self {
        self.xor(self.shr(1))
    }

    /// Inverse of [`gray_encode`](Bits::gray_encode): the prefix XOR
    /// from the top, i.e. [`prefix_xor`](Bits::prefix_xor) of the
    /// bit-reversed word, reversed back; computed directly as a
    /// downward smear.
    #[inline]
    #[must_use]
    fn gray_decode(self) -> Self {
        let mut x = self;
        let mut s = 1;
        while s < Self::BITS {
            x = x.xor(x.shr(s));
            s <<= 1;
        }
        x
    }

    /// Positions immediately following an odd-length run of backslashes
    /// (simdjson's `find_escaped`), with the run parity carried across
    /// words. Backslashes inside a run are not reported (they pair up);
    /// only the character an odd run escapes.
    ///
    /// `self` marks backslash positions. `prev_ends_odd` says whether
    /// the previous word ended in an odd-length run, i.e. whether bit 0
    /// of this word is escaped. Returns the escaped positions and the
    /// carry for the next word. Pure carry-chain arithmetic: runs
    /// starting on even and on odd positions are added separately so
    /// each run's end parity falls out of the alternating-bit masks.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// // text:      a \ " \ \ " \ \ \ "
    /// // index:     0 1 2 3 4 5 6 7 8 9
    /// let backslashes: u16 = 0b01_1101_1010;
    /// let (escaped, carry) = backslashes.find_escaped(false);
    /// // The quote at 2 is escaped (run of 1), the quote at 5 is not
    /// // (run of 2), the quote at 9 is escaped (run of 3).
    /// assert_eq!(escaped, 0b10_0000_0100);
    /// assert!(!carry);
    /// ```
    #[must_use]
    fn find_escaped(self, prev_ends_odd: bool) -> (Self, bool) {
        let even = Self::splat_byte(0x55);
        let odd = even.not();
        // Only bit 0 can continue a run from the previous word.
        let flip = if prev_ends_odd { Self::ONE } else { Self::ZERO };

        let start_edges = self.and(self.shl(1).not());
        let even_start_mask = even.xor(flip);
        let even_starts = start_edges.and(even_start_mask);
        let odd_starts = start_edges.and(even_start_mask.not());

        let even_carries = self.wrapping_add(even_starts);
        // A run continuing from the previous word with odd parity ends
        // here as well; feed the carry in at bit 0.
        let odd_carries = self.wrapping_add(odd_starts).or(flip);
        // Carry out of the odd-start addition = a run that reached the
        // top with odd length.
        let overflow = self
            .and(odd_starts)
            .or(self.or(odd_starts).and(odd_carries.not()));
        let carry = overflow.bit(Self::BITS - 1);

        let even_carry_ends = even_carries.and(self.not());
        let odd_carry_ends = odd_carries.and(self.not());
        let even_start_odd_end = even_carry_ends.and(odd);
        let odd_start_even_end = odd_carry_ends.and(even);
        (even_start_odd_end.or(odd_start_even_end), carry)
    }
    /// Cells reachable from the set bits of `self` by repeatedly
    /// stepping `stride` positions upward into cells where `propagate`
    /// is set. `log₂(BITS / stride)` rounds of three operations.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// // A rook on a1 sliding north on an 8×8 board with a blocker on a5:
    /// // it reaches a2, a3, a4 and the blocker square itself.
    /// let rook = 1u64;
    /// let empty = !(1u64 << 32); // a5 occupied
    /// let attacks = rook.fill_up(empty, 8) << 8; // slide, then step once more
    /// assert_eq!(attacks & 0x0101_0101_0101_0101, 0x0000_0001_0101_0100);
    /// ```
    #[inline]
    #[must_use]
    fn fill_up(self, propagate: Self, stride: u32) -> Self {
        debug_assert!(
            stride >= 1 && stride < Self::BITS,
            "fill stride {stride} out of range"
        );
        let mut reach = self;
        let mut prop = propagate;
        let mut s = stride;
        while s < Self::BITS {
            reach = reach.or(prop.and(reach.shl(s)));
            prop = prop.and(prop.shl(s));
            s <<= 1;
        }
        reach
    }

    /// Mirror of [`fill_up`](Bits::fill_up), stepping downward.
    #[inline]
    #[must_use]
    fn fill_down(self, propagate: Self, stride: u32) -> Self {
        debug_assert!(
            stride >= 1 && stride < Self::BITS,
            "fill stride {stride} out of range"
        );
        let mut reach = self;
        let mut prop = propagate;
        let mut s = stride;
        while s < Self::BITS {
            reach = reach.or(prop.and(reach.shr(s)));
            prop = prop.and(prop.shr(s));
            s <<= 1;
        }
        reach
    }

    // ===================================================================
    // Compact / expand: PEXT / PDEP
    // ===================================================================
    /// Gathers the bits of `self` at the set positions of `mask` into
    /// the low `count_ones(mask)` bits, in order (PEXT).
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// assert_eq!(0b1001u32.compact(0b1010), 0b10);
    /// assert_eq!(0xF0u64.compact(0xFF), 0xF0);
    /// ```
    #[inline]
    #[must_use]
    fn compact(self, mask: Self) -> Self {
        self.pext(mask)
    }

    /// Scatters the low `count_ones(mask)` bits of `self` to the set
    /// positions of `mask`, in order (PDEP). Higher bits of `self`
    /// are ignored.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// assert_eq!(0b11u32.expand(0b1010), 0b1010);
    /// assert_eq!(0b01u32.expand(0b1010), 0b0010);
    /// ```
    #[inline]
    #[must_use]
    fn expand(self, mask: Self) -> Self {
        self.pdep(mask)
    }

    // ===================================================================
    // Basics: Hacker's Delight ch. 2, HAKMEM 175, powers of two
    // ===================================================================
    /// Only the lowest set bit (BLSI): `x & -x`; zero stays zero.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// assert_eq!(0b1011_0000u32.lowest_set_mask(), 0b0001_0000);
    /// assert_eq!(0u32.lowest_set_mask(), 0);
    /// ```
    #[inline]
    #[must_use]
    fn lowest_set_mask(self) -> Self {
        self.and(Self::ZERO.wrapping_sub(self))
    }

    /// Every bit strictly below the lowest set bit: `!x & (x − 1)`;
    /// all ones for zero.
    #[inline]
    #[must_use]
    fn below_lowest_set(self) -> Self {
        self.not().and(self.wrapping_sub(Self::ONE))
    }

    /// Every bit at or below the lowest set bit (BLSMSK): `x ^ (x − 1)`;
    /// all ones for zero.
    #[inline]
    #[must_use]
    fn up_to_lowest_set(self) -> Self {
        self.xor(self.wrapping_sub(Self::ONE))
    }

    /// Sets the lowest clear bit: `x | (x + 1)`; all-ones stays.
    #[inline]
    #[must_use]
    fn set_lowest_zero(self) -> Self {
        self.or(self.wrapping_add(Self::ONE))
    }

    /// Clears the lowest run of consecutive set bits:
    /// `((x | (x − 1)) + 1) & x`.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// assert_eq!(0b1011_0110u32.clear_lowest_run(), 0b1011_0000);
    /// ```
    #[inline]
    #[must_use]
    fn clear_lowest_run(self) -> Self {
        self.or(self.wrapping_sub(Self::ONE))
            .wrapping_add(Self::ONE)
            .and(self)
    }

    /// Bits of `other` where `mask` is set, bits of `self` elsewhere,
    /// without a branch: `x ^ ((x ^ y) & m)`.
    #[inline]
    #[must_use]
    fn blend(self, other: Self, mask: Self) -> Self {
        self.xor(self.xor(other).and(mask))
    }

    /// Gosper's hack (HAKMEM 175): the next larger integer with the
    /// same number of set bits, or `None` when there is none in the
    /// word (or `self` is zero). Iterating from `low_ones(k)`
    /// enumerates every `k`-subset of `BITS` in increasing order.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// let subsets: Vec<u8> = core::iter::successors(Some(0b0011u8), |x| x.next_same_popcount())
    ///     .take_while(|&x| x < 0b1_0000)
    ///     .collect();
    /// assert_eq!(subsets, [0b0011, 0b0101, 0b0110, 0b1001, 0b1010, 0b1100]);
    /// ```
    #[inline]
    #[must_use]
    fn next_same_popcount(self) -> Option<Self> {
        let s = self.lowest_set_mask();
        let r = self.wrapping_add(s);
        if r.is_zero() {
            // Zero input, or the lowest run of ones reached the top.
            return None;
        }
        // Ones cleared by the carry, minus one, moved back to the bottom.
        // Two shifts: `tzcnt + 2` may equal `BITS` when the run sits at
        // the top, and a single shift by `BITS` is out of range.
        let ones = self.xor(r).shr(self.trailing_zeros()).shr(2);
        Some(r.or(ones))
    }

    /// `true` for exactly one set bit.
    #[inline]
    #[must_use]
    fn is_pow2(self) -> bool {
        !self.is_zero() && self.clear_lowest_set().is_zero()
    }

    /// Largest power of two `<= self`; `None` for zero.
    #[inline]
    #[must_use]
    fn round_down_pow2(self) -> Option<Self> {
        self.last_set().map(|p| Self::ONE.shl(p))
    }

    /// Smallest power of two `>= self` (`1` for zero); `None` when it
    /// does not fit the word.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// assert_eq!(37u32.round_up_pow2(), Some(64));
    /// assert_eq!(64u32.round_up_pow2(), Some(64));
    /// assert_eq!(0u32.round_up_pow2(), Some(1));
    /// assert_eq!(0x8000_0001u32.round_up_pow2(), None);
    /// ```
    #[inline]
    #[must_use]
    fn round_up_pow2(self) -> Option<Self> {
        if self.shr(1).is_zero() {
            return Some(Self::ONE);
        }
        let lz = self.wrapping_sub(Self::ONE).leading_zeros();
        if lz == 0 {
            None
        } else {
            Some(Self::ONE.shl(Self::BITS - lz))
        }
    }

    /// `⌊log₂ self⌋`; `None` for zero.
    #[inline]
    #[must_use]
    fn log2_floor(self) -> Option<u32> {
        self.last_set()
    }

    /// `⌈log₂ self⌉`; `None` for zero.
    #[inline]
    #[must_use]
    fn log2_ceil(self) -> Option<u32> {
        if self.is_zero() {
            None
        } else {
            Some(Self::BITS - self.wrapping_sub(Self::ONE).leading_zeros())
        }
    }

    // ===================================================================
    // SWAR byte lanes: Hacker's Delight 6-1
    // ===================================================================
    /// `0x01` in every lane.
    #[inline]
    #[must_use]
    fn lanes_lo() -> Self {
        Self::splat_byte(0x01)
    }

    /// `0x80` in every lane.
    #[inline]
    #[must_use]
    fn lanes_hi() -> Self {
        Self::splat_byte(0x80)
    }

    /// High bit of every lane whose byte is zero; other bits clear.
    /// Exact (no borrow between lanes).
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// assert_eq!(0x0012_0034u32.zero_bytes(), 0x8000_8000);
    /// assert_eq!(0x0100u16.zero_bytes(), 0x0080);
    /// ```
    #[inline]
    #[must_use]
    fn zero_bytes(self) -> Self {
        let low7 = Self::splat_byte(0x7F);
        // Lane sum ≥ 0x80 iff its low seven bits are non-zero; OR in
        // the byte's own high bit; every non-zero byte now has bit 7.
        let y = self.and(low7).wrapping_add(low7).or(self).or(low7);
        y.not()
    }

    /// `true` when any lane is zero.
    #[inline]
    #[must_use]
    fn has_zero_byte(self) -> bool {
        !self.zero_bytes().is_zero()
    }

    /// Index of the lowest zero lane, if any.
    #[inline]
    #[must_use]
    fn first_zero_byte(self) -> Option<u32> {
        let z = self.zero_bytes();
        if z.is_zero() {
            None
        } else {
            Some(z.trailing_zeros() / 8)
        }
    }

    /// High bit of every lane equal to `b`.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// assert_eq!(0x41_42_41_43u32.bytes_eq(0x41), 0x80_00_80_00);
    /// ```
    #[inline]
    #[must_use]
    fn bytes_eq(self, b: u8) -> Self {
        self.xor(Self::splat_byte(b)).zero_bytes()
    }

    /// High bit of every lane whose byte is `>= n`, for `n` in `1..=128`.
    /// Exact.
    #[inline]
    #[must_use]
    fn bytes_ge(self, n: u8) -> Self {
        debug_assert!((1..=128).contains(&n), "bytes_ge: n={n} outside 1..=128");
        // low7 + (128 - n) reaches bit 7 iff low7 >= n; a byte >= 128
        // already carries bit 7 itself.
        let low7 = Self::splat_byte(0x7F);
        self.and(low7)
            .wrapping_add(Self::splat_byte(0x80 - n))
            .or(self)
            .and(Self::lanes_hi())
    }

    /// High bit of every lane whose byte is `< n`, for `n` in `1..=128`.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// // Control characters (< 0x20) in an ASCII word.
    /// assert_eq!(0x41_0A_42_09u32.bytes_lt(0x20), 0x00_80_00_80);
    /// ```
    #[inline]
    #[must_use]
    fn bytes_lt(self, n: u8) -> Self {
        self.bytes_ge(n).xor(Self::lanes_hi())
    }

    /// Number of lanes equal to `b`.
    #[inline]
    #[must_use]
    fn count_bytes_eq(self, b: u8) -> u32 {
        self.bytes_eq(b).count_ones()
    }

    /// Sum of all lanes; lanes must each be `< 256 / lanes` to avoid
    /// overflow, as produced by [`zero_bytes`](Bits::zero_bytes)-style
    /// masks shifted down, or per-lane popcounts.
    #[inline]
    #[must_use]
    fn sum_bytes(self) -> u32 {
        // Multiply by 0x01…01: the top lane accumulates every lane.
        u32::from(
            self.wrapping_mul(Self::lanes_lo())
                .shr(Self::BITS - 8)
                .low_byte(),
        )
    }

    // ===================================================================
    // Permutations: delta swap
    // ===================================================================
    /// Swaps the bits selected by `mask` with the bits `shift` positions
    /// above them. `mask` and `mask << shift` must be disjoint and
    /// `mask << shift` must not overflow (debug-asserted); the operation
    /// is then an involution.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// // Swap the low and high nibbles of a byte.
    /// assert_eq!(0xA5u8.delta_swap(0x0F, 4), 0x5A);
    /// ```
    #[inline]
    #[must_use]
    fn delta_swap(self, mask: Self, shift: u32) -> Self {
        debug_assert!(
            mask.and(mask.shl(shift)).is_zero() && mask.shl(shift).shr(shift) == mask,
            "delta_swap: mask and mask << shift must be disjoint and in range"
        );
        let t = self.xor(self.shr(shift)).and(mask);
        self.xor(t).xor(t.shl(shift))
    }
}

impl<W: Word> Bits for W {}

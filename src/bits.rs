//! The one trait to import: every combinator, on every carrier.
//!
//! `use hakmem::Bits;` and the whole algebra is available on `u8`,
//! `u16`, `u32`, `u64` and `u128` (the Itertools shape: one trait,
//! many methods, organised below by the circuit each method is).
//! Primitives (the [`Word`] trait) keep `std` names, `count_ones`,
//! `trailing_zeros`, `leading_zeros`, plus `pext` / `pdep` /
//! `select_lowest` / `xor_scan` for the hardware-backed ones.

use crate::set::{Positions, Subsets};
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
/// - **Multiply as shift-and-add**: a constant factor is a set of left shifts summed at once; when
///   the copies never meet it is a broadcast, a scan or a gather (Kindergarten bitboards).
/// - **Basics** (HD ch. 2, HAKMEM 175): lowest-set-bit family, blend, powers of two, Gosper's hack,
///   the carry-rippler over the subsets of a mask.
/// - **SWAR byte lanes** (HD 6-1): exact zero / equal / less-than tests on every byte at once; the
///   `strlen` / `memchr` sentences.
/// - **Permutations**: delta swap, the primitive of every Beneš network (8×8 board permutations
///   live in [`crate::permute::board8`]).
pub trait Bits: Word {
    // ===================================================================
    // Runs: Hacker's Delight 6-2 / 6-3
    // ===================================================================
    /// Bit `p` of the result is set iff bits `p..p + k` of `self` are
    /// all set. Total: `k = 0` starts everywhere (every bit), `k` above
    /// the width nowhere (zero).
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
        if k == 0 {
            return Self::ONES;
        }
        if k > Self::BITS {
            return Self::ZERO;
        }
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
    /// assert_eq!(x.rank(0), 0);
    /// assert_eq!(x.rank(2), 2);
    /// assert_eq!(x.rank(64), 3);
    /// ```
    #[inline]
    #[must_use]
    fn rank(self, i: u32) -> u32 {
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
    /// Every subset of `self` as a mask, ascending, starting at zero.
    /// See [`Subsets`].
    #[inline]
    fn subsets(self) -> Subsets<Self> {
        Subsets::new(self)
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
    /// is set. `log₂(BITS / stride)` rounds of three operations. On a
    /// board, an east, west or diagonal stride wraps across the edge
    /// unless the wrap file is masked out of `propagate`;
    /// [`crate::permute::board8::slide`] does that for the eight
    /// directions.
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
        if stride == 0 || stride >= Self::BITS {
            return self;
        }
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
        if stride == 0 || stride >= Self::BITS {
            return self;
        }
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
    // Multiply as shift-and-add: broadcast, scan, gather
    // ===================================================================
    /// The bits of `self & mask`, in order, as the low `count_ones(mask)`
    /// bits of the result, by one multiplication.
    ///
    /// Computes `(x & mask) * factor`, shifted right by `target` and
    /// masked. A multiply by a constant is the sum of `x` shifted left
    /// by each set bit of `factor`; when the shifted copies of the
    /// selected bits never meet, the sum is an OR and the multiply is a
    /// gather: PEXT by arithmetic, the Kindergarten bitboard of a file
    /// or a diagonal as a byte index. The crate's own [`Word::splat_byte`]
    /// (a broadcast) and the byte prefix sums of the broadword select (a
    /// scan) are the same instruction read two other ways.
    ///
    /// Whether a triple is exact is a property of `mask`, `factor` and
    /// `target`, not of the input: `laws::gather_is_exact` checks it
    /// over every subset of the mask, and
    /// `laws::strided_gather_is_exact` says when it must hold. When it
    /// is, the result equals [`compact`](Bits::compact); when it is not,
    /// carries corrupt it.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// // The a-file of a bitboard (bits 0, 8, .., 56) as one byte.
    /// let a_file = 0x0101_0101_0101_0101u64;
    /// let factor = u64::gather_factor(a_file, 56).unwrap();
    /// assert_eq!(factor, 0x0102_0408_1020_4080);
    /// let occupied = 0x0000_0100_0000_0101u64; // a1, a2, a6
    /// assert_eq!(occupied.gather(a_file, factor, 56), 0b0010_0011);
    /// assert_eq!(
    ///     occupied.gather(a_file, factor, 56),
    ///     occupied.compact(a_file)
    /// );
    /// ```
    #[inline]
    #[must_use]
    fn gather(self, mask: Self, factor: Self, target: u32) -> Self {
        if target >= Self::BITS {
            return Self::ZERO;
        }
        self.and(mask)
            .wrapping_mul(factor)
            .shr(target)
            .and(Self::low_ones(mask.count_ones()))
    }

    /// The factor that sends the `i`-th set bit of `mask` (ascending)
    /// to bit `place(i)`: one set bit per selected bit, at the distance
    /// it has to travel; bits that travel the same distance share it.
    /// `None` when a bit would have to move right or past the top,
    /// which a multiply cannot do. Existence is not exactness: the
    /// copies may still collide, see [`gather`](Bits::gather) and
    /// `laws::gather_is_exact_by`.
    ///
    /// The placement need not preserve order. A bitboard's
    /// antidiagonal read by column runs against bit order, and its
    /// Kindergarten factor is this with `place(i) = 56 + c_i`, the
    /// a-file again.
    #[must_use]
    fn gather_factor_by(mask: Self, place: impl Fn(u32) -> u32) -> Option<Self> {
        let mut factor = Self::ZERO;
        for (i, q) in (0..).zip(mask.positions()) {
            let t = place(i);
            if t < q || t >= Self::BITS {
                return None;
            }
            factor = factor.or(Self::ONE.shl(t - q));
        }
        Some(factor)
    }

    /// The factor that moves the `i`-th set bit of `mask` to bit
    /// `target + i`, in order: [`gather_factor_by`](Bits::gather_factor_by)
    /// with `place(i) = target + i`.
    ///
    /// For a mask whose bits are `stride` apart with `stride >=
    /// count_ones(mask)`, the gather is exact: every partial product
    /// lands on its own bit (`laws::strided_gather_is_exact`). Files
    /// (stride 8) and diagonals (stride 9) of a bitboard qualify; the
    /// factor for a diagonal read by column comes out as the a-file,
    /// `0x0101…01`, which is where the Kindergarten constants come from.
    #[inline]
    #[must_use]
    fn gather_factor(mask: Self, target: u32) -> Option<Self> {
        Self::gather_factor_by(mask, |i| target + i)
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

    /// Any Boolean function of three words, bit by bit, from its 8-bit
    /// truth table: bit `4 a + 2 b + c` of `table` is the result for
    /// input bits `a` (from `self`), `b`, `c`. This is VPTERNLOG's
    /// contract; the table of a function `f` is `f(0xF0, 0xCC, 0xAA)`
    /// ([`truth_table`]). Provided as a Shannon expansion, at most ten
    /// operations, which a compiler with AVX-512 folds back into the
    /// instruction.
    ///
    /// ```
    /// use hakmem::bits::truth_table;
    /// use hakmem::prelude::*;
    ///
    /// let majority = |a: u8, b: u8, c: u8| (a & b) | (a & c) | (b & c);
    /// assert_eq!(truth_table(majority), 0xE8);
    /// assert_eq!(0b1100u32.ternary(0b1010, 0b0110, 0xE8), 0b1110);
    /// ```
    #[inline]
    #[must_use]
    fn ternary(self, b: Self, c: Self, table: u8) -> Self {
        let leaf = |t: u8| match t & 3 {
            0 => Self::ZERO,
            1 => c.not(),
            2 => c,
            _ => Self::ONES,
        };
        let on_b = |t: u8| b.and(leaf(t >> 2)).or(b.not().and(leaf(t)));
        self.and(on_b(table >> 4)).or(self.not().and(on_b(table)))
    }

    /// Overflow of `self + other` read as two's complement, from the
    /// sign bits alone (Hacker's Delight 2-13): the operands agree in
    /// sign and the sum does not, so `!(x ^ y) & (x ^ (x + y))` has its
    /// top bit set. The carrier being unsigned does not matter; the test
    /// reads three bits. As a function of `(x, y, x + y)` its truth
    /// table is `0x42`: one VPTERNLOG for a register of lanes.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// assert!(100u8.signed_add_overflows(100));
    /// assert!(!100u8.signed_add_overflows(27));
    /// for a in 0..=255u8 {
    ///     for b in [0, 1, 0x7F, 0x80, 0xFF] {
    ///         let checked = (a as i8).checked_add(b as i8).is_none();
    ///         assert_eq!(a.signed_add_overflows(b), checked);
    ///     }
    /// }
    /// ```
    #[inline]
    #[must_use]
    fn signed_add_overflows(self, other: Self) -> bool {
        self.xor(other)
            .not()
            .and(self.xor(self.wrapping_add(other)))
            .bit(Self::BITS - 1)
    }

    /// Overflow of `self − other` read as two's complement: the operands
    /// differ in sign and the difference disagrees with `self`, so
    /// `(x ^ y) & (x ^ (x − y))` has its top bit set; truth table `0x18`.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// assert!(0x80u8.signed_sub_overflows(1)); // -128 - 1
    /// assert!(!0x80u8.signed_sub_overflows(0xFF)); // -128 - (-1)
    /// ```
    #[inline]
    #[must_use]
    fn signed_sub_overflows(self, other: Self) -> bool {
        self.xor(other)
            .and(self.xor(self.wrapping_sub(other)))
            .bit(Self::BITS - 1)
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

    /// The carry-rippler: the next subset of `mask` after `self`, in
    /// increasing order, or `None` after the last (the mask itself).
    /// `(x − mask) & mask`: subtracting the mask borrows through the
    /// selected bits exactly as adding one would carry through them if
    /// they were contiguous, so this is `+ 1` in the compacted domain,
    /// `expand(compact(x) + 1)`, without the PEXT / PDEP. Bits of
    /// `self` outside the mask are ignored.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// assert_eq!(0u8.next_subset(0b1010), Some(0b0010));
    /// assert_eq!(0b0010u8.next_subset(0b1010), Some(0b1000));
    /// assert_eq!(0b1000u8.next_subset(0b1010), Some(0b1010));
    /// assert_eq!(0b1010u8.next_subset(0b1010), None);
    /// ```
    #[inline]
    #[must_use]
    fn next_subset(self, mask: Self) -> Option<Self> {
        let s = self.and(mask);
        if s == mask {
            None
        } else {
            Some(s.wrapping_sub(mask).and(mask))
        }
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

/// The truth table of a Boolean function of three words: `f(0xF0, 0xCC, 0xAA)`.
///
/// This is the immediate VPTERNLOG and [`Bits::ternary`] take. Bit `k` of
/// those three bytes is bit 2, 1 and 0 of `k`, so across their eight bit
/// positions they enumerate the eight input combinations, and the
/// function evaluated once on them is its own table.
///
/// ```
/// use hakmem::bits::truth_table;
///
/// assert_eq!(truth_table(|a, b, c| (a & b) | (a & c) | (b & c)), 0xE8);
/// assert_eq!(truth_table(|a, b, s| !(a ^ b) & (a ^ s)), 0x42); // signed add overflows
/// assert_eq!(truth_table(|a, b, d| (a ^ b) & (a ^ d)), 0x18); // signed sub overflows
/// ```
#[inline]
#[must_use]
pub fn truth_table(f: impl Fn(u8, u8, u8) -> u8) -> u8 {
    f(0xF0, 0xCC, 0xAA)
}

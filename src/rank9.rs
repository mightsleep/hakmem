//! Rank and select in O(1) over a slice of words: Vigna's rank9
//! directory (2008) for rank, and a select inventory in the shape of
//! his select9 over the same block counts.
//!
//! The storage is yours. The bits are any `&[u64]`; the directory is
//! two `&[u64]` you size with [`Rank9::counts_len`] and
//! [`Rank9::select_len`] and fill once with [`Rank9::build`]. No
//! allocator, no copy of the bits, and the directory can live next to
//! them (a file, a static, an arena). Cost: two words per eight words
//! of bits for rank (25 %) and three per eight for select (37.5 %).
//! The select part is optional: pass an empty slice and `select` falls
//! back to a binary search over the block counts.
//!
//! Rank is one directory pair and one [`Bits::rank`]: the count
//! before the block, the packed count before the word, a masked
//! popcount. Select reads one inventory pair per 512 set bits and, by
//! how many blocks those 512 span, either compares packed 16-bit
//! block counts in one or two SWAR steps (dense spans) or reads the
//! answer straight from stored positions (sparse spans, where the
//! span is long enough to hold every position). Then one SWAR compare
//! of the packed 9-bit counts and one in-word [`Bits::select`]. No
//! loop depends on the data. The laws in [`crate::laws`] pin both to
//! the linear scans of [`Words`](crate::slice::Words).
//!
//! ```
//! use hakmem::rank9::Rank9;
//!
//! let bits = [0b1011u64, u64::MAX, 0];
//! let mut counts = vec![0; Rank9::counts_len(bits.len())];
//! let mut select = vec![0; Rank9::select_len(bits.len())];
//! let dir = Rank9::build(&bits, &mut counts, &mut select);
//!
//! assert_eq!(dir.count_ones(), 67);
//! assert_eq!(dir.rank(2), 2);
//! assert_eq!(dir.rank(64), 3);
//! assert_eq!(dir.select(2), Some(3));
//! assert_eq!(dir.select(3), Some(64));
//! assert_eq!(dir.select(67), None);
//! ```

use crate::bits::Bits;
use crate::word::Word;

const WORDS_PER_BLOCK: usize = 8;
const BLOCK_BITS: usize = WORDS_PER_BLOCK * 64;
/// One inventory entry per this many set bits.
const ONES_PER_ENTRY: usize = 512;
const FIELD: usize = 9;
const FIELD_MASK: u64 = (1u64 << FIELD) - 1;

/// Bit 0 of every 9-bit lane, and the top bit of every 9-bit lane.
const ONES_STEP_9: u64 = 0x0040_2010_0804_0201;
const MSBS_STEP_9: u64 = ONES_STEP_9 << 8;
/// Bit 0 of every 16-bit lane, and the top bit of every 16-bit lane.
const ONES_STEP_16: u64 = 0x0001_0001_0001_0001;
const MSBS_STEP_16: u64 = ONES_STEP_16 << 15;
/// A 16-bit lane no in-range count ever reaches: padding.
const PAD_16: u64 = 0x7FFF;

/// Per 9-bit lane: the top bit set iff `x <= y` (unsigned, full lane).
/// Hacker's Delight's lane compare, as in Vigna's rank9.
#[inline]
const fn uleq_step9(x: u64, y: u64) -> u64 {
    ((((y | MSBS_STEP_9) - (x & !MSBS_STEP_9)) | (x ^ y)) ^ (x & !y)) & MSBS_STEP_9
}

/// Per 16-bit lane: the top bit set iff `x <= y`, for lanes below
/// `2^15` on both sides, so no lane borrows from its neighbour.
#[inline]
const fn uleq_step16_small(x: u64, y: u64) -> u64 {
    ((y | MSBS_STEP_16) - x) & MSBS_STEP_16
}

/// Rank/select directory over a borrowed bit slice. See the module docs.
#[derive(Clone, Copy, Debug)]
pub struct Rank9<'a> {
    bits: &'a [u64],
    /// Per block `b` of eight words: `counts[2b]` is the number of set
    /// bits before the block; `counts[2b + 1]` packs, for `j` in `1..8`,
    /// the number of set bits before word `j` of the block, nine bits
    /// each at bit `9 (j - 1)`. Bit 63 stays clear, so shifting by
    /// `9 ((j - 1) & 7)` reads zero for `j = 0` without a branch. A
    /// trailing pair holds the total.
    counts: &'a [u64],
    /// `inventory[i]` is the block holding set bit number `512 i`; one
    /// trailing entry holds the number of blocks. Empty without the
    /// select part.
    inventory: &'a [u64],
    /// Two words per block. Entry `i` owns the words of its blocks,
    /// `2 inventory[i] .. 2 inventory[i + 1]`, and what they hold
    /// depends on that span: see [`Span`].
    pool: &'a [u64],
    /// Cached from the lengths and the trailing count pair.
    blocks: usize,
    ones: usize,
}

/// What an inventory entry stores about its 512 set bits, chosen by
/// the number of blocks `s` between it and the next entry. Every case
/// fits in the entry's `2 s` pool words.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Span {
    /// Up to four blocks: nothing stored, up to three compares of the
    /// block counts themselves.
    Tiny,
    /// `4..16` blocks: 16 lanes of 16-bit block counts relative to the
    /// entry's block, one SWAR compare over four words.
    Flat,
    /// `16..64` blocks: eight 16-bit counts every eight blocks, then
    /// eight per-block counts for each of those groups; two compares.
    Two,
    /// `64..128` blocks: the 16-bit position of every set bit, relative
    /// to the entry's block. The answer, no search.
    Pos16,
    /// `128..2^23` blocks: the same as 32-bit positions.
    Pos32,
    /// Longer: the pool would fit 64-bit positions, but a span of four
    /// gigabits between two set bits is not a case worth a code path.
    /// Binary search over the block counts.
    Search,
}

impl Span {
    const fn of(blocks: usize) -> Self {
        match blocks {
            0..=3 => Self::Tiny,
            4..=15 => Self::Flat,
            16..=63 => Self::Two,
            64..=127 => Self::Pos16,
            128..=0x7F_FFFF => Self::Pos32,
            _ => Self::Search,
        }
    }
}

impl<'a> Rank9<'a> {
    const fn blocks(words: usize) -> usize {
        words.div_ceil(WORDS_PER_BLOCK)
    }

    /// Words of `counts` that [`build`](Self::build) needs for `words`
    /// words of bits.
    #[must_use]
    pub const fn counts_len(words: usize) -> usize {
        2 * (Self::blocks(words) + 1)
    }

    /// Words of `select` that [`build`](Self::build) needs for `words`
    /// words of bits: three per block. Zero is accepted too and means
    /// no select inventory.
    #[must_use]
    pub const fn select_len(words: usize) -> usize {
        3 * Self::blocks(words) + 2
    }

    /// Fills `counts` and `select` for `bits` and returns the directory
    /// over them: one pass over the bits for the counts and the
    /// inventory, one over the inventory for the pool. An empty `select`
    /// builds the rank part only.
    ///
    /// ```
    /// use hakmem::rank9::Rank9;
    ///
    /// let bits = [0b1011u64, u64::MAX, 0];
    /// let mut counts = vec![0; Rank9::counts_len(bits.len())];
    /// let mut select = vec![0; Rank9::select_len(bits.len())];
    /// let dir = Rank9::build(&bits, &mut counts, &mut select);
    /// assert_eq!((dir.rank(64), dir.select(3)), (3, Some(64)));
    /// ```
    ///
    /// # Panics
    ///
    /// If `counts` is shorter than [`counts_len`](Self::counts_len), or
    /// `select` is neither empty nor at least [`select_len`](Self::select_len).
    pub fn build(bits: &'a [u64], counts: &'a mut [u64], select: &'a mut [u64]) -> Self {
        Self::fill(bits, counts, select);
        Self::from_parts(bits, counts, select)
    }

    /// The length checks [`build`](Self::build) and
    /// [`from_parts`](Self::from_parts) share, with one wording for both.
    fn check_lengths(bits: usize, counts: usize, select: usize) {
        assert!(
            counts >= Self::counts_len(bits),
            "counts needs {} words, has {counts}",
            Self::counts_len(bits),
        );
        assert!(
            select == 0 || select >= Self::select_len(bits),
            "select needs {} words or none, has {select}",
            Self::select_len(bits),
        );
    }

    /// [`build`](Self::build) without the view, for owners that keep the
    /// directories and view them later.
    fn fill(bits: &[u64], counts: &mut [u64], select: &mut [u64]) {
        let blocks = Self::blocks(bits.len());
        Self::check_lengths(bits.len(), counts.len(), select.len());
        let (inventory, pool) = if select.is_empty() {
            (&mut [][..], &mut [][..])
        } else {
            select.split_at_mut(blocks + 1)
        };

        let mut before = 0u64;
        let mut next_entry = 0u64;
        let mut entries = 0;
        for b in 0..blocks {
            let block = &bits[b * WORDS_PER_BLOCK..bits.len().min((b + 1) * WORDS_PER_BLOCK)];
            let mut packed = 0u64;
            let mut inside = 0u64;
            for (j, w) in block.iter().enumerate() {
                if j > 0 {
                    packed |= inside << (FIELD * (j - 1));
                }
                inside += u64::from(w.count_ones());
            }
            // A short last block: the missing words count as full, so a
            // select inside the block never lands on them.
            for j in block.len().max(1)..WORDS_PER_BLOCK {
                packed |= inside << (FIELD * (j - 1));
            }
            while !inventory.is_empty() && next_entry < before + inside {
                inventory[entries] = b as u64;
                entries += 1;
                next_entry += ONES_PER_ENTRY as u64;
            }
            counts[2 * b] = before;
            counts[2 * b + 1] = packed;
            before += inside;
        }
        counts[2 * blocks] = before;
        counts[2 * blocks + 1] = 0;
        if inventory.is_empty() {
            return;
        }
        inventory[entries] = blocks as u64;
        for i in 0..entries {
            let (lo, hi) = (
                Self::to_usize(inventory[i]),
                Self::to_usize(inventory[i + 1]),
            );
            Self::fill_entry(bits, counts, &mut pool[2 * lo..2 * hi], i, lo, hi);
        }
    }

    /// The pool words of entry `i`, whose set bits `512 i ..` start in
    /// block `lo` and whose successor entry starts in block `hi`.
    fn fill_entry(
        bits: &[u64],
        counts: &[u64],
        region: &mut [u64],
        i: usize,
        lo: usize,
        hi: usize,
    ) {
        region.fill(0);
        let base = counts[2 * lo];
        // Set bits before block `lo + 1 + j`, relative to block `lo`;
        // padding past the span.
        let rel = |j: usize| {
            if lo + 1 + j <= hi {
                counts[2 * (lo + 1 + j)] - base
            } else {
                PAD_16
            }
        };
        let lanes4 = |f: &dyn Fn(usize) -> u64| f(0) | f(1) << 16 | f(2) << 32 | f(3) << 48;
        match Span::of(hi - lo) {
            Span::Tiny | Span::Search => {}
            Span::Flat => {
                for (w, word) in region.iter_mut().take(4).enumerate() {
                    *word = lanes4(&|l| rel(4 * w + l));
                }
            }
            Span::Two => {
                // Words 0..2: before block `lo + 8 (g + 1)`, for g in 0..8.
                for (w, word) in region.iter_mut().take(2).enumerate() {
                    *word = lanes4(&|l| rel(8 * (4 * w + l) + 7));
                }
                // Words 2 + 2 g .. 4 + 2 g: before block `lo + 8 g + 1 + m`.
                for g in 0..8 {
                    for (w, word) in region.iter_mut().skip(2 + 2 * g).take(2).enumerate() {
                        *word = lanes4(&|l| rel(8 * g + 4 * w + l));
                    }
                }
            }
            Span::Pos16 | Span::Pos32 => {
                let wide = Span::of(hi - lo) == Span::Pos32;
                let start = lo * BLOCK_BITS;
                let first = i * ONES_PER_ENTRY;
                let mut ordinal = Self::to_usize(base);
                let mut stored = 0;
                let first_word = lo * WORDS_PER_BLOCK;
                let words = &bits[first_word..bits.len().min((hi + 1) * WORDS_PER_BLOCK)];
                'words: for (wi, w) in words.iter().enumerate().map(|(n, w)| (first_word + n, w)) {
                    for p in w.positions() {
                        if ordinal >= first {
                            let offset = (wi * 64 + p as usize - start) as u64;
                            if wide {
                                region[stored / 2] |= offset << (32 * (stored % 2));
                            } else {
                                region[stored / 4] |= offset << (16 * (stored % 4));
                            }
                            stored += 1;
                            if stored == ONES_PER_ENTRY {
                                break 'words;
                            }
                        }
                        ordinal += 1;
                    }
                }
            }
        }
    }

    /// The directory over `counts` and `select` that
    /// [`build`](Self::build) filled for these same `bits` earlier, kept
    /// or loaded back. The contents are trusted, the lengths checked:
    /// directories from other bits answer wrong without saying so.
    ///
    /// # Panics
    ///
    /// If `counts` is too short for `bits`, or `select` is neither empty
    /// nor long enough.
    #[must_use]
    pub fn from_parts(bits: &'a [u64], counts: &'a [u64], select: &'a [u64]) -> Self {
        let blocks = Self::blocks(bits.len());
        Self::check_lengths(bits.len(), counts.len(), select.len());
        let (inventory, pool) = if select.is_empty() {
            (&[][..], &[][..])
        } else {
            select.split_at(blocks + 1)
        };
        Self {
            bits,
            counts,
            inventory,
            pool,
            blocks,
            ones: Self::to_usize(counts[2 * blocks]),
        }
    }

    /// The bits this directory indexes.
    #[must_use]
    pub const fn bits(&self) -> &'a [u64] {
        self.bits
    }

    /// Length in bits.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.bits.len() * 64
    }

    /// `true` when there are no bits at all.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bits.is_empty()
    }

    /// Number of set bits.
    #[must_use]
    pub const fn count_ones(&self) -> usize {
        self.ones
    }

    /// `true` when the select inventory was built.
    #[must_use]
    pub const fn has_select(&self) -> bool {
        !self.inventory.is_empty()
    }

    /// Number of set bits at positions `< i`; `i` past the end counts
    /// them all.
    #[must_use]
    pub fn rank(&self, i: usize) -> usize {
        if i >= self.len() {
            return self.count_ones();
        }
        let (block, word) = (i / BLOCK_BITS, (i / 64) % WORDS_PER_BLOCK);
        let before_block = self.counts[2 * block];
        let before_word = Self::field(self.counts[2 * block + 1], word);
        // `i % 64 < 64` fits a u32.
        #[allow(clippy::cast_possible_truncation)]
        let in_word = self.bits[i / 64].rank((i % 64) as u32);
        Self::to_usize(before_block + before_word + u64::from(in_word))
    }

    /// Number of clear bits at positions `< i`; `i` past the end counts
    /// them all.
    #[must_use]
    pub fn rank0(&self, i: usize) -> usize {
        i.min(self.len()) - self.rank(i)
    }

    /// Position of set bit number `k` (0-based), if `k < count_ones()`.
    #[must_use]
    pub fn select(&self, k: usize) -> Option<usize> {
        if k >= self.ones {
            return None;
        }
        if self.inventory.is_empty() {
            return Some(self.select_in_block(self.search(0, self.blocks - 1, k), k));
        }
        let i = k / ONES_PER_ENTRY;
        let entry = &self.inventory[i..i + 2];
        let (lo, hi) = (Self::to_usize(entry[0]), Self::to_usize(entry[1]));
        let k64 = k as u64;
        // One bounds check per case: the slice the case reads, whole.
        let block = match Span::of(hi - lo) {
            Span::Tiny => {
                let next = &self.counts[2 * lo + 2..2 * hi + 2];
                lo + usize::from(next[0] <= k64)
                    + usize::from(hi - lo >= 2 && next[2] <= k64)
                    + usize::from(hi - lo >= 3 && next[4] <= k64)
            }
            Span::Flat => {
                let rem = (k64 - self.counts[2 * lo]) * ONES_STEP_16;
                let lanes = &self.pool[2 * lo..2 * lo + 4];
                let n: u32 = lanes
                    .iter()
                    .take((hi - lo).div_ceil(4))
                    .map(|&w| uleq_step16_small(w, rem).count_ones())
                    .sum();
                lo + n as usize
            }
            Span::Two => {
                let rem = (k64 - self.counts[2 * lo]) * ONES_STEP_16;
                let region = &self.pool[2 * lo..2 * lo + 18];
                let g = (uleq_step16_small(region[0], rem).count_ones()
                    + uleq_step16_small(region[1], rem).count_ones())
                    as usize;
                let group = &region[2 + 2 * g..4 + 2 * g];
                let off = (uleq_step16_small(group[0], rem).count_ones()
                    + uleq_step16_small(group[1], rem).count_ones())
                    as usize;
                lo + 8 * g + off
            }
            Span::Pos16 => {
                let n = k % ONES_PER_ENTRY;
                let offset = (self.pool[2 * lo + n / 4] >> (16 * (n % 4))) & 0xFFFF;
                return Some(lo * BLOCK_BITS + Self::to_usize(offset));
            }
            Span::Pos32 => {
                let n = k % ONES_PER_ENTRY;
                let offset = (self.pool[2 * lo + n / 2] >> (32 * (n % 2))) & 0xFFFF_FFFF;
                return Some(lo * BLOCK_BITS + Self::to_usize(offset));
            }
            Span::Search => self.search(lo, hi, k),
        };
        Some(self.select_in_block(block, k))
    }

    /// The highest block in `lo..=hi` whose count of set bits before it
    /// is `<= k`; `lo` must qualify.
    const fn search(&self, mut lo: usize, mut hi: usize, k: usize) -> usize {
        let k64 = k as u64;
        while lo < hi {
            let mid = lo + (hi - lo).div_ceil(2);
            if self.counts[2 * mid] <= k64 {
                lo = mid;
            } else {
                hi = mid - 1;
            }
        }
        lo
    }

    /// Position of set bit `k`, known to lie in `block`: one SWAR compare
    /// of the seven packed counts names the word, one in-word select
    /// the bit.
    fn select_in_block(&self, block: usize, k: usize) -> usize {
        let rem = k as u64 - self.counts[2 * block];
        let packed = self.counts[2 * block + 1];
        let word = uleq_step9(packed, rem * ONES_STEP_9).count_ones() as usize;
        let in_word = rem - Self::field(packed, word);
        // `in_word < 64` fits a u32.
        #[allow(clippy::cast_possible_truncation)]
        let bit = self.bits[block * WORDS_PER_BLOCK + word].select_lowest(in_word as u32);
        block * BLOCK_BITS + word * 64 + bit as usize
    }

    /// Set bits before word `j` of a block, from the packed counts; zero
    /// for `j = 0` through the clear top bit, no branch.
    #[inline]
    const fn field(packed: u64, j: usize) -> u64 {
        (packed >> (FIELD * (j.wrapping_sub(1) & 7))) & FIELD_MASK
    }

    /// Directory values are counts of bits in a slice, so they fit.
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    const fn to_usize(v: u64) -> usize {
        v as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uleq_step9_every_pair_in_every_lane() {
        for lane in 0..7 {
            for x in 0..512u64 {
                for y in 0..512u64 {
                    let got = uleq_step9(x << (9 * lane), y << (9 * lane)) >> (9 * lane + 8) & 1;
                    assert_eq!(got == 1, x <= y, "lane {lane}: {x} <= {y}");
                }
            }
        }
    }

    #[test]
    fn uleq_step9_lanes_do_not_interfere() {
        // Neighbouring lanes at the extremes must not borrow into each other.
        let x = 511 | 511 << 18 | 511 << 36 | 511 << 54;
        let y = 511 << 9 | 511 << 18 | 1 << 36 | 510 << 54;
        let expect = [false, true, true, true, false, true, false];
        let got = uleq_step9(x, y);
        for (lane, e) in expect.into_iter().enumerate() {
            assert_eq!(got >> (9 * lane + 8) & 1 == 1, e, "lane {lane}");
        }
    }

    #[test]
    fn uleq_step16_small_every_pair_sampled() {
        let mut s = 0x9E37_79B9u64;
        for _ in 0..200_000 {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            let (x, y) = (s & 0x7FFF, s >> 32 & 0x7FFF);
            for lane in 0..4 {
                let got =
                    uleq_step16_small(x << (16 * lane), y << (16 * lane)) >> (16 * lane + 15) & 1;
                assert_eq!(got == 1, x <= y, "lane {lane}: {x} <= {y}");
            }
        }
    }

    #[test]
    fn span_thresholds() {
        assert_eq!(Span::of(0), Span::Tiny);
        assert_eq!(Span::of(3), Span::Tiny);
        assert_eq!(Span::of(4), Span::Flat);
        assert_eq!(Span::of(15), Span::Flat);
        assert_eq!(Span::of(16), Span::Two);
        assert_eq!(Span::of(63), Span::Two);
        assert_eq!(Span::of(64), Span::Pos16);
        assert_eq!(Span::of(127), Span::Pos16);
        assert_eq!(Span::of(128), Span::Pos32);
        assert_eq!(Span::of(0x7F_FFFF), Span::Pos32);
        assert_eq!(Span::of(0x80_0000), Span::Search);
    }
}

/// [`Rank9`] with the directory allocated for you: one call from a bit
/// slice to rank and select. Behind the `alloc` feature, on by default; the view type
/// stays the whole API, this only owns its two buffers.
///
/// ```
/// # #[cfg(feature = "alloc")] {
/// use hakmem::rank9::Rank9Buf;
///
/// let bits = [0b1011u64, u64::MAX, 0];
/// let dir = Rank9Buf::new(&bits);
/// assert_eq!(dir.rank(64), 3);
/// assert_eq!(dir.select(3), Some(64));
/// assert_eq!(dir.view().count_ones(), 67);
/// # }
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[derive(Clone, Debug)]
pub struct Rank9Buf<'a> {
    bits: &'a [u64],
    counts: alloc::boxed::Box<[u64]>,
    select: alloc::boxed::Box<[u64]>,
}

#[cfg(feature = "alloc")]
impl<'a> Rank9Buf<'a> {
    /// Rank and select over `bits`.
    #[must_use]
    pub fn new(bits: &'a [u64]) -> Self {
        Self::build(bits, Rank9::select_len(bits.len()))
    }

    /// Rank only, at 25 % instead of 62.5 % of the bits; `select` falls
    /// back to a binary search.
    #[must_use]
    pub fn rank_only(bits: &'a [u64]) -> Self {
        Self::build(bits, 0)
    }

    fn build(bits: &'a [u64], select_words: usize) -> Self {
        let mut counts = alloc::vec![0; Rank9::counts_len(bits.len())].into_boxed_slice();
        let mut select = alloc::vec![0; select_words].into_boxed_slice();
        Rank9::fill(bits, &mut counts, &mut select);
        Self {
            bits,
            counts,
            select,
        }
    }

    /// The borrowed view, for anything not forwarded here.
    #[must_use]
    pub fn view(&self) -> Rank9<'_> {
        Rank9::from_parts(self.bits, &self.counts, &self.select)
    }

    /// See [`Rank9::rank`].
    #[must_use]
    pub fn rank(&self, i: usize) -> usize {
        self.view().rank(i)
    }

    /// See [`Rank9::rank0`].
    #[must_use]
    pub fn rank0(&self, i: usize) -> usize {
        self.view().rank0(i)
    }

    /// See [`Rank9::select`].
    #[must_use]
    pub fn select(&self, k: usize) -> Option<usize> {
        self.view().select(k)
    }

    /// See [`Rank9::count_ones`].
    #[must_use]
    pub fn count_ones(&self) -> usize {
        self.view().count_ones()
    }

    /// The directory's two buffers, to keep next to the bits.
    #[must_use]
    pub fn into_parts(self) -> (alloc::boxed::Box<[u64]>, alloc::boxed::Box<[u64]>) {
        (self.counts, self.select)
    }
}

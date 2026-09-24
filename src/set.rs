//! Set view: a word as a set of positions `0..BITS`.
//!
//! Rank (POPCNT under a mask), select (`k`-th set bit via PDEP + TZCNT,
//! the Pandey et al. 2017 construction) and first/last element
//! (TZCNT / LZCNT).

use crate::bits::Bits;
use crate::word::Word;

/// Iterator over the positions of set bits, lowest first (TZCNT +
/// BLSR per step); from the back, highest first (LZCNT).
///
/// ```
/// use hakmem::prelude::*;
///
/// let v: Vec<u32> = 0b1011_0000u64.positions().collect();
/// assert_eq!(v, [4, 5, 7]);
/// assert_eq!(0b1011_0000u64.positions().rev().next(), Some(7));
/// assert_eq!(0u32.positions().len(), 0);
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Positions<W: Word>(pub(crate) W);

impl<W: Word> Iterator for Positions<W> {
    type Item = u32;

    #[inline]
    fn next(&mut self) -> Option<u32> {
        let p = self.0.first_set()?;
        self.0 = self.0.clear_lowest_set();
        Some(p)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let n = self.0.count_ones() as usize;
        (n, Some(n))
    }
}

impl<W: Word> DoubleEndedIterator for Positions<W> {
    #[inline]
    fn next_back(&mut self) -> Option<u32> {
        let p = self.0.last_set()?;
        self.0 = self.0.xor(W::ONE.shl(p));
        Some(p)
    }
}

impl<W: Word> ExactSizeIterator for Positions<W> {}
impl<W: Word> core::iter::FusedIterator for Positions<W> {}

/// Iterator over the subsets of a mask, ascending, from zero to the mask.
///
/// The carry-rippler, [`Bits::next_subset`] per step; `2^count_ones(mask)`
/// items. The way a magic-bitboard table is filled, one occupancy subset
/// of the relevant squares at a time.
///
/// ```
/// use hakmem::prelude::*;
///
/// let v: Vec<u8> = 0b1010u8.subsets().collect();
/// assert_eq!(v, [0b0000, 0b0010, 0b1000, 0b1010]);
/// assert_eq!(0u32.subsets().count(), 1);
/// ```
#[derive(Clone, Debug)]
#[must_use = "iterators are lazy and do nothing unless consumed"]
pub struct Subsets<W: Word> {
    mask: W,
    next: Option<W>,
}

impl<W: Word> Subsets<W> {
    #[inline]
    pub(crate) const fn new(mask: W) -> Self {
        Self {
            mask,
            next: Some(W::ZERO),
        }
    }
}

impl<W: Word> Iterator for Subsets<W> {
    type Item = W;

    #[inline]
    fn next(&mut self) -> Option<W> {
        let current = self.next?;
        self.next = current.next_subset(self.mask);
        Some(current)
    }
}

impl<W: Word> core::iter::FusedIterator for Subsets<W> {}

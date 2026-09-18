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

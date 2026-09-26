//! The curves behind one interface, for code that should not care which.
//!
//! Z-order and Hilbert keys have the same shape: encode, decode, the
//! raw key and back, and on the plane the query side. An index written
//! against [`Curve2`] changes curve by changing a type, where the
//! inherent methods would have it rename `code` to `index` at every
//! call. The inherent methods stay, and win where both are in scope.
//!
//! ```
//! use hakmem::prelude::*;
//!
//! /// Sorted keys of the points, on whichever curve.
//! fn keys<C: Curve2<Word = u64>>(points: &[(u64, u64)]) -> Vec<u64> {
//!     let mut keys: Vec<u64> = points.iter().map(|&(x, y)| C::encode(x, y).key()).collect();
//!     keys.sort_unstable();
//!     keys
//! }
//!
//! /// How many of `keys` lie in the rectangle, a seek per range.
//! fn count<C: Curve2<Word = u64>>(keys: &[u64], x0: u64, x1: u64, y0: u64, y1: u64) -> usize {
//!     let mut ranges = [(0, 0); 16];
//!     C::cover(x0..=x1, y0..=y1, &mut ranges)
//!         .iter()
//!         .map(|&(a, b)| keys.partition_point(|&k| k <= b) - keys.partition_point(|&k| k < a))
//!         .sum()
//! }
//!
//! let points = [(10, 10), (11, 12), (500, 500), (12, 11)];
//! assert_eq!(
//!     count::<Morton2<u64>>(&keys::<Morton2<u64>>(&points), 8, 15, 8, 15),
//!     3
//! );
//! assert_eq!(
//!     count::<Hilbert2<u64>>(&keys::<Hilbert2<u64>>(&points), 8, 15, 8, 15),
//!     3
//! );
//! ```

use core::fmt::Debug;
use core::hash::Hash;
use core::ops::RangeBounds;

use crate::dilated::{Morton2, Morton3};
use crate::hilbert::Hilbert2;
use crate::hilbert3::Hilbert3;
use crate::word::Word;

/// A key of a 2D curve over the full width of a word: `LEVELS` levels,
/// a `2^LEVELS` square, keys ordered as the curve visits the cells.
pub trait Curve2: Copy + Ord + Hash + Debug {
    /// The word of the coordinates and the key.
    type Word: Word;
    /// Levels of the curve: half the bits of the word.
    const LEVELS: u32;

    /// The key of `(x, y)`; coordinate bits above `LEVELS` are dropped.
    #[must_use]
    fn encode(x: Self::Word, y: Self::Word) -> Self;
    /// The `(x, y)` of this key.
    #[must_use]
    fn decode(self) -> (Self::Word, Self::Word);
    /// The raw key, the word to sort and store.
    #[must_use]
    fn key(self) -> Self::Word;
    /// Wraps a raw key.
    #[must_use]
    fn from_key(key: Self::Word) -> Self;

    /// The keys of the rectangle as at most `out.len()` ranges: the part
    /// of `out` it filled. See [`Hilbert2::cover`].
    ///
    /// # Panics
    ///
    /// If the rectangle is not empty and `out` is.
    fn cover(
        x: impl RangeBounds<Self::Word>,
        y: impl RangeBounds<Self::Word>,
        out: &mut [(Self::Word, Self::Word)],
    ) -> &[(Self::Word, Self::Word)];

    /// Whether some cell of the rectangle has its key in `keys`. See
    /// [`Hilbert2::intersects`].
    #[must_use]
    fn intersects(
        keys: impl RangeBounds<Self::Word>,
        x: impl RangeBounds<Self::Word>,
        y: impl RangeBounds<Self::Word>,
    ) -> bool;
}

/// A key of a 3D curve over the full width of a word: `LEVELS` levels,
/// a `2^LEVELS` cube.
pub trait Curve3: Copy + Ord + Hash + Debug {
    /// The word of the coordinates and the key.
    type Word: Word;
    /// Levels of the curve: a third of the bits of the word.
    const LEVELS: u32;

    /// The key of `(x, y, z)`; coordinate bits above `LEVELS` are
    /// dropped.
    #[must_use]
    fn encode(x: Self::Word, y: Self::Word, z: Self::Word) -> Self;
    /// The `(x, y, z)` of this key.
    #[must_use]
    fn decode(self) -> (Self::Word, Self::Word, Self::Word);
    /// The raw key.
    #[must_use]
    fn key(self) -> Self::Word;
    /// Wraps a raw key.
    #[must_use]
    fn from_key(key: Self::Word) -> Self;
}

macro_rules! curve2 {
    ($($curve:ident . $key:ident . $from:ident);*) => {$(
        impl<W: Word> Curve2 for $curve<W> {
            type Word = W;
            const LEVELS: u32 = $curve::<W>::LEVELS;

            #[inline]
            fn encode(x: W, y: W) -> Self {
                $curve::encode(x, y)
            }
            #[inline]
            fn decode(self) -> (W, W) {
                $curve::decode(self)
            }
            #[inline]
            fn key(self) -> W {
                self.$key()
            }
            #[inline]
            fn from_key(key: W) -> Self {
                $curve::$from(key)
            }
            #[inline]
            fn cover(x: impl RangeBounds<W>, y: impl RangeBounds<W>, out: &mut [(W, W)]) -> &[(W, W)] {
                $curve::cover(x, y, out)
            }
            #[inline]
            fn intersects(keys: impl RangeBounds<W>, x: impl RangeBounds<W>, y: impl RangeBounds<W>) -> bool {
                $curve::intersects(keys, x, y)
            }
        }
    )*};
}

macro_rules! curve3 {
    ($($curve:ident . $key:ident . $from:ident);*) => {$(
        impl<W: Word> Curve3 for $curve<W> {
            type Word = W;
            const LEVELS: u32 = $curve::<W>::LEVELS;

            #[inline]
            fn encode(x: W, y: W, z: W) -> Self {
                $curve::encode(x, y, z)
            }
            #[inline]
            fn decode(self) -> (W, W, W) {
                $curve::decode(self)
            }
            #[inline]
            fn key(self) -> W {
                self.$key()
            }
            #[inline]
            fn from_key(key: W) -> Self {
                $curve::$from(key)
            }
        }
    )*};
}

curve2!(Morton2.code.from_code; Hilbert2.index.from_index);
curve3!(Morton3.code.from_code; Hilbert3.index.from_index);

// A Hilbert key and the Morton code of the same cell are the same point
// spelled twice; `From` both ways, as `from_morton` and `to_morton`.
macro_rules! same_cell {
    ($($hilbert:ident <-> $morton:ident);*) => {$(
        impl<W: Word> From<$morton<W>> for $hilbert<W> {
            #[inline]
            fn from(m: $morton<W>) -> Self {
                Self::from_morton(m)
            }
        }

        impl<W: Word> From<$hilbert<W>> for $morton<W> {
            #[inline]
            fn from(h: $hilbert<W>) -> Self {
                h.to_morton()
            }
        }
    )*};
}

same_cell!(Hilbert2 <-> Morton2; Hilbert3 <-> Morton3);

// A key prints as the number it is, `{:b}` being the useful one, and
// defaults to the first cell of the curve.
macro_rules! key_traits {
    ($($curve:ident . $key:ident . $from:ident);*) => {$(
        impl<W: Word> Default for $curve<W> {
            #[inline]
            fn default() -> Self {
                Self::$from(W::ZERO)
            }
        }

        impl<W: Word> core::fmt::Binary for $curve<W> {
            // Debug output, not a hot path.
            #[allow(clippy::missing_inline_in_public_items)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::Binary::fmt(&self.$key(), f)
            }
        }

        impl<W: Word> core::fmt::LowerHex for $curve<W> {
            // Debug output, not a hot path.
            #[allow(clippy::missing_inline_in_public_items)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::LowerHex::fmt(&self.$key(), f)
            }
        }

        impl<W: Word> core::fmt::UpperHex for $curve<W> {
            // Debug output, not a hot path.
            #[allow(clippy::missing_inline_in_public_items)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                core::fmt::UpperHex::fmt(&self.$key(), f)
            }
        }
    )*};
}

key_traits!(
    Morton2.code.from_code;
    Morton3.code.from_code;
    Hilbert2.index.from_index;
    Hilbert3.index.from_index
);

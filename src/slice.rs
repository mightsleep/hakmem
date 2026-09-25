//! Bit-set operations over a slice of words: the index-free layer
//! between single-word combinators and a real succinct structure.
//!
//! [`Words`] puts them on `[W]` itself, so a slice, an array or a `Vec`
//! of words answers `rank` and `select` the way a word does. Every
//! method is a linear scan over words with O(1) work per word;
//! positions are absolute bit indices (`word_index * BITS + bit`).
//! Build rank/select directories on top when the slice is large
//! ([`Rank9`](crate::rank9::Rank9)); use these directly when it is not,
//! or when the words change often.
//!
//! `count_ones`, `rank`, `select` and `for_each_position` ask the CPU
//! once per call and run under the best level it has
//! ([`crate::isa`]): POPCNT, PDEP and TZCNT in a build without flags,
//! three and a half times the SWAR speed at 1024 words and no slower at
//! one. The `_in` forms take a token instead, for code that has chosen.
//!
//! ```
//! use hakmem::prelude::*;
//!
//! let words = [0b1011u64, 1 << 63];
//! assert_eq!(words.count_ones(), 4);
//! assert_eq!(words.rank(64), 3);
//! assert_eq!(words.select(3), Some(127));
//! assert_eq!(words.next_set_from(4), Some(127));
//!
//! let mut words = [0u8; 2];
//! words.set_bit(9);
//! assert!(words.bit(9) && !words.bit(8));
//! words.clear_bit(9);
//! assert_eq!(words, [0, 0]);
//! ```

use crate::bits::Bits;
use crate::isa::Isa;
use crate::word::Word;

/// A slice of words as one long bit set, bit `i` of word `j` at
/// position `j * BITS + i`.
pub trait Words {
    /// The word the slice is made of.
    type Word: Word;

    /// Whether bit `i` is set.
    ///
    /// # Panics
    ///
    /// If `i` is past the last word.
    #[must_use]
    fn bit(&self, i: usize) -> bool;

    /// Sets bit `i`.
    ///
    /// # Panics
    ///
    /// If `i` is past the last word.
    fn set_bit(&mut self, i: usize);

    /// Clears bit `i`.
    ///
    /// # Panics
    ///
    /// If `i` is past the last word.
    fn clear_bit(&mut self, i: usize);

    /// Number of set bits in the whole slice.
    #[must_use]
    #[doc(alias("popcount"))]
    fn count_ones(&self) -> usize;

    /// Number of set bits at positions `< i`. `i` may equal the bit
    /// length; larger `i` counts everything.
    #[must_use]
    #[doc(alias("rank1"))]
    fn rank(&self, i: usize) -> usize;

    /// Position of the `k`-th set bit (from 0), if `k < count_ones`.
    #[must_use]
    #[doc(alias("select1"))]
    fn select(&self, k: usize) -> Option<usize>;

    /// Lowest set position `>= i`, if any: from `i` on, `i` included.
    #[must_use]
    fn next_set_from(&self, i: usize) -> Option<usize>;

    /// Lowest position where `k` consecutive set bits start, if any.
    /// `k` in `1..=BITS`; a run may straddle one word boundary.
    ///
    /// Per word: [`Bits::run_starts`] finds runs inside the word; a run
    /// crossing into the next word is detected from this word's leading
    /// ones and the next word's trailing ones, no wider window needed.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// // 60 free slots, then 4 allocated, then free: a 6-run must straddle.
    /// assert_eq!([u64::MAX >> 4, u64::MAX].find_run(6), Some(0));
    /// assert_eq!([0xFFu64 << 56, u64::MAX].find_run(16), Some(56));
    /// assert_eq!([0u64, 0].find_run(1), None);
    /// ```
    #[must_use]
    fn find_run(&self, k: u32) -> Option<usize>;

    /// Positions of every set bit, ascending, across the words: the
    /// selection vector of a bitmap.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// let v: Vec<usize> = [0b101u64, 1].positions().collect();
    /// assert_eq!(v, [0, 2, 64]);
    /// ```
    fn positions(&self) -> impl Iterator<Item = usize> + '_;

    /// Calls `f` with every set position, ascending: [`positions`](Self::positions)
    /// by internal iteration, so the whole walk runs under one
    /// instruction set and `f` inlines into it.
    ///
    /// ```
    /// use hakmem::prelude::*;
    ///
    /// let mut v = Vec::new();
    /// [0b101u64, 1].for_each_position(|p| v.push(p));
    /// assert_eq!(v, [0, 2, 64]);
    /// ```
    fn for_each_position(&self, f: impl FnMut(usize));

    /// [`count_ones`](Self::count_ones) compiled for `isa`: POPCNT on a
    /// level that has it, whatever the build.
    #[must_use]
    fn count_ones_in<I: Isa>(&self, isa: I) -> usize;

    /// [`rank`](Self::rank) compiled for `isa`.
    #[must_use]
    fn rank_in<I: Isa>(&self, i: usize, isa: I) -> usize;

    /// [`select`](Self::select) compiled for `isa`: POPCNT to find the
    /// word, PDEP and TZCNT to find the bit, on a level that has them.
    #[must_use]
    fn select_in<I: Isa>(&self, k: usize, isa: I) -> Option<usize>;

    /// [`for_each_position`](Self::for_each_position) compiled for `isa`.
    fn for_each_position_in<I: Isa>(&self, isa: I, f: impl FnMut(usize));
}

impl<W: Word> Words for [W] {
    type Word = W;

    // `i % BITS < BITS` fits a u32.
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn bit(&self, i: usize) -> bool {
        let bits = W::BITS as usize;
        self[i / bits].bit((i % bits) as u32)
    }

    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn set_bit(&mut self, i: usize) {
        let bits = W::BITS as usize;
        let w = &mut self[i / bits];
        *w = w.or(W::ONE.shl((i % bits) as u32));
    }

    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn clear_bit(&mut self, i: usize) {
        let bits = W::BITS as usize;
        let w = &mut self[i / bits];
        *w = w.and(W::ONE.shl((i % bits) as u32).not());
    }

    #[inline]
    fn count_ones(&self) -> usize {
        crate::dispatch!(|cpu| self.count_ones_in(cpu))
    }

    #[inline]
    fn rank(&self, i: usize) -> usize {
        crate::dispatch!(|cpu| self.rank_in(i, cpu))
    }

    #[inline]
    fn select(&self, k: usize) -> Option<usize> {
        crate::dispatch!(|cpu| self.select_in(k, cpu))
    }

    #[inline]
    fn for_each_position(&self, f: impl FnMut(usize)) {
        crate::dispatch!(|cpu| self.for_each_position_in(cpu, f));
    }

    #[inline]
    fn count_ones_in<I: Isa>(&self, isa: I) -> usize {
        isa.run(|_| self.iter().map(|w| w.count_ones() as usize).sum())
    }

    #[inline]
    fn rank_in<I: Isa>(&self, i: usize, isa: I) -> usize {
        isa.run(|_| {
            let bits = W::BITS as usize;
            let (full, rest) = (i / bits, i % bits);
            let head: usize = self
                .iter()
                .take(full)
                .map(|w| w.count_ones() as usize)
                .sum();
            // `rest < BITS` always fits a u32.
            #[allow(clippy::cast_possible_truncation)]
            let tail = self.get(full).map_or(0, |w| w.rank(rest as u32) as usize);
            head + tail
        })
    }

    #[inline]
    fn select_in<I: Isa>(&self, mut k: usize, isa: I) -> Option<usize> {
        isa.run(|cpu| {
            for (wi, &w) in self.iter().enumerate() {
                let n = w.count_ones() as usize;
                if k < n {
                    // `k < n <= BITS` fits a u32.
                    #[allow(clippy::cast_possible_truncation)]
                    let inner = w.select_lowest_in(k as u32, cpu) as usize;
                    return Some(wi * W::BITS as usize + inner);
                }
                k -= n;
            }
            None
        })
    }

    #[inline]
    fn for_each_position_in<I: Isa>(&self, isa: I, mut f: impl FnMut(usize)) {
        isa.run(|_| {
            let bits = W::BITS as usize;
            for (wi, &w) in self.iter().enumerate() {
                let mut x = w;
                while !x.is_zero() {
                    f(wi * bits + x.trailing_zeros() as usize);
                    x = x.clear_lowest_set();
                }
            }
        });
    }

    fn next_set_from(&self, i: usize) -> Option<usize> {
        let bits = W::BITS as usize;
        let (wi, bit) = (i / bits, i % bits);
        let first = self.get(wi)?;
        // `bit < BITS` fits a u32.
        #[allow(clippy::cast_possible_truncation)]
        let masked = first.and(W::low_ones(bit as u32).not());
        if let Some(p) = masked.first_set() {
            return Some(wi * bits + p as usize);
        }
        self[wi + 1..]
            .iter()
            .enumerate()
            .find_map(|(j, w)| w.first_set().map(|p| (wi + 1 + j) * bits + p as usize))
    }

    fn find_run(&self, k: u32) -> Option<usize> {
        debug_assert!(
            (1..=W::BITS).contains(&k),
            "run length {k} outside 1..={}",
            W::BITS
        );
        let bits = W::BITS as usize;
        for (wi, &w) in self.iter().enumerate() {
            if w.is_zero() {
                continue;
            }
            if let Some(s) = w.run_starts(k).first_set() {
                return Some(wi * bits + s as usize);
            }
            // No run fits inside `w`; the only remaining candidate starts
            // in w's top run of ones and finishes in the next word.
            let suffix = w.leading_ones();
            if suffix == 0 {
                continue;
            }
            let prefix = self.get(wi + 1).map_or(0, |n| n.trailing_ones());
            if suffix + prefix >= k {
                return Some(wi * bits + (W::BITS - suffix) as usize);
            }
        }
        None
    }

    fn positions(&self) -> impl Iterator<Item = usize> + '_ {
        let bits = W::BITS as usize;
        self.iter()
            .enumerate()
            .flat_map(move |(wi, w)| w.positions().map(move |p| wi * bits + p as usize))
    }
}

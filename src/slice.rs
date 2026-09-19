//! Bit-set operations over a slice of words: the index-free layer
//! between single-word combinators and a real succinct structure.
//!
//! Every function is a linear scan over words with O(1) work per word;
//! positions are absolute bit indices (`word_index * BITS + bit`).
//! Build rank/select directories on top when the slice is large; use
//! these directly when it is not, or when the words change often.

use crate::bits::Bits;
use crate::word::Word;

/// Number of set bits in the whole slice.
#[inline]
#[must_use]
pub fn popcount<W: Word>(words: &[W]) -> usize {
    words.iter().map(|w| w.count_ones() as usize).sum()
}

/// Number of set bits at positions `< i`. `i` may equal the slice's
/// bit length; larger `i` counts everything.
#[must_use]
pub fn rank<W: Word>(words: &[W], i: usize) -> usize {
    let bits = W::BITS as usize;
    let (full, rest) = (i / bits, i % bits);
    let head: usize = words
        .iter()
        .take(full)
        .map(|w| w.count_ones() as usize)
        .sum();
    // `rest < BITS` always fits a u32.
    #[allow(clippy::cast_possible_truncation)]
    let tail = words.get(full).map_or(0, |w| w.rank(rest as u32) as usize);
    head + tail
}

/// Position of the `k`-th set bit (0-indexed), if `k < popcount`.
#[must_use]
pub fn select<W: Word>(words: &[W], mut k: usize) -> Option<usize> {
    for (wi, &w) in words.iter().enumerate() {
        let n = w.count_ones() as usize;
        if k < n {
            // `k < n <= BITS` fits a u32.
            #[allow(clippy::cast_possible_truncation)]
            let inner = w.select_lowest(k as u32) as usize;
            return Some(wi * W::BITS as usize + inner);
        }
        k -= n;
    }
    None
}

/// Lowest set position `>= i`, if any.
#[must_use]
pub fn next_set_after<W: Word>(words: &[W], i: usize) -> Option<usize> {
    let bits = W::BITS as usize;
    let (wi, bit) = (i / bits, i % bits);
    let first = words.get(wi)?;
    // `bit < BITS` fits a u32.
    #[allow(clippy::cast_possible_truncation)]
    let masked = first.and(W::low_ones(bit as u32).not());
    if let Some(p) = masked.first_set() {
        return Some(wi * bits + p as usize);
    }
    words[wi + 1..]
        .iter()
        .enumerate()
        .find_map(|(j, w)| w.first_set().map(|p| (wi + 1 + j) * bits + p as usize))
}

/// Lowest position where `k` consecutive set bits start, if any.
/// `k` in `1..=BITS`; a run may straddle one word boundary.
///
/// Per word: [`Bits::run_starts`] finds runs inside the word; a run
/// crossing into the next word is detected from this word's leading
/// ones and the next word's trailing ones, no wider window needed.
///
/// ```
/// use hakmem::slice::find_run;
///
/// // 60 free slots, then 4 allocated, then free: a 6-run must straddle.
/// let words = [u64::MAX >> 4, u64::MAX];
/// assert_eq!(find_run(&words, 6), Some(0));
/// let words = [0xFFu64 << 56, u64::MAX];
/// assert_eq!(find_run(&words, 16), Some(56));
/// assert_eq!(find_run(&[0u64, 0], 1), None);
/// ```
#[must_use]
pub fn find_run<W: Word>(words: &[W], k: u32) -> Option<usize> {
    debug_assert!(
        (1..=W::BITS).contains(&k),
        "run length {k} outside 1..={}",
        W::BITS
    );
    let bits = W::BITS as usize;
    for (wi, &w) in words.iter().enumerate() {
        if w.is_zero() {
            continue;
        }
        if let Some(s) = w.run_starts(k).first_set() {
            return Some(wi * bits + s as usize);
        }
        // No run fits inside `w`; the only remaining candidate starts in
        // w's top run of ones and finishes in the next word.
        let suffix = w.leading_ones();
        if suffix == 0 {
            continue;
        }
        let prefix = words.get(wi + 1).map_or(0, |n| n.trailing_ones());
        if suffix + prefix >= k {
            return Some(wi * bits + (W::BITS - suffix) as usize);
        }
    }
    None
}

/// Positions of every set bit, ascending, across the words: the
/// selection vector of a bitmap.
///
/// ```
/// use hakmem::slice::positions;
///
/// let v: Vec<usize> = positions(&[0b101u64, 1]).collect();
/// assert_eq!(v, [0, 2, 64]);
/// ```
pub fn positions<W: Word>(words: &[W]) -> impl Iterator<Item = usize> + '_ {
    let bits = W::BITS as usize;
    words
        .iter()
        .enumerate()
        .flat_map(move |(wi, w)| w.positions().map(move |p| wi * bits + p as usize))
}

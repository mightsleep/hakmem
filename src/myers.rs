//! Bit-parallel edit distance (Myers 1999, Hyyrö's formulation).
//!
//! The dynamic-programming column of Levenshtein distance is encoded
//! as two bit-vectors of vertical deltas (`+1` / `−1`) and advanced one
//! text character per step with a handful of word operations. The
//! carry chain does the work: `((Eq & Pv) + Pv) ^ Pv` propagates
//! matches down the column exactly where the DP would. Patterns up to
//! `W::BITS` characters; one step is ~12 ops regardless of length.
//!
//! Two boundary conditions, one engine: [`distance_in`] charges for
//! every text character (global alignment), [`search`] lets a match
//! start anywhere (semi-global, approximate string matching).

use crate::word::Word;

/// Positions of each byte in the pattern, plus the pattern length.
struct Peq<W: Word> {
    table: [W; 256],
    m: u32,
}

impl<W: Word> Peq<W> {
    /// `None` when the pattern is longer than the word.
    fn new(pattern: &[u8]) -> Option<Self> {
        let m = u32::try_from(pattern.len())
            .ok()
            .filter(|&m| m <= W::BITS)?;
        Some(Self::fill(pattern.iter().copied(), m))
    }

    /// The same pattern read backwards, for walking a text from an end
    /// towards its start. Same length, so it fits.
    fn reversed(&self, pattern: &[u8]) -> Self {
        debug_assert_eq!(pattern.len(), self.m as usize, "not this table's pattern");
        Self::fill(pattern.iter().rev().copied(), self.m)
    }

    /// The table of `m <= BITS` bytes.
    fn fill(pattern: impl Iterator<Item = u8>, m: u32) -> Self {
        let mut table = [W::ZERO; 256];
        for (i, c) in pattern.enumerate() {
            // `i < m <= BITS`.
            #[allow(clippy::cast_possible_truncation)]
            let bit = W::ONE.shl(i as u32);
            table[usize::from(c)] = table[usize::from(c)].or(bit);
        }
        Self { table, m }
    }
}

/// The column state: vertical positive / negative deltas.
#[derive(Clone, Copy)]
struct Column<W: Word> {
    pv: W,
    mv: W,
    top: W,
}

impl<W: Word> Column<W> {
    fn new(m: u32) -> Self {
        Self {
            pv: W::ONES,
            mv: W::ZERO,
            top: W::ONE.shl(m - 1),
        }
    }

    /// Advances one text character; returns the change of the bottom
    /// cell (`+1`, `0` or `−1`). `charge_start` is the DP boundary
    /// `D[0][j] = j` (global) versus `0` (semi-global).
    #[inline]
    fn step(&mut self, eq: W, charge_start: bool) -> i32 {
        let xv = eq.or(self.mv);
        let xh = eq.and(self.pv).wrapping_add(self.pv).xor(self.pv).or(eq);
        let ph = self.mv.or(xh.or(self.pv).not());
        let mh = self.pv.and(xh);

        let delta = if !ph.and(self.top).is_zero() {
            1
        } else if !mh.and(self.top).is_zero() {
            -1
        } else {
            0
        };

        let ph = if charge_start {
            ph.shl(1).or(W::ONE)
        } else {
            ph.shl(1)
        };
        let mh = mh.shl(1);
        self.pv = mh.or(xv.or(ph).not());
        self.mv = ph.and(xv);
        delta
    }
}

/// Levenshtein distance between `pattern` and `text`, or `None` when
/// the pattern is longer than `W::BITS`.
///
/// ```
/// use hakmem::myers::distance_in;
///
/// assert_eq!(distance_in::<u64>(b"kitten", b"sitting"), Some(3));
/// assert_eq!(distance_in::<u64>(b"", b"abc"), Some(3));
/// assert_eq!(distance_in::<u8>(b"123456789", b""), None);
/// ```
#[must_use]
pub fn distance_in<W: Word>(pattern: &[u8], text: &[u8]) -> Option<u32> {
    let peq = Peq::<W>::new(pattern)?;
    if peq.m == 0 {
        return u32::try_from(text.len()).ok();
    }
    let mut col = Column::new(peq.m);
    let mut score = i64::from(peq.m);
    for &c in text {
        score += i64::from(col.step(peq.table[usize::from(c)], true));
    }
    u32::try_from(score).ok()
}

/// Approximate occurrences of `pattern` in `text`.
///
/// Yields every end position `end` (exclusive) at which some substring
/// of `text[..end]` ending there is within `max_dist` edits of the
/// pattern, with that distance. Semi-global: the match may start
/// anywhere. `None` when the pattern is empty or wider than the word.
///
/// ```
/// use hakmem::myers::search;
///
/// // Exact: each occurrence ends once.
/// let exact: Vec<(usize, u32)> = search::<u64>(b"lo", b"hello lo", 0).unwrap().collect();
/// assert_eq!(exact, [(5, 0), (8, 0)]);
/// // Two edits allowed: "helo" is one from "hello", and "hel" and "helo "
/// // are two, so the one occurrence ends three times.
/// let ends: Vec<(usize, u32)> = search::<u64>(b"hello", b"say helo there", 2)
///     .unwrap()
///     .collect();
/// assert_eq!(ends, [(7, 2), (8, 1), (9, 2)]);
/// ```
///
/// A run of ends is one occurrence; [`Search::occurrences`] keeps its
/// best end and finds where it starts.
#[must_use]
pub fn search<'t, W: Word>(
    pattern: &'t [u8],
    text: &'t [u8],
    max_dist: u32,
) -> Option<Search<'t, W>> {
    let peq = Peq::<W>::new(pattern)?;
    if peq.m == 0 {
        return None;
    }
    let m = i64::from(peq.m);
    let col = Column::new(peq.m);
    Some(Search {
        pattern,
        peq,
        col,
        score: m,
        text,
        pos: 0,
        max_dist: i64::from(max_dist),
    })
}

/// Smallest edit distance between `pattern` and any substring of
/// `text` (including the empty one, so at most `pattern.len()`).
#[must_use]
pub fn substring_distance<W: Word>(pattern: &[u8], text: &[u8]) -> Option<u32> {
    let peq = Peq::<W>::new(pattern)?;
    if peq.m == 0 {
        return Some(0);
    }
    let mut col = Column::new(peq.m);
    let mut score = i64::from(peq.m);
    let mut best = score;
    for &c in text {
        score += i64::from(col.step(peq.table[usize::from(c)], false));
        best = best.min(score);
    }
    u32::try_from(best).ok()
}

/// Iterator behind [`search`].
#[must_use = "iterators are lazy; this one has not read a byte"]
pub struct Search<'t, W: Word> {
    pattern: &'t [u8],
    peq: Peq<W>,
    col: Column<W>,
    score: i64,
    text: &'t [u8],
    pos: usize,
    max_dist: i64,
}

// The pattern tables are 256 words of noise to a reader; where the
// search stands is what a failing test wants to see.
impl<W: Word> core::fmt::Debug for Search<'_, W> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Search")
            .field("pos", &self.pos)
            .field("text_len", &self.text.len())
            .field("score", &self.score)
            .field("max_dist", &self.max_dist)
            .finish_non_exhaustive()
    }
}

impl<W: Word> Iterator for Search<'_, W> {
    type Item = (usize, u32);

    fn next(&mut self) -> Option<(usize, u32)> {
        while let Some(&c) = self.text.get(self.pos) {
            self.pos += 1;
            self.score += i64::from(self.col.step(self.peq.table[usize::from(c)], false));
            if self.score <= self.max_dist {
                // `score` is in `0..=m`, fits a u32.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                return Some((self.pos, self.score as u32));
            }
        }
        None
    }
}

impl<W: Word> core::iter::FusedIterator for Search<'_, W> {}

impl<'t, W: Word> Search<'t, W> {
    /// The occurrences behind the ends: one per run of adjacent ends,
    /// at its best end, with where it starts.
    ///
    /// An approximate match ends at several neighbouring positions (one
    /// more or one fewer byte is one more edit, often still within
    /// `max_dist`); this keeps the least distance of each run, the first
    /// on a tie. The start comes from walking back from that end with
    /// the pattern reversed: after `j` bytes the score is the distance of
    /// the pattern from `text[end - j..end]`, and the start is the `j`
    /// with the least, the one nearest the pattern's length on a tie.
    ///
    /// ```
    /// use hakmem::myers::search;
    ///
    /// let text = b"the quick brown fox and the quikc brown fox";
    /// let found: Vec<_> = search::<u64>(b"quick brown fox", text, 2)
    ///     .unwrap()
    ///     .occurrences()
    ///     .map(|o| (&text[o.range()], o.distance()))
    ///     .collect();
    /// assert_eq!(
    ///     found,
    ///     [(&b"quick brown fox"[..], 0), (&b"quikc brown fox"[..], 2)]
    /// );
    /// ```
    pub fn occurrences(self) -> Occurrences<'t, W> {
        let reversed = self.peq.reversed(self.pattern);
        Occurrences {
            search: self,
            reversed,
            best: None,
            run_end: 0,
        }
    }
}

/// One approximate occurrence: `text[start..end]` is `distance` edits
/// from the pattern.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Occurrence {
    start: usize,
    end: usize,
    distance: u32,
}

impl Occurrence {
    /// Where the occurrence starts in the text.
    #[must_use]
    pub const fn start(&self) -> usize {
        self.start
    }

    /// Where it ends, exclusive.
    #[must_use]
    pub const fn end(&self) -> usize {
        self.end
    }

    /// Edits between `text[start..end]` and the pattern.
    #[must_use]
    pub const fn distance(&self) -> u32 {
        self.distance
    }

    /// `start..end`, to index the text with.
    #[must_use]
    pub const fn range(&self) -> core::ops::Range<usize> {
        self.start..self.end
    }
}

/// Iterator behind [`Search::occurrences`].
#[must_use = "iterators are lazy; this one has not read a byte"]
pub struct Occurrences<'t, W: Word> {
    search: Search<'t, W>,
    reversed: Peq<W>,
    /// The best end of the run under way, and its distance.
    best: Option<(usize, u32)>,
    /// The last end of that run.
    run_end: usize,
}

impl<W: Word> Occurrences<'_, W> {
    /// The occurrence with its best end at `end`, `distance` edits away.
    fn finish(&self, (end, distance): (usize, u32)) -> Occurrence {
        let m = self.reversed.m as usize;
        // An alignment of `distance` edits spans at most `m + distance`
        // bytes of text.
        let lo = end.saturating_sub(m + distance as usize);
        let mut col = Column::new(self.reversed.m);
        let mut score = i64::from(self.reversed.m);
        let (mut best, mut len) = (score, 0usize);
        for (j, &c) in self.search.text[lo..end].iter().rev().enumerate() {
            score += i64::from(col.step(self.reversed.table[usize::from(c)], true));
            let j = j + 1;
            if score < best || (score == best && j.abs_diff(m) < len.abs_diff(m)) {
                (best, len) = (score, j);
            }
        }
        debug_assert_eq!(
            best,
            i64::from(distance),
            "the walk back disagrees with the walk forward"
        );
        Occurrence {
            start: end - len,
            end,
            distance,
        }
    }
}

impl<W: Word> Iterator for Occurrences<'_, W> {
    type Item = Occurrence;

    fn next(&mut self) -> Option<Occurrence> {
        for (end, distance) in self.search.by_ref() {
            match self.best {
                // The run goes on: keep its least distance, first on a tie.
                Some((_, d)) if end == self.run_end + 1 => {
                    if distance < d {
                        self.best = Some((end, distance));
                    }
                    self.run_end = end;
                }
                _ => {
                    let done = self.best.replace((end, distance));
                    self.run_end = end;
                    if let Some(done) = done {
                        return Some(self.finish(done));
                    }
                }
            }
        }
        self.best.take().map(|done| self.finish(done))
    }
}

impl<W: Word> core::iter::FusedIterator for Occurrences<'_, W> {}

impl<W: Word> core::fmt::Debug for Occurrences<'_, W> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Occurrences")
            .field("search", &self.search)
            .field("best", &self.best)
            .finish_non_exhaustive()
    }
}

/// [`distance_in`] with the carrier chosen by the pattern's length.
///
/// `u64` up to 64 bytes, `u128` to 128, [`Wide<4>`](crate::Wide) to
/// 256, `Wide<8>` to 512, `None` beyond. The generic functions are for
/// callers who know their lengths; this is for the rest.
///
/// ```
/// use hakmem::myers::distance;
///
/// assert_eq!(distance(b"kitten", b"sitting"), Some(3));
/// let long = [b'a'; 300];
/// assert_eq!(distance(&long, &long[..290]), Some(10));
/// assert_eq!(distance(&[0; 513], b""), None);
/// ```
#[must_use]
pub fn distance(pattern: &[u8], text: &[u8]) -> Option<u32> {
    match pattern.len() {
        0..=64 => distance_in::<u64>(pattern, text),
        65..=128 => distance_in::<u128>(pattern, text),
        129..=256 => distance_in::<crate::Wide<4>>(pattern, text),
        257..=512 => distance_in::<crate::Wide<8>>(pattern, text),
        _ => None,
    }
}

//! Bit-parallel edit distance (Myers 1999, Hyyrö's formulation).
//!
//! The dynamic-programming column of Levenshtein distance is encoded
//! as two bit-vectors of vertical deltas (`+1` / `−1`) and advanced one
//! text character per step with a handful of word operations. The
//! carry chain does the work: `((Eq & Pv) + Pv) ^ Pv` propagates
//! matches down the column exactly where the DP would. Patterns up to
//! `W::BITS` characters; one step is ~12 ops regardless of length.
//!
//! Two boundary conditions, one engine: [`edit_distance`] charges for
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
        let m = u32::try_from(pattern.len()).ok()?;
        if m > W::BITS {
            return None;
        }
        let mut table = [W::ZERO; 256];
        for (i, &c) in pattern.iter().enumerate() {
            // `i < m <= BITS` by the check above.
            #[allow(clippy::cast_possible_truncation)]
            let bit = W::ONE.shl(i as u32);
            table[usize::from(c)] = table[usize::from(c)].or(bit);
        }
        Some(Self { table, m })
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
/// use hakmem::myers::edit_distance;
///
/// assert_eq!(edit_distance::<u64>(b"kitten", b"sitting"), Some(3));
/// assert_eq!(edit_distance::<u64>(b"", b"abc"), Some(3));
/// assert_eq!(edit_distance::<u8>(b"123456789", b""), None);
/// ```
#[must_use]
pub fn edit_distance<W: Word>(pattern: &[u8], text: &[u8]) -> Option<u32> {
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
/// let hits: Vec<(usize, u32)> = search::<u64>(b"world", b"hello wrold!", 1)
///     .unwrap()
///     .collect();
/// // "wrold" matches with one transposition = two edits at distance 2;
/// // at distance ≤ 1 the closest ends are after "wro"…"wrold" partials.
/// assert!(hits.iter().all(|&(_, d)| d <= 1));
/// let exact: Vec<(usize, u32)> = search::<u64>(b"lo", b"hello lo", 0).unwrap().collect();
/// assert_eq!(exact, [(5, 0), (8, 0)]);
/// ```
#[must_use]
pub fn search<'t, W: Word>(pattern: &[u8], text: &'t [u8], max_dist: u32) -> Option<Search<'t, W>> {
    let peq = Peq::<W>::new(pattern)?;
    if peq.m == 0 {
        return None;
    }
    let m = i64::from(peq.m);
    let col = Column::new(peq.m);
    Some(Search {
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
pub fn min_distance<W: Word>(pattern: &[u8], text: &[u8]) -> Option<u32> {
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
pub struct Search<'t, W: Word> {
    peq: Peq<W>,
    col: Column<W>,
    score: i64,
    text: &'t [u8],
    pos: usize,
    max_dist: i64,
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

/// [`edit_distance`] with the carrier chosen by the pattern's length.
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
        0..=64 => edit_distance::<u64>(pattern, text),
        65..=128 => edit_distance::<u128>(pattern, text),
        129..=256 => edit_distance::<crate::Wide<4>>(pattern, text),
        257..=512 => edit_distance::<crate::Wide<8>>(pattern, text),
        _ => None,
    }
}

//! Rectangles as ranges of keys on a quadrant-recursive curve: the query
//! side of an index built by sorting on `Morton2` or `Hilbert2` codes.
//!
//! The exact cover of a rectangle has about one range per cell of its
//! boundary, far more than a scan wants to restart, so the caller sets a
//! budget: the length of the output. Two things keep to it.
//!
//! The depth: below some level `s` a node across the edge is taken
//! whole, which is the exact cover of the rectangle rounded out to
//! multiples of `2^s`. Its number of runs is counted without visiting a
//! node: a run starts at a cell of the rectangle whose predecessor on the
//! curve is not in it, and each curve has a closed form for those (see
//! `Quadrants::runs`). The count never grows with `s` (each run of a
//! finer cover lies in one run of a coarser), so an estimate from the
//! perimeter and a step or two either way find the least `s` whose cover
//! has at most twice the budget.
//!
//! Then the merge. The cover at that depth is walked twice, once to
//! record its gaps into the output as scratch, once to write the runs,
//! closing every gap below the `runs - budget`-th smallest. What remains
//! are the `budget - 1` largest gaps, the least over-cover any `budget`
//! ranges of that cover can have. The threshold is found as an offset
//! allocator finds a free block: by class, the bit length and the three
//! bits under the leading one. Gaps are sums of a few node sizes, so
//! the class of the threshold nearly always holds one value.
//!
//! The walk goes three levels at a time: the 64 descendants of a node
//! three levels down as one `u64` in curve order, those meeting the
//! rectangle the AND of a mask of its columns and a mask of its rows,
//! each looked up per frame. A run of ones in a mask is a range of keys,
//! and only the partial children are descended into.
//!
//! `intersects` asks the other question, whether a range of keys holds
//! a cell of the rectangle, by the same masks: the range too is two
//! runs of bits over the descendants, and at most two of them go down.

// `unreachable_pub` wants `pub(crate)` here and clippy wants `pub`; the
// rustc lint is the one the crate chose.
#![allow(clippy::redundant_pub_crate)]

use core::cmp::Ordering;

use crate::word::Word;

/// A quadrant-recursive curve as the cover sees it.
pub(crate) trait Quadrants {
    /// What [`runs`](Self::runs) wants to know about the rectangle once,
    /// at every depth.
    type Context<W>;

    /// The digit of quadrant `(dx, dy)` of a node in `frame`, and the
    /// frame of that quadrant.
    fn digit(frame: u8, dx: u8, dy: u8) -> (u8, u8);
    /// The descendant `k` levels down, `1 ≤ k ≤ 3`, whose `k` digits
    /// read `p`: its column and row in `0..2^k` and its frame.
    fn child(k: u32, frame: u8, p: u32) -> (u8, u8, u8);
    /// Of the `4^k` descendants `k` levels down, those in one of the
    /// columns `cols` and one of the rows `rows`, as bit `p` for digits
    /// `p`.
    fn cells(k: u32, frame: u8, cols: usize, rows: usize) -> u64;

    fn context<W: Word + Ord>(r: &Rect<W>) -> Self::Context<W>;
    /// The number of runs of the exact cover of `r` rounded out to
    /// multiples of `2^s`, on a curve of `levels` levels whose top frame
    /// is 0.
    fn runs<W: Word + Ord>(levels: u32, r: &Rect<W>, s: u32, context: &Self::Context<W>) -> W;
}

/// The tables behind [`Quadrants::child`] and [`Quadrants::cells`] for
/// a curve of `F` frames, built from its one-level digit table; about
/// 5 KB a frame. Subtrees of `k` levels sit at offset `CHILD[k]` in the
/// child tables and `COLS[k]` in the column and row tables.
pub(crate) struct Masks<const F: usize> {
    child: [[(u8, u8, u8); 84]; F],
    cols: [[u64; 276]; F],
    rows: [[u64; 276]; F],
}

const CHILD: [usize; 4] = [0, 0, 4, 20];
const COLS: [usize; 4] = [0, 0, 4, 20];

impl<const F: usize> Masks<F> {
    /// From `digit[frame][dx | dy << 1] = (digit, frame below)`.
    #[allow(clippy::cast_possible_truncation, clippy::many_single_char_names)]
    pub(crate) const fn build(digit: [[(u8, u8); 4]; F]) -> Self {
        let mut t = Self {
            child: [[(0, 0, 0); 84]; F],
            cols: [[0; 276]; F],
            rows: [[0; 276]; F],
        };
        let mut k = 1;
        while k <= 3 {
            let side = 1u8 << k;
            let mut f = 0;
            while f < F {
                // The cells of each column and each row, then every set.
                let (mut col, mut row) = ([0u64; 8], [0u64; 8]);
                let mut cx = 0;
                while cx < side {
                    let mut cy = 0;
                    while cy < side {
                        let (mut p, mut g) = (0, f as u8);
                        let mut l = k;
                        while l > 0 {
                            l -= 1;
                            let q = ((cx >> l) & 1) | ((cy >> l) & 1) << 1;
                            let (d, below) = digit[g as usize][q as usize];
                            p = p << 2 | d as usize;
                            g = below;
                        }
                        t.child[f][CHILD[k] + p] = (cx, cy, g);
                        col[cx as usize] |= 1 << p;
                        row[cy as usize] |= 1 << p;
                        cy += 1;
                    }
                    cx += 1;
                }
                let mut m = 0;
                while m < 1 << side {
                    let mut c = 0;
                    while c < side as usize {
                        if m >> c & 1 == 1 {
                            t.cols[f][COLS[k] + m] |= col[c];
                            t.rows[f][COLS[k] + m] |= row[c];
                        }
                        c += 1;
                    }
                    m += 1;
                }
                f += 1;
            }
            k += 1;
        }
        t
    }

    #[inline]
    pub(crate) const fn child(&self, k: u32, frame: u8, p: u32) -> (u8, u8, u8) {
        self.child[frame as usize][CHILD[k as usize] + p as usize]
    }

    #[inline]
    pub(crate) const fn cells(&self, k: u32, frame: u8, cols: usize, rows: usize) -> u64 {
        let (f, o) = (frame as usize, COLS[k as usize]);
        self.cols[f][o + cols] & self.rows[f][o + rows]
    }
}

/// What a walk emits into: inclusive ranges in increasing order.
trait Sink<W: Word> {
    fn emit(&mut self, first: W, last: W);

    /// The runs of ones of `m`, bit `p` the range of `2^span` keys from
    /// `key | p << span`.
    #[inline]
    fn emit_mask(&mut self, key: W, mut m: u64, span: u32) {
        while m != 0 {
            let p = m.trailing_zeros();
            let len = (m >> p).trailing_ones();
            let first = key.or(small::<W>(p).shl(span));
            // `(p + len) << span` is one past the node at the top.
            let last = key
                .or(small::<W>(p + len - 1).shl(span))
                .or(W::low_ones(span));
            self.emit(first, last);
            m &= !(u64::MAX >> (64 - len) << p);
        }
    }
}

/// Records the gaps between the runs of a cover into the output, used as
/// scratch: `2 · out.len()` words, gap `i` in half `i % 2` of pair `i / 2`.
/// The gap is `next.first - prev.last`, one more than the keys between.
struct Gaps<'a, W> {
    out: &'a mut [(W, W)],
    runs: usize,
    last: Option<W>,
    /// Gaps by bit length: `lengths[b]` of them have `b` bits.
    lengths: [u32; 129],
    /// The least bit length among them, where the search starts.
    least: u32,
}

impl<W: Word> Sink<W> for Gaps<'_, W> {
    fn emit(&mut self, first: W, last: W) {
        match self.last {
            Some(end) if end != W::ONES && end.wrapping_add(W::ONE) == first => {}
            Some(end) => {
                let gap = first.wrapping_sub(end);
                set(self.out, self.runs - 1, gap);
                let bits = bitlen(gap);
                self.lengths[bits as usize] += 1;
                self.least = self.least.min(bits);
                self.runs += 1;
            }
            None => self.runs += 1,
        }
        self.last = Some(last);
    }
}

const fn get<W: Copy>(v: &[(W, W)], i: usize) -> W {
    if i.is_multiple_of(2) {
        v[i / 2].0
    } else {
        v[i / 2].1
    }
}

fn set<W>(v: &mut [(W, W)], i: usize, x: W) {
    if i.is_multiple_of(2) {
        v[i / 2].0 = x;
    } else {
        v[i / 2].1 = x;
    }
}

/// The `m`-th smallest (from 0) of the first `len` words of `v`, by
/// quickselect with the middle element as the pivot; reorders them.
#[allow(clippy::many_single_char_names)]
fn select<W: Copy + Ord>(v: &mut [(W, W)], len: usize, m: usize) -> W {
    let (mut lo, mut hi) = (0, len - 1);
    loop {
        if lo == hi {
            return get(v, lo);
        }
        let pivot = get(v, lo + (hi - lo) / 2);
        // Three-way partition: [lo, lt) below, [lt, gt] equal, (gt, hi] above.
        let (mut lt, mut i, mut gt) = (lo, lo, hi);
        while i <= gt {
            let x = get(v, i);
            match x.cmp(&pivot) {
                Ordering::Less => {
                    let y = get(v, lt);
                    set(v, lt, x);
                    set(v, i, y);
                    lt += 1;
                    i += 1;
                }
                Ordering::Greater => {
                    let y = get(v, gt);
                    set(v, gt, x);
                    set(v, i, y);
                    // `gt` stays at or above the pivot's copy, never 0 here.
                    gt -= 1;
                }
                Ordering::Equal => i += 1,
            }
        }
        if m < lt {
            hi = lt - 1;
        } else if m > gt {
            lo = gt + 1;
        } else {
            return pivot;
        }
    }
}

/// The `close`-th smallest of the first `gaps` words of `v` (from 1),
/// and how many equal to it are among the `close` smallest.
///
/// The classes of an offset allocator: the bit length, counted as the
/// gaps were written, names the bucket and how many lie below; the
/// three bits under the leading one split the bucket in eight, counted
/// here with the least and greatest of each. Gaps are sums of a few node
/// sizes, and on random rectangles the class of the threshold held one
/// value every time it was measured, which is then the answer. Only a
/// class of several values is gathered for a select.
// The bucket is a bit length, at most 128.
#[allow(clippy::many_single_char_names, clippy::cast_possible_truncation)]
fn threshold<W: Word + Ord>(
    v: &mut [(W, W)],
    gaps: usize,
    (lengths, least): (&[u32; 129], u32),
    close: usize,
) -> (W, usize) {
    let (mut below, mut b) = (0, least as usize);
    while below + (lengths[b] as usize) < close {
        below += lengths[b] as usize;
        b += 1;
    }
    let mut need = close - below;
    let bits = b as u32;
    // Below 16 the value is its own class.
    let class = |g: W| usize::from(if bits > 4 { g.shr(bits - 4) } else { g }.low_byte() & 7);
    let (mut count, mut lo, mut hi) = ([0usize; 8], [W::ONES; 8], [W::ZERO; 8]);
    for i in 0..gaps {
        let g = get(v, i);
        if bitlen(g) == bits {
            let c = class(g);
            count[c] += 1;
            lo[c] = lo[c].min(g);
            hi[c] = hi[c].max(g);
        }
    }
    let mut c = 0;
    while count[c] < need {
        need -= count[c];
        c += 1;
    }
    if lo[c] == hi[c] {
        return (lo[c], need);
    }
    let mut k = 0;
    for i in 0..gaps {
        let g = get(v, i);
        if bitlen(g) == bits && class(g) == c {
            set(v, i, get(v, k));
            set(v, k, g);
            k += 1;
        }
    }
    let t = select(v, k, need - 1);
    let less = (0..k).filter(|&i| get(v, i) < t).count();
    (t, need - less)
}

/// Writes the runs into `out`, closing every gap below `threshold` and
/// the first `ties` gaps equal to it.
struct Merge<'a, W> {
    out: &'a mut [(W, W)],
    n: usize,
    threshold: W,
    ties: usize,
}

impl<W: Word + Ord> Sink<W> for Merge<'_, W> {
    fn emit(&mut self, first: W, last: W) {
        if self.n > 0 {
            let end = self.out[self.n - 1].1;
            let touch = end != W::ONES && end.wrapping_add(W::ONE) == first;
            let gap = first.wrapping_sub(end);
            let close = touch
                || gap < self.threshold
                || (gap == self.threshold && self.ties > 0 && {
                    self.ties -= 1;
                    true
                });
            if close {
                self.out[self.n - 1].1 = last;
                return;
            }
        }
        self.out[self.n] = (first, last);
        self.n += 1;
    }
}

/// A small constant in `W`: `v < 256`.
#[allow(clippy::cast_possible_truncation)]
#[inline]
pub(crate) fn small<W: Word>(v: u32) -> W {
    debug_assert!(v < 256);
    W::splat_byte(v as u8).and(W::low_ones(8))
}

/// Bits up to the highest set one; 0 for 0.
#[inline]
pub(crate) fn bitlen<W: Word>(v: W) -> u32 {
    W::BITS - v.leading_zeros()
}

/// `usize` in `W`, saturating.
#[allow(clippy::cast_possible_truncation)]
fn from_usize<W: Word>(v: usize) -> W {
    if usize::BITS - v.leading_zeros() > W::BITS {
        return W::ONES;
    }
    let mut w = W::ZERO;
    let mut i = 0;
    while i < usize::BITS && i < W::BITS {
        w = w.or(small::<W>(((v >> i) & 0xFF) as u32).shl(i));
        i += 8;
    }
    w
}

/// The rectangle, inclusive on both axes, already clipped to the grid.
pub(crate) struct Rect<W> {
    pub(crate) x0: W,
    pub(crate) x1: W,
    pub(crate) y0: W,
    pub(crate) y1: W,
}

/// Of the `2^k` columns `2^below` wide of a node at `o` on one axis,
/// those meeting `a..=b` and those inside it, as masks.
#[allow(clippy::many_single_char_names)]
#[inline]
fn axis<W: Word + Ord>(o: W, (a, b): (W, W), below: u32, k: u32) -> (usize, usize) {
    if b < o {
        return (0, 0);
    }
    let n = 1u32 << k;
    // Column numbers past the node do not matter beyond `n`.
    let clamp = |v: W| {
        if v >= small(n) {
            n
        } else {
            u32::from(v.low_byte())
        }
    };
    let span = |lo: u32, hi: u32| {
        if lo > hi {
            0
        } else {
            ((2usize << hi) - 1) & !((1usize << lo) - 1)
        }
    };
    let d = b.wrapping_sub(o);
    // The last column met and one past the last one inside, which ends
    // where `b + 1` starts.
    let met = clamp(d.shr(below)).min(n - 1);
    let past = clamp(d.wrapping_add(W::ONE).shr(below)).min(n);
    let (first_met, first_in) = if a > o {
        let d = a.wrapping_sub(o);
        (
            clamp(d.shr(below)),
            clamp(d.wrapping_sub(W::ONE).shr(below)) + 1,
        )
    } else {
        (0, 0)
    };
    let inside = if past == 0 {
        0
    } else {
        span(first_in, past - 1)
    };
    (span(first_met, met), inside)
}

/// A node at `level` whose first key is `key`, at `(ox, oy)`, in
/// `frame`, `level > stop`: its cover down to `stop`, below which a
/// node across the edge is taken whole. Three levels a step, the odd
/// ones at the top, so every step ends on `stop`.
#[allow(clippy::too_many_arguments)]
fn walk<W: Word + Ord, C: Quadrants, S: Sink<W>>(
    r: &Rect<W>,
    level: u32,
    stop: u32,
    key: W,
    (ox, oy): (W, W),
    frame: u8,
    sink: &mut S,
) {
    let k = match (level - stop) % 3 {
        0 => 3,
        k => k,
    };
    let below = level - k;
    let (cols, cols_in) = axis(ox, (r.x0, r.x1), below, k);
    let (rows, rows_in) = axis(oy, (r.y0, r.y1), below, k);
    let met = C::cells(k, frame, cols, rows);
    let span = 2 * below;
    if below == stop {
        return sink.emit_mask(key, met, span);
    }
    let inside = C::cells(k, frame, cols_in, rows_in);
    let mut rest = met;
    while rest != 0 {
        let p = rest.trailing_zeros();
        if inside >> p & 1 == 1 {
            let run = (inside >> p).trailing_ones();
            let first = key.or(small::<W>(p).shl(span));
            let last = key
                .or(small::<W>(p + run - 1).shl(span))
                .or(W::low_ones(span));
            sink.emit(first, last);
            rest &= !(u64::MAX >> (64 - run) << p);
        } else {
            let (cx, cy, next) = C::child(k, frame, p);
            let child_key = key.or(small::<W>(p).shl(span));
            let child = (
                ox.or(small::<W>(cx.into()).shl(below)),
                oy.or(small::<W>(cy.into()).shl(below)),
            );
            walk::<W, C, S>(r, below, stop, child_key, child, next, sink);
            rest &= rest - 1;
        }
    }
}

/// The least `s ≤ top` whose cover has at most `limit` runs; the cover
/// at `top` is one node. The runs of a cover go as its perimeter over
/// `2^s`, which gives a first guess, then single steps. On random
/// rectangles the guess is right or one off nine times in ten, about two
/// counts a query; long thin ones and aligned ones stray further, a count
/// a level.
fn depth<W: Word + Ord, C: Quadrants>(
    levels: u32,
    r: &Rect<W>,
    top: u32,
    limit: usize,
    context: &C::Context<W>,
) -> u32 {
    let limit = from_usize::<W>(limit);
    let fits = |s: u32| C::runs(levels, r, s, context) <= limit;
    let perimeter =
        r.x1.wrapping_sub(r.x0)
            .wrapping_add(r.y1.wrapping_sub(r.y0));
    let mut s = bitlen(perimeter).saturating_sub(bitlen(limit)).min(top);
    if fits(s) {
        while s > 0 && fits(s - 1) {
            s -= 1;
        }
    } else {
        s += 1;
        while s < top && !fits(s) {
            s += 1;
        }
    }
    s
}

/// The cover of `x0..=x1` × `y0..=y1` on a curve of `levels` levels,
/// top frame 0, in at most `out.len()` ranges; returns how many.
pub(crate) fn cover<W: Word + Ord, C: Quadrants>(
    levels: u32,
    (x0, x1): (W, W),
    (y0, y1): (W, W),
    out: &mut [(W, W)],
) -> usize {
    let side = W::low_ones(levels);
    let (x1, y1) = (x1.min(side), y1.min(side));
    if x0 > x1 || y0 > y1 {
        return 0;
    }
    assert!(
        !out.is_empty(),
        "a nonempty rectangle needs room for a range"
    );
    let r = Rect { x0, x1, y0, y1 };
    // The least aligned node holding the rectangle: above it every cover
    // is that one node, so the walks start there, found by one descent
    // along the path of a corner.
    let top_level = bitlen(x0.xor(x1).or(y0.xor(y1)));
    let (mut key, mut frame) = (W::ZERO, 0);
    let mut level = levels;
    while level > top_level {
        level -= 1;
        let bit = |v: W| u8::from(v.bit(level));
        let (digit, below) = C::digit(frame, bit(x0), bit(y0));
        key = key.or(small::<W>(digit.into()).shl(2 * level));
        frame = below;
    }
    let context = C::context(&r);
    let stop = depth::<W, C>(levels, &r, top_level, out.len().saturating_mul(2), &context);
    if stop == top_level {
        out[0] = (key, key.or(W::low_ones(2 * top_level)));
        return 1;
    }
    let above = W::low_ones(top_level).not();
    let origin = (x0.and(above), y0.and(above));
    // The gaps of that cover, and the threshold that leaves `budget`
    // runs: the largest `budget - 1` gaps stay open.
    let mut gaps = Gaps {
        out: &mut *out,
        runs: 0,
        last: None,
        lengths: [0; 129],
        least: u32::MAX,
    };
    walk::<W, C, _>(&r, top_level, stop, key, origin, frame, &mut gaps);
    let (runs, lengths, least) = (gaps.runs, gaps.lengths, gaps.least);
    let budget = out.len();
    let (threshold, ties) = if runs > budget {
        threshold(out, runs - 1, (&lengths, least), runs - budget)
    } else {
        (W::ZERO, 0)
    };
    let mut merge = Merge {
        out,
        n: 0,
        threshold,
        ties,
    };
    walk::<W, C, _>(&r, top_level, stop, key, origin, frame, &mut merge);
    merge.n
}

/// Bits `lo..=hi` of a `u64`, none when `lo > hi`; both below 64.
#[inline]
const fn bits(lo: u32, hi: u32) -> u64 {
    if lo > hi {
        0
    } else {
        (u64::MAX >> (63 - hi)) & (u64::MAX << lo)
    }
}

/// Whether the node at `level` whose first key is `key`, square at
/// `(ox, oy)`, shares a cell with the rectangle whose key lies in
/// `a..=b`; the node meets both. A node is an interval of keys and a
/// square of cells at once, and so are its descendants three levels
/// down: those the interval meets and those inside it are two runs of
/// bits, the rectangle's two masks as in the cover. A descendant inside
/// either while meeting the other settles it. Only the descendants
/// holding `a` and `b` are partly in the interval, so at most two go
/// down, and a path is a third as long as a level at a time.
#[allow(clippy::many_single_char_names)]
fn meets<W: Word + Ord, C: Quadrants>(
    r: &Rect<W>,
    (a, b): (W, W),
    level: u32,
    key: W,
    (ox, oy): (W, W),
    frame: u8,
) -> bool {
    let k = level.min(3);
    let below = level - k;
    let span = 2 * below;
    let n = 1u32 << (2 * k);
    // Descendant numbers past the node do not matter beyond `n`.
    let clamp = |v: W| {
        if v >= small(n) {
            n
        } else {
            u32::from(v.low_byte())
        }
    };
    let (first_met, first_in) = if a > key {
        let d = a.wrapping_sub(key);
        (
            clamp(d.shr(span)),
            clamp(d.wrapping_sub(W::ONE).shr(span)) + 1,
        )
    } else {
        (0, 0)
    };
    // The node meets `a..=b`, so `b` is at or past its first key.
    let d = b.wrapping_sub(key);
    let last_met = clamp(d.shr(span)).min(n - 1);
    // One past the last descendant inside: where `b + 1` starts, or all
    // of them when `b` is at or past the node's end.
    let past = if d >= W::low_ones(2 * level) {
        n
    } else {
        clamp(d.wrapping_add(W::ONE).shr(span)).min(n)
    };
    let keys_met = bits(first_met, last_met);
    let keys_in = if past == 0 {
        0
    } else {
        bits(first_in, past - 1)
    };
    let (cols, cols_in) = axis(ox, (r.x0, r.x1), below, k);
    let (rows, rows_in) = axis(oy, (r.y0, r.y1), below, k);
    let met = C::cells(k, frame, cols, rows);
    let inside = C::cells(k, frame, cols_in, rows_in);
    if met & keys_in != 0 || inside & keys_met != 0 {
        return true;
    }
    // Partly in both; none once the descendants are cells.
    let mut rest = met & keys_met;
    while rest != 0 {
        let p = rest.trailing_zeros();
        let (cx, cy, next) = C::child(k, frame, p);
        let child_key = key.or(small::<W>(p).shl(span));
        let child = (
            ox.or(small::<W>(cx.into()).shl(below)),
            oy.or(small::<W>(cy.into()).shl(below)),
        );
        if meets::<W, C>(r, (a, b), below, child_key, child, next) {
            return true;
        }
        rest &= rest - 1;
    }
    false
}

/// Whether some cell of `x0..=x1` × `y0..=y1` has its key in `a..=b`.
pub(crate) fn intersects<W: Word + Ord, C: Quadrants>(
    levels: u32,
    (a, b): (W, W),
    (x0, x1): (W, W),
    (y0, y1): (W, W),
) -> bool {
    let side = W::low_ones(levels);
    let (x1, y1) = (x1.min(side), y1.min(side));
    if a > b || x0 > x1 || y0 > y1 {
        return false;
    }
    let r = Rect { x0, x1, y0, y1 };
    meets::<W, C>(&r, (a, b), levels, W::ZERO, (W::ZERO, W::ZERO), 0)
}

#[cfg(test)]
mod tests {
    use super::{Quadrants, Rect, Sink, walk};
    use crate::dilated::ZQuadrants;
    use crate::hilbert::HilbertQuadrants;
    use crate::word::Word;

    /// Counts runs, touching ones merged.
    struct Count<W> {
        n: usize,
        last: Option<W>,
    }

    impl<W: Word> Sink<W> for Count<W> {
        fn emit(&mut self, first: W, last: W) {
            match self.last {
                Some(end) if end != W::ONES && end.wrapping_add(W::ONE) == first => {}
                _ => self.n += 1,
            }
            self.last = Some(last);
        }
    }

    /// The closed form against the walk, at every depth below the top.
    fn agree<W: Word + Ord, C: Quadrants>(r: &Rect<W>) -> bool {
        let levels = W::BITS / 2;
        let top = super::bitlen(r.x0.xor(r.x1).or(r.y0.xor(r.y1)));
        let context = C::context(r);
        (0..top).all(|s| {
            // The walk starts at the root here; the digits above the
            // rectangle's node are the same one key in every range.
            let mut count = Count { n: 0, last: None };
            walk::<W, C, _>(r, levels, s, W::ZERO, (W::ZERO, W::ZERO), 0, &mut count);
            let runs = C::runs(levels, r, s, &context);
            runs == super::from_usize(count.n)
        })
    }

    /// The threshold against sorting, on gaps that share classes with
    /// other values (16 and 17, 32 and 35) so the select behind the
    /// classes runs too.
    #[allow(clippy::cast_possible_truncation)]
    #[test]
    fn threshold_matches_sorting() {
        let mut state = 0x2545_F491_4F6C_DD1Du64;
        let mut next = || {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state
        };
        let pool = [
            2u64,
            3,
            5,
            7,
            15,
            16,
            17,
            18,
            24,
            31,
            32,
            35,
            40,
            64,
            65,
            1000,
            1001,
            1 << 40,
        ];
        for _ in 0..20_000 {
            let n = 1 + (next() % 70) as usize;
            let mut v = [(0u64, 0u64); 35];
            let (mut lengths, mut least) = ([0u32; 129], u32::MAX);
            let mut all = [0u64; 70];
            for (i, slot) in all.iter_mut().enumerate().take(n) {
                let g = if next() % 4 == 0 {
                    2 + next() % 5000
                } else {
                    pool[(next() % pool.len() as u64) as usize]
                };
                *slot = g;
                super::set(&mut v, i, g);
                lengths[super::bitlen(g) as usize] += 1;
                least = least.min(super::bitlen(g));
            }
            let close = 1 + (next() % n as u64) as usize;
            let (t, ties) = super::threshold(&mut v, n, (&lengths, least), close);
            let sorted = &mut all[..n];
            sorted.sort_unstable();
            let want = sorted[close - 1];
            let below = sorted.iter().filter(|&&g| g < want).count();
            assert_eq!((t, ties), (want, close - below), "{sorted:?} close {close}");
        }
    }

    #[test]
    fn runs_every_u8_rectangle() {
        for x0 in 0..16u8 {
            for x1 in x0..16 {
                for y0 in 0..16u8 {
                    for y1 in y0..16 {
                        let r = Rect { x0, x1, y0, y1 };
                        assert!(agree::<u8, ZQuadrants>(&r), "Z {x0}..={x1} × {y0}..={y1}");
                        assert!(
                            agree::<u8, HilbertQuadrants>(&r),
                            "H {x0}..={x1} × {y0}..={y1}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn runs_random_u32_rectangles() {
        let mut state = 0x9E37_79B9_7F4A_7C15u64;
        let mut next = || {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (state >> 32) as u32
        };
        for i in 0..4000 {
            // Sides of every scale, walks of a few thousand nodes at most.
            let side = |v: u32, n: u32| v & (u32::MAX >> (32 - n.max(1)));
            let (x0, y0) = (next() >> 16, next() >> 16);
            let n = i % 11;
            let r = Rect {
                x0,
                x1: (x0 + side(next(), n)).min(0xFFFF),
                y0,
                y1: (y0 + side(next(), n)).min(0xFFFF),
            };
            assert!(
                agree::<u32, ZQuadrants>(&r),
                "Z {}..={} × {}..={}",
                r.x0,
                r.x1,
                r.y0,
                r.y1
            );
            assert!(
                agree::<u32, HilbertQuadrants>(&r),
                "H {}..={} × {}..={}",
                r.x0,
                r.x1,
                r.y0,
                r.y1
            );
        }
    }
}

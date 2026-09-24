//! Rectangles as ranges of keys on a quadrant-recursive curve: the query
//! side of an index built by sorting on `Morton2` or `Hilbert2` codes.
//!
//! A descent of the quadtree in curve order: a node inside the rectangle
//! is one range of keys, a node outside is none, a node across its edge
//! is split. The ranges come out sorted, and ranges that touch are merged
//! as they come.
//!
//! The exact cover of a rectangle has about one range per cell of its
//! boundary, far more than a scan wants to restart, so the caller sets a
//! budget: the length of the output. Two things keep to it. The depth:
//! below some level a node across the edge is taken whole, and counting
//! passes find the deepest level whose cover has at most twice the
//! budget (a coarser cover never has more ranges than a finer one: each
//! run of the finer lies in one run of the coarser). Then the merge: the
//! last pass writes into the output and, when it is full, closes the
//! smallest gap between neighbours, the new range's included. The ranges
//! arrive in order, so what remains are the `budget - 1` largest gaps of
//! that cover, which is the smallest over-cover any `budget` ranges of it
//! can have. The work is a few times the budget per level, not the
//! perimeter of the rectangle.

// `unreachable_pub` wants `pub(crate)` here and clippy wants `pub`; the
// rustc lint is the one the crate chose.
#![allow(clippy::redundant_pub_crate)]

use core::cmp::Ordering;

use crate::word::Word;

/// A quadrant-recursive curve as the descent sees it: the child of a
/// node in `frame` that holds digit `d` (`0..4`, curve order) sits in
/// quadrant `(dx, dy)` and reads its own children in the frame returned.
pub(crate) trait Quadrants {
    fn child(frame: u8, digit: u8) -> (u8, u8, u8);
    /// The other way: the digit of quadrant `(dx, dy)` and the frame
    /// below.
    fn digit(frame: u8, dx: u8, dy: u8) -> (u8, u8);
}

/// What a walk emits into: inclusive ranges in increasing order.
trait Sink<W> {
    /// `false` stops the walk.
    fn emit(&mut self, first: W, last: W) -> bool;
}

/// Counts ranges, touching ones merged, up to `limit`.
struct Count<W> {
    n: usize,
    last: Option<W>,
    limit: usize,
}

impl<W: Word> Sink<W> for Count<W> {
    fn emit(&mut self, first: W, last: W) -> bool {
        match self.last {
            Some(end) if end != W::ONES && end.wrapping_add(W::ONE) == first => {}
            _ => self.n += 1,
        }
        self.last = Some(last);
        self.n <= self.limit
    }
}

/// Records the gaps between the runs of a cover into the output, used as
/// scratch: `2 · out.len()` words, gap `i` in half `i % 2` of pair `i / 2`.
/// The gap is `next.first - prev.last`, one more than the keys between.
struct Gaps<'a, W> {
    out: &'a mut [(W, W)],
    runs: usize,
    last: Option<W>,
}

impl<W: Word> Sink<W> for Gaps<'_, W> {
    fn emit(&mut self, first: W, last: W) -> bool {
        match self.last {
            Some(end) if end != W::ONES && end.wrapping_add(W::ONE) == first => {}
            Some(end) => {
                set(self.out, self.runs - 1, first.wrapping_sub(end));
                self.runs += 1;
            }
            None => self.runs += 1,
        }
        self.last = Some(last);
        true
    }
}

const fn get<W: Copy>(v: &[(W, W)], i: usize) -> W {
    if i.is_multiple_of(2) { v[i / 2].0 } else { v[i / 2].1 }
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

/// Writes the runs into `out`, closing every gap below `threshold` and
/// the first `ties` gaps equal to it.
struct Merge<'a, W> {
    out: &'a mut [(W, W)],
    n: usize,
    threshold: W,
    ties: usize,
}

impl<W: Word + Ord> Sink<W> for Merge<'_, W> {
    fn emit(&mut self, first: W, last: W) -> bool {
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
                return true;
            }
        }
        self.out[self.n] = (first, last);
        self.n += 1;
        true
    }
}

/// A small constant in `W`: the byte splatted, all but the low byte off.
fn small<W: Word>(v: u8) -> W {
    W::splat_byte(v).and(W::low_ones(8))
}

/// The rectangle, inclusive on both axes, already clipped to the grid.
struct Rect<W> {
    x0: W,
    x1: W,
    y0: W,
    y1: W,
}

/// A node `side = 2^level` cells wide at `(ox, oy)` whose first key is
/// `key`, in `frame`. Children at `level - 1` down to `stop`, below which
/// a node across the edge is emitted whole.
#[allow(clippy::too_many_arguments)]
fn walk<W: Word + Ord, C: Quadrants, S: Sink<W>>(
    r: &Rect<W>,
    level: u32,
    stop: u32,
    key: W,
    (ox, oy): (W, W),
    frame: u8,
    sink: &mut S,
) -> bool {
    let reach = W::low_ones(level);
    let (ex, ey) = (ox.wrapping_add(reach), oy.wrapping_add(reach));
    if ox > r.x1 || ex < r.x0 || oy > r.y1 || ey < r.y0 {
        return true;
    }
    let inside = r.x0 <= ox && ex <= r.x1 && r.y0 <= oy && ey <= r.y1;
    if inside || level == stop {
        return sink.emit(key, key.or(W::low_ones(2 * level)));
    }
    let below = level - 1;
    for digit in 0..4u8 {
        let (dx, dy, next) = C::child(frame, digit);
        let child_key = key.or(small::<W>(digit).shl(2 * below));
        let child = (
            ox.or(small::<W>(dx).shl(below)),
            oy.or(small::<W>(dy).shl(below)),
        );
        if !walk::<W, C, S>(r, below, stop, child_key, child, next, sink) {
            return false;
        }
    }
    true
}

/// The cover of `x0..=x1` × `y0..=y1` on a curve of `levels` levels whose
/// top frame is `top`, in at most `out.len()` ranges; returns how many.
pub(crate) fn cover<W: Word + Ord, C: Quadrants>(
    levels: u32,
    top: u8,
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
    let top_level = W::BITS - x0.xor(x1).or(y0.xor(y1)).leading_zeros();
    let (mut key, mut frame) = (W::ZERO, top);
    let mut level = levels;
    while level > top_level {
        level -= 1;
        let bit = |v: W| u8::from(!v.shr(level).and(W::ONE).is_zero());
        let (digit, below) = C::digit(frame, bit(x0), bit(y0));
        key = key.or(small::<W>(digit).shl(2 * level));
        frame = below;
    }
    let above = W::low_ones(top_level).not();
    let origin = (x0.and(above), y0.and(above));
    // The deepest stop whose cover fits two budgets, coarse to fine: its
    // gaps then fit the output as scratch.
    let limit = out.len().saturating_mul(2);
    let mut stop = top_level;
    while stop > 0 {
        let mut count = Count {
            n: 0,
            last: None,
            limit,
        };
        if !walk::<W, C, _>(&r, top_level, stop - 1, key, origin, frame, &mut count) {
            break;
        }
        stop -= 1;
    }
    // The gaps of that cover, and the threshold that leaves `budget`
    // runs: the largest `budget - 1` gaps stay open.
    let mut gaps = Gaps { out: &mut *out, runs: 0, last: None };
    walk::<W, C, _>(&r, top_level, stop, key, origin, frame, &mut gaps);
    let runs = gaps.runs;
    let budget = out.len();
    let (threshold, ties) = if runs > budget {
        let close = runs - budget;
        let t = select(out, runs - 1, close - 1);
        let below = (0..runs - 1).filter(|&i| get(out, i) < t).count();
        (t, close - below)
    } else {
        (W::ZERO, 0)
    };
    let mut merge = Merge { out, n: 0, threshold, ties };
    walk::<W, C, _>(&r, top_level, stop, key, origin, frame, &mut merge);
    merge.n
}

/// Whether the node at `level` whose first key is `key`, square at
/// `(ox, oy)`, shares a cell with the rectangle whose key lies in
/// `a..=b`. A node is an interval of keys and a square of cells at once:
/// disjoint from either, no; inside either while meeting the other, yes;
/// else its children. Only the nodes on the paths of `a` and `b` are
/// partly inside `a..=b`, so the walk is two paths down, not a tree.
fn meets<W: Word + Ord, C: Quadrants>(
    r: &Rect<W>,
    (a, b): (W, W),
    level: u32,
    key: W,
    (ox, oy): (W, W),
    frame: u8,
) -> bool {
    let last = key.or(W::low_ones(2 * level));
    if last < a || key > b {
        return false;
    }
    let reach = W::low_ones(level);
    let (ex, ey) = (ox.wrapping_add(reach), oy.wrapping_add(reach));
    if ox > r.x1 || ex < r.x0 || oy > r.y1 || ey < r.y0 {
        return false;
    }
    // A cell of the node meets both; a single cell is inside both here.
    if (a <= key && last <= b) || (r.x0 <= ox && ex <= r.x1 && r.y0 <= oy && ey <= r.y1) {
        return true;
    }
    let below = level - 1;
    (0..4u8).any(|digit| {
        let (dx, dy, next) = C::child(frame, digit);
        let child_key = key.or(small::<W>(digit).shl(2 * below));
        let child = (
            ox.or(small::<W>(dx).shl(below)),
            oy.or(small::<W>(dy).shl(below)),
        );
        meets::<W, C>(r, (a, b), below, child_key, child, next)
    })
}

/// Whether some cell of `x0..=x1` × `y0..=y1` has its key in `a..=b`.
pub(crate) fn intersects<W: Word + Ord, C: Quadrants>(
    levels: u32,
    top: u8,
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
    meets::<W, C>(&r, (a, b), levels, W::ZERO, (W::ZERO, W::ZERO), top)
}

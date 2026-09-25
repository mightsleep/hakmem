//! Two-dimensional runs: rectangular blocks in a bitmap of rows.
//!
//! One word per row, bit `c` of row `r` is cell `(r, c)`. Finding a
//! `w × h` block of set cells is separable: the halving chain of
//! [`Bits::run_starts`] applied *down the rows* (rows `r..r + h` all
//! set, `⌈log₂ h⌉` passes) and then *along each row* (`⌈log₂ w⌉`
//! steps). In morphology terms this is binary erosion by a `w × h`
//! rectangle; in allocator terms it is first-fit of a rectangle over
//! a tile bitmap, texture atlases, sparse-texture page tables,
//! inventory grids, occupancy grids.
//!
//! Composition carries over from one dimension:
//! `block_starts(block_starts(g, w₁, h₁), w₂, h₂) =
//!  block_starts(g, w₁ + w₂ − 1, h₁ + h₂ − 1)`.
//!
//! Grids narrower than `BITS` columns must keep unused columns clear.

use crate::bits::Bits;
use crate::word::Word;

/// Writes into `out[r]` the columns `c` at which a `w × h` block of set
/// cells starts at `(r, c)`. `out.len() == rows.len()`; rows that
/// cannot fit `h` more rows come out zero. `w` in `1..=BITS`, `h >= 1`.
///
/// ```
/// use hakmem::grid::block_starts;
///
/// let rows = [0b0111_1000u8, 0b0111_1100, 0b0011_1100, 0b0000_0000];
/// let mut out = [0u8; 4];
/// block_starts(&rows, 2, 2, &mut out);
/// // 2×2 blocks start at (0, 3..=5) and (1, 2..=4).
/// assert_eq!(out, [0b0011_1000, 0b0001_1100, 0, 0]);
/// ```
///
/// # Panics
///
/// If `out` and `rows` differ in length.
#[inline]
pub fn block_starts<W: Word>(rows: &[W], w: u32, h: u32, out: &mut [W]) {
    assert_eq!(rows.len(), out.len(), "block_starts: out must match rows");
    debug_assert!(
        (1..=W::BITS).contains(&w) && h >= 1,
        "block {w}×{h} out of range"
    );
    out.copy_from_slice(rows);
    let len = rows.len();

    // Vertical halving chain: after this, out[r] = AND of rows r..r+h.
    let mut n = h as usize;
    while n > 1 {
        let s = n >> 1;
        for r in 0..len.saturating_sub(s) {
            out[r] = out[r].and(out[r + s]);
        }
        for cell in out.iter_mut().skip(len.saturating_sub(s)) {
            *cell = W::ZERO;
        }
        n -= s;
    }
    if h as usize > len {
        for cell in out.iter_mut() {
            *cell = W::ZERO;
        }
    }

    // Horizontal chain per row.
    for cell in out.iter_mut() {
        *cell = cell.run_starts(w);
    }
}

/// First-fit (row-major) position `(row, col)` of a `w × h` block of
/// set cells, using `scratch` (same length as `rows`) as workspace.
///
/// ```
/// use hakmem::grid::find_block;
///
/// let rows = [0b0111_1000u8, 0b0111_1100, 0b0011_1100, 0];
/// let mut scratch = [0u8; 4];
/// assert_eq!(find_block(&rows, 2, 2, &mut scratch), Some((0, 3)));
/// assert_eq!(find_block(&rows, 3, 3, &mut scratch), Some((0, 3)));
/// assert_eq!(find_block(&rows, 4, 3, &mut scratch), None);
/// ```
///
/// # Panics
///
/// If `scratch` and `rows` differ in length.
#[must_use]
#[inline]
pub fn find_block<W: Word>(rows: &[W], w: u32, h: u32, scratch: &mut [W]) -> Option<(usize, u32)> {
    block_starts(rows, w, h, scratch);
    scratch
        .iter()
        .enumerate()
        .find_map(|(r, &row)| row.first_set().map(|c| (r, c)))
}

/// Sets every cell of the `w × h` block at `(row, col)`. The block must
/// lie inside the grid (debug-asserted).
///
/// ```
/// use hakmem::grid::{clear_block, fill_block};
///
/// let mut rows = [0u8; 4];
/// fill_block(&mut rows, 1, 2, 3, 2);
/// assert_eq!(rows, [0, 0b1_1100, 0b1_1100, 0]);
/// clear_block(&mut rows, 2, 3, 1, 1);
/// assert_eq!(rows, [0, 0b1_1100, 0b1_0100, 0]);
/// ```
///
/// # Panics
///
/// If the block reaches past the last row.
#[inline]
pub fn fill_block<W: Word>(rows: &mut [W], row: usize, col: u32, w: u32, h: u32) {
    let mask = block_mask::<W>(rows.len(), row, col, w, h);
    for cell in &mut rows[row..row + h as usize] {
        *cell = cell.or(mask);
    }
}

/// Clears every cell of the `w × h` block at `(row, col)`, as
/// [`fill_block`] sets them.
///
/// # Panics
///
/// If the block reaches past the last row.
#[inline]
pub fn clear_block<W: Word>(rows: &mut [W], row: usize, col: u32, w: u32, h: u32) {
    let mask = block_mask::<W>(rows.len(), row, col, w, h);
    for cell in &mut rows[row..row + h as usize] {
        *cell = cell.and(mask.not());
    }
}

/// The block's columns in one row. It used to be one function with a
/// `bool` for "fill", which read at the call site as `true`.
fn block_mask<W: Word>(len: usize, row: usize, col: u32, w: u32, h: u32) -> W {
    debug_assert!(
        col + w <= W::BITS && row + h as usize <= len,
        "block outside grid"
    );
    W::low_ones(w).shl(col)
}

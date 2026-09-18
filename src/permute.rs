//! Permutations built from delta swaps.
//!
//! A delta swap exchanges two sets of bits a fixed distance apart in
//! three operations (`t = (x ^ (x >> s)) & m; x ^ t ^ (t << s)`). Every
//! bit permutation is a composition of at most `2·log₂ BITS − 1` of them
//! (a Beneš network); the ones that matter in practice (8×8 board
//! transposes, flips and rotations for bitboards and bit matrices)
//! are three each and are provided here for `u64`.

/// An 8×8 bit matrix in a `u64`: bit `8·row + col`. For chess
/// bitboards this is the usual little-endian rank-file mapping
/// (`a1 = 0`, `h8 = 63`, rows = ranks, cols = files).
pub mod board8 {
    use crate::bits::Bits;

    /// Transpose about the main diagonal: bit `(r, c)` moves to `(c, r)`.
    /// Three delta swaps (Hacker's Delight 7-3).
    ///
    /// ```
    /// use hakmem::permute::board8::transpose;
    ///
    /// // A single bit at row 1, col 5 → row 5, col 1.
    /// assert_eq!(transpose(1 << (8 * 1 + 5)), 1 << (8 * 5 + 1));
    /// ```
    #[inline]
    #[must_use]
    pub fn transpose(x: u64) -> u64 {
        let x = x.delta_swap(0x00AA_00AA_00AA_00AA, 7);
        let x = x.delta_swap(0x0000_CCCC_0000_CCCC, 14);
        x.delta_swap(0x0000_0000_F0F0_F0F0, 28)
    }

    /// Reverse the row order: `(r, c)` → `(7 − r, c)`. One byte swap.
    #[inline]
    #[must_use]
    pub const fn flip_vertical(x: u64) -> u64 {
        x.swap_bytes()
    }

    /// Reverse the column order: `(r, c)` → `(r, 7 − c)`. Three delta
    /// swaps reversing the bits of every byte.
    #[inline]
    #[must_use]
    pub fn mirror_horizontal(x: u64) -> u64 {
        let x = x.delta_swap(0x5555_5555_5555_5555, 1);
        let x = x.delta_swap(0x3333_3333_3333_3333, 2);
        x.delta_swap(0x0F0F_0F0F_0F0F_0F0F, 4)
    }

    /// Rotate 90° clockwise (rows upward, columns rightward):
    /// `(r, c)` → `(7 − c, r)`.
    #[inline]
    #[must_use]
    pub fn rotate_90_cw(x: u64) -> u64 {
        flip_vertical(transpose(x))
    }

    /// Rotate 90° counter-clockwise: `(r, c)` → `(c, 7 − r)`.
    #[inline]
    #[must_use]
    pub fn rotate_90_ccw(x: u64) -> u64 {
        transpose(flip_vertical(x))
    }

    /// Rotate 180°: `(r, c)` → `(7 − r, 7 − c)`. Bit reversal.
    #[inline]
    #[must_use]
    pub const fn rotate_180(x: u64) -> u64 {
        x.reverse_bits()
    }
}

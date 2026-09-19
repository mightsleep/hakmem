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

    const FILE_A: u64 = 0x0101_0101_0101_0101;
    const FILE_H: u64 = FILE_A << 7;

    /// A sliding direction on the board.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum Dir {
        /// Up a file: `+8`.
        North,
        /// Down a file: `-8`.
        South,
        /// Along a rank towards h: `+1`.
        East,
        /// Along a rank towards a: `-1`.
        West,
        /// `+9`.
        NorthEast,
        /// `+7`.
        NorthWest,
        /// `-7`.
        SouthEast,
        /// `-9`.
        SouthWest,
    }

    impl Dir {
        /// The eight directions, the queen's set.
        pub const ALL: [Self; 8] = [
            Self::North,
            Self::South,
            Self::East,
            Self::West,
            Self::NorthEast,
            Self::NorthWest,
            Self::SouthEast,
            Self::SouthWest,
        ];

        /// Stride, whether it shifts up, and the squares a step may land
        /// on (everything but the file a step off the board wraps into).
        const fn step(self) -> (u32, bool, u64) {
            match self {
                Self::North => (8, true, u64::MAX),
                Self::South => (8, false, u64::MAX),
                Self::East => (1, true, !FILE_A),
                Self::West => (1, false, !FILE_H),
                Self::NorthEast => (9, true, !FILE_A),
                Self::NorthWest => (7, true, !FILE_H),
                Self::SouthEast => (7, false, !FILE_A),
                Self::SouthWest => (9, false, !FILE_H),
            }
        }

        /// One step of every bit of `x` in this direction; bits that would
        /// leave the board vanish.
        #[inline]
        #[must_use]
        pub const fn shift(self, x: u64) -> u64 {
            let (s, up, keep) = self.step();
            (if up { x << s } else { x >> s }) & keep
        }
    }

    /// Squares attacked by the sliding pieces `pieces` in direction `dir`.
    ///
    /// With `empty` the empty squares: every square reached up to and
    /// including the first occupied one. Kogge–Stone through
    /// [`Bits::fill_up`] / [`Bits::fill_down`] with the wrap file masked
    /// out of the propagation, then one more step. Eight calls make a
    /// queen; no table.
    ///
    /// ```
    /// use hakmem::permute::board8::{Dir, slide};
    ///
    /// // A rook on a1, a blocker on a5 and one on e1.
    /// let rook = 1u64;
    /// let occupied = 1u64 << 32 | 1 << 4;
    /// assert_eq!(slide(rook, !occupied, Dir::North), 0x0000_0001_0101_0100);
    /// assert_eq!(slide(rook, !occupied, Dir::East), 0b1_1110);
    /// assert_eq!(slide(rook, !occupied, Dir::West), 0);
    /// ```
    #[inline]
    #[must_use]
    pub fn slide(pieces: u64, empty: u64, dir: Dir) -> u64 {
        let (s, up, keep) = dir.step();
        let propagate = empty & keep;
        let filled = if up {
            pieces.fill_up(propagate, s)
        } else {
            pieces.fill_down(propagate, s)
        };
        dir.shift(filled)
    }
}

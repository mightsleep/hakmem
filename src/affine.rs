//! Affine maps on the bits of a byte: `x ↦ A·x ⊕ b` over GF(2).
//!
//! Eight rows of eight bits are a matrix `A`; with a constant `b` the
//! pair names every function of a byte that is XOR-linear in the bits
//! plus a constant: shifts, rotates, bit reversal, parity, NOT, and any
//! composition of those. Two things make the name worth having. GFNI's
//! `gf2p8affineqb` applies one such map to every byte of a register in
//! one instruction, which is where the matrices below come from
//! (Wunkolo's posts on bit reversal and on 8-bit shifts, 2020; SSE has
//! no byte shift, the matrix is one). And composition is matrix
//! multiplication, [`Affine8::then`], so a chain of byte tricks folds
//! into one map at compile time and one instruction at run time.
//!
//! [`Lanes::affine`](crate::lanes::Lanes::affine) applies a map to every
//! lane. Without GFNI, linearity splits it into two nibble lookups
//! (`A·x = A·hi ⊕ A·lo`, so two `lut16` and an XOR); the SWAR carrier
//! folds eight parities instead ([`Affine8::apply8`]).
//!
//! The layout is the instruction's: byte `7 - i` of the matrix is the
//! row that builds output bit `i`, so byte 0 builds bit 7 and the
//! identity is `0x0102_0408_1020_4080`. [`Affine8::from_rows`] and
//! [`Affine8::row`] take rows in bit order if you would rather not
//! think about it.
//!
//! ```
//! use hakmem::affine::Affine8;
//!
//! // Wunkolo's constants fall out of the identity.
//! assert_eq!(Affine8::IDENTITY.matrix(), 0x0102_0408_1020_4080);
//! assert_eq!(Affine8::shl(1).matrix(), 0x0001_0204_0810_2040);
//! assert_eq!(Affine8::shr(2).matrix(), 0x0408_1020_4080_0000);
//! assert_eq!(Affine8::sra(2).matrix(), 0x0408_1020_4080_8080);
//! assert_eq!(Affine8::REVERSE.matrix(), 0x8040_2010_0804_0201);
//!
//! // Reversing then shifting left is shifting right then reversing.
//! let a = Affine8::REVERSE.then(Affine8::shl(3));
//! let b = Affine8::shr(3).then(Affine8::REVERSE);
//! assert_eq!(a, b);
//! for x in 0..=255u8 {
//!     assert_eq!(a.apply(x), x.reverse_bits() << 3);
//! }
//! ```

/// An affine map on the bits of a byte: `x ↦ A·x ⊕ b` over GF(2).
///
/// Every constructor is `const`; a map built from constants is a
/// constant, and so are the tables [`tables`](Self::tables) derives
/// from it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Affine8 {
    matrix: u64,
    add: u8,
}

const IDENTITY_MATRIX: u64 = 0x0102_0408_1020_4080;
const ONES_STEP_8: u64 = 0x0101_0101_0101_0101;
const MSBS_STEP_8: u64 = 0x8080_8080_8080_8080;

impl Affine8 {
    /// `x ↦ x`.
    pub const IDENTITY: Self = Self::new(IDENTITY_MATRIX, 0);
    /// `x ↦ !x`: the identity with every constant bit set.
    pub const NOT: Self = Self::new(IDENTITY_MATRIX, 0xFF);
    /// `x ↦ x.reverse_bits()`: the identity with its rows reversed.
    pub const REVERSE: Self = Self::new(0x8040_2010_0804_0201, 0);
    /// Every output bit is the parity of the input: `0xFF` for a byte
    /// with an odd number of set bits, `0x00` otherwise. A lane mask.
    pub const PARITY: Self = Self::new(u64::MAX, 0);
    /// `x ↦ 0`.
    pub const ZERO: Self = Self::new(0, 0);

    /// A map from its matrix in `gf2p8affineqb` layout (byte `7 - i`
    /// builds bit `i`) and its constant.
    #[inline]
    #[must_use]
    pub const fn new(matrix: u64, add: u8) -> Self {
        Self { matrix, add }
    }

    /// A map from its rows in bit order: `rows[i]` masks the input and
    /// its parity is output bit `i`, XOR bit `i` of `add`.
    #[inline]
    #[must_use]
    pub const fn from_rows(rows: [u8; 8], add: u8) -> Self {
        let mut bytes = [0u8; 8];
        let mut i = 0;
        while i < 8 {
            bytes[7 - i] = rows[i];
            i += 1;
        }
        Self::new(u64::from_le_bytes(bytes), add)
    }

    /// The matrix, in the instruction's layout.
    #[inline]
    #[must_use]
    pub const fn matrix(self) -> u64 {
        self.matrix
    }

    /// The constant term.
    #[inline]
    #[must_use]
    pub const fn add(self) -> u8 {
        self.add
    }

    /// The row that builds output bit `bit < 8`.
    #[inline]
    #[must_use]
    pub const fn row(self, bit: usize) -> u8 {
        self.matrix.to_le_bytes()[7 - bit]
    }

    /// `true` when the constant is zero: the map is linear, and
    /// `gf2p8affineqb` needs no XOR after it.
    #[inline]
    #[must_use]
    pub const fn is_linear(self) -> bool {
        self.add == 0
    }

    /// `x ↦ x << n`; `n >= 8` is [`ZERO`](Self::ZERO). The identity
    /// matrix shifted right by `8 n` bits.
    #[inline]
    #[must_use]
    pub const fn shl(n: u32) -> Self {
        if n >= 8 {
            Self::ZERO
        } else {
            Self::new(IDENTITY_MATRIX >> (8 * n), 0)
        }
    }

    /// `x ↦ x >> n`, logical; `n >= 8` is [`ZERO`](Self::ZERO). The
    /// identity matrix shifted left by `8 n` bits.
    #[inline]
    #[must_use]
    pub const fn shr(n: u32) -> Self {
        if n >= 8 {
            Self::ZERO
        } else {
            Self::new(IDENTITY_MATRIX << (8 * n), 0)
        }
    }

    /// `x ↦ (x as i8) >> n`, arithmetic: [`shr`](Self::shr) with the
    /// rows of the vacated bits set to `0x80`, the sign. `n >= 8` fills
    /// every bit with the sign.
    #[inline]
    #[must_use]
    pub const fn sra(n: u32) -> Self {
        if n >= 8 {
            return Self::new(MSBS_STEP_8, 0);
        }
        let vacated = !(u64::MAX << (8 * n)) & MSBS_STEP_8;
        Self::new((IDENTITY_MATRIX << (8 * n)) | vacated, 0)
    }

    /// `x ↦ x.rotate_left(n mod 8)`: the rows of [`shl`](Self::shl)
    /// and of [`shr`](Self::shr) by the complement, disjoint, combined.
    #[inline]
    #[must_use]
    pub const fn rotl(n: u32) -> Self {
        let n = n % 8;
        if n == 0 {
            Self::IDENTITY
        } else {
            Self::new(Self::shl(n).matrix | Self::shr(8 - n).matrix, 0)
        }
    }

    /// `x ↦ x.rotate_right(n mod 8)`.
    #[inline]
    #[must_use]
    pub const fn rotr(n: u32) -> Self {
        Self::rotl((8 - n % 8) % 8)
    }

    /// The map applied to one byte: eight ANDs, eight parities.
    #[inline]
    #[must_use]
    pub const fn apply(self, x: u8) -> u8 {
        let mut out = 0u8;
        let mut i = 0;
        while i < 8 {
            if (self.row(i) & x).count_ones() & 1 == 1 {
                out |= 1 << i;
            }
            i += 1;
        }
        out ^ self.add
    }

    /// The map applied to every byte of a word at once: for each output
    /// bit, AND the word with the row splatted, fold the parity of each
    /// byte into its bit 0 with three XOR-shifts, and place it. Eight
    /// rounds of seven operations, no table; the SWAR carrier's path.
    #[inline]
    #[must_use]
    pub const fn apply8(self, bytes: u64) -> u64 {
        let mut out = 0u64;
        let mut i = 0;
        while i < 8 {
            let mut t = bytes & (self.row(i) as u64 * ONES_STEP_8);
            // Bits from the byte above leak into bits 4..8, 2..8, 1..8;
            // bit 0 only ever sees its own byte.
            t ^= t >> 4;
            t ^= t >> 2;
            t ^= t >> 1;
            out |= (t & ONES_STEP_8) << i;
            i += 1;
        }
        out ^ (self.add as u64 * ONES_STEP_8)
    }

    /// The nibble tables that compute the map by linearity: with
    /// `(lo, hi) = m.tables()`, `m.apply(x) == lo[x & 15] ^ hi[x >> 4]`.
    /// The constant lives in `lo`. This is what
    /// [`Lanes::affine`](crate::lanes::Lanes::affine) runs through
    /// two `lut16` where there is no GFNI.
    #[inline]
    #[must_use]
    pub const fn tables(self) -> ([u8; 16], [u8; 16]) {
        let mut lo = [0u8; 16];
        let mut hi = [0u8; 16];
        let mut k: u8 = 0;
        while k < 16 {
            lo[k as usize] = self.apply(k);
            hi[k as usize] = self.apply(k << 4) ^ self.add;
            k += 1;
        }
        (lo, hi)
    }

    /// `self` first, then `next`: the composite `x ↦ next(self(x))`.
    /// Its matrix is the product `A_next · A_self`, row `i` being the
    /// XOR of the rows of `self` selected by row `i` of `next`; its
    /// constant is `next` applied to `self`'s.
    #[inline]
    #[must_use]
    pub const fn then(self, next: Self) -> Self {
        let mut rows = [0u8; 8];
        let mut i = 0;
        while i < 8 {
            let selector = next.row(i);
            let mut acc = 0u8;
            let mut j = 0;
            while j < 8 {
                if selector >> j & 1 == 1 {
                    acc ^= self.row(j);
                }
                j += 1;
            }
            rows[i] = acc;
            i += 1;
        }
        Self::from_rows(rows, next.apply(self.add))
    }

    /// `self ∘ first`: `first` is applied first. The same map as
    /// `first.then(self)`, in the order mathematicians write it.
    #[inline]
    #[must_use]
    pub const fn compose(self, first: Self) -> Self {
        first.then(self)
    }
}

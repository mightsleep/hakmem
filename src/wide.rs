//! A multi-limb carrier: `N × 64` bits that behave as one word.
//!
//! [`Wide<N>`] implements [`Word`] over `[u64; N]`, so every
//! combinator and every law in the crate runs on it unchanged. The
//! carry chain (`wrapping_add`) propagates across limbs, shifts cross
//! limb boundaries, `count_ones` sums. One operation costs `N` limb
//! operations instead of one instruction, which is still bit-parallel.
//!
//! This is what lifts the "pattern ≤ 128 bytes" limit of
//! [`crate::myers`]: `edit_distance::<Wide<8>>` handles 512-byte
//! patterns with no change to the algorithm, because the algorithm was
//! written against the carrier, not against `u64`. It is also the
//! smallest instance of the carrier-over-carrier idea from the design
//! doc: a "word" need not be one register.

use crate::word::{Word, compress_broadword, expand_broadword, sealed};

/// `N` little-endian limbs of 64 bits: limb 0 holds bits `0..64`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Wide<const N: usize>([u64; N]);

impl<const N: usize> Wide<N> {
    /// Wraps limbs, least significant first.
    #[inline]
    #[must_use]
    pub const fn from_limbs(limbs: [u64; N]) -> Self {
        Self(limbs)
    }

    /// The limbs, least significant first.
    #[inline]
    #[must_use]
    pub const fn limbs(self) -> [u64; N] {
        self.0
    }

    #[inline]
    fn map(self, f: impl Fn(u64) -> u64) -> Self {
        let mut out = self.0;
        for limb in &mut out {
            *limb = f(*limb);
        }
        Self(out)
    }

    #[inline]
    fn zip(self, other: Self, f: impl Fn(u64, u64) -> u64) -> Self {
        let mut out = self.0;
        for (o, b) in out.iter_mut().zip(other.0) {
            *o = f(*o, b);
        }
        Self(out)
    }
}

impl<const N: usize> sealed::Sealed for Wide<N> {}

impl<const N: usize> Word for Wide<N> {
    // `N` limbs of 64 bits fit a u32 for any N a caller can afford.
    #[allow(clippy::cast_possible_truncation)]
    const BITS: u32 = 64 * N as u32;
    const ZERO: Self = Self([0; N]);
    const ONE: Self = {
        let mut limbs = [0; N];
        limbs[0] = 1;
        Self(limbs)
    };
    const ONES: Self = Self([u64::MAX; N]);

    #[inline]
    fn and(self, other: Self) -> Self {
        self.zip(other, |a, b| a & b)
    }
    #[inline]
    fn or(self, other: Self) -> Self {
        self.zip(other, |a, b| a | b)
    }
    #[inline]
    fn xor(self, other: Self) -> Self {
        self.zip(other, |a, b| a ^ b)
    }
    #[inline]
    fn not(self) -> Self {
        self.map(|a| !a)
    }

    #[inline]
    fn shl(self, n: u32) -> Self {
        debug_assert!(n < Self::BITS, "shift {n} >= width {}", Self::BITS);
        let (q, r) = ((n / 64) as usize, n % 64);
        let mut out = [0; N];
        for i in q..N {
            let lo = self.0[i - q] << r;
            let hi = if r != 0 && i > q {
                self.0[i - q - 1] >> (64 - r)
            } else {
                0
            };
            out[i] = lo | hi;
        }
        Self(out)
    }

    #[inline]
    fn shr(self, n: u32) -> Self {
        debug_assert!(n < Self::BITS, "shift {n} >= width {}", Self::BITS);
        let (q, r) = ((n / 64) as usize, n % 64);
        let mut out = [0; N];
        for i in 0..N - q {
            let lo = self.0[i + q] >> r;
            let hi = if r != 0 && i + q + 1 < N {
                self.0[i + q + 1] << (64 - r)
            } else {
                0
            };
            out[i] = lo | hi;
        }
        Self(out)
    }

    #[inline]
    fn wrapping_add(self, other: Self) -> Self {
        let mut out = [0; N];
        let mut carry = false;
        for i in 0..N {
            let (s, c1) = self.0[i].overflowing_add(other.0[i]);
            let (s, c2) = s.overflowing_add(u64::from(carry));
            out[i] = s;
            carry = c1 || c2;
        }
        Self(out)
    }

    #[inline]
    fn wrapping_sub(self, other: Self) -> Self {
        let mut out = [0; N];
        let mut borrow = false;
        for i in 0..N {
            let (s, b1) = self.0[i].overflowing_sub(other.0[i]);
            let (s, b2) = s.overflowing_sub(u64::from(borrow));
            out[i] = s;
            borrow = b1 || b2;
        }
        Self(out)
    }

    /// Schoolbook multiplication modulo `2^BITS`.
    // Limb products are u128; the low half is the limb, the high half the carry.
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn wrapping_mul(self, other: Self) -> Self {
        let mut out = [0u64; N];
        for i in 0..N {
            let mut carry = 0u128;
            for j in 0..N - i {
                let acc =
                    u128::from(out[i + j]) + u128::from(self.0[i]) * u128::from(other.0[j]) + carry;
                out[i + j] = acc as u64;
                carry = acc >> 64;
            }
        }
        Self(out)
    }

    #[inline]
    fn count_ones(self) -> u32 {
        self.0.iter().map(|l| l.count_ones()).sum()
    }

    #[inline]
    fn trailing_zeros(self) -> u32 {
        let mut n = 0;
        for limb in self.0 {
            if limb != 0 {
                return n + limb.trailing_zeros();
            }
            n += 64;
        }
        n
    }

    #[inline]
    fn leading_zeros(self) -> u32 {
        let mut n = 0;
        for limb in self.0.iter().rev() {
            if *limb != 0 {
                return n + limb.leading_zeros();
            }
            n += 64;
        }
        n
    }

    #[inline]
    fn clear_lowest_set(self) -> Self {
        let mut out = self.0;
        for limb in &mut out {
            if *limb != 0 {
                *limb &= *limb - 1;
                break;
            }
        }
        Self(out)
    }

    #[inline]
    fn pext(self, mask: Self) -> Self {
        compress_broadword(self, mask)
    }

    #[inline]
    fn pdep(self, mask: Self) -> Self {
        expand_broadword(self, mask)
    }

    #[inline]
    fn select_lowest(self, mut k: u32) -> u32 {
        let mut base = 0;
        for limb in self.0 {
            let n = limb.count_ones();
            if k < n {
                return base + limb.select_lowest(k);
            }
            k -= n;
            base += 64;
        }
        base
    }

    /// Per-limb prefix XOR, with the parity of all lower limbs folded
    /// into every bit of the limb above.
    #[inline]
    fn xor_scan(self) -> Self {
        let mut out = [0; N];
        let mut parity = 0u64;
        for (o, limb) in out.iter_mut().zip(self.0) {
            *o = limb.xor_scan() ^ parity;
            parity ^= 0u64.wrapping_sub(u64::from(limb.count_ones() & 1));
        }
        Self(out)
    }

    #[inline]
    fn low_ones(n: u32) -> Self {
        debug_assert!(n <= Self::BITS, "mask width {n} > {}", Self::BITS);
        let mut out = [0; N];
        let (q, r) = ((n / 64) as usize, n % 64);
        for limb in out.iter_mut().take(q) {
            *limb = u64::MAX;
        }
        if q < N && r != 0 {
            out[q] = u64::low_ones(r);
        }
        Self(out)
    }

    #[inline]
    fn splat_byte(b: u8) -> Self {
        Self([u64::splat_byte(b); N])
    }

    // Truncation is the point.
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    fn low_byte(self) -> u8 {
        self.0[0] as u8
    }

    #[inline]
    fn bit(self, i: u32) -> bool {
        (self.0[(i / 64) as usize] >> (i % 64)) & 1 != 0
    }
}

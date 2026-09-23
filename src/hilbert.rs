//! Hilbert curves as two scans over the Morton code: parities one way,
//! the carry chain two bits wide the other.
//!
//! A 2D Hilbert index has the shape of a Morton code: one pair of bits
//! per level, most significant level first. The difference is that
//! each level is read in a frame the levels above it chose, one of
//! four: the base curve, transposed, point-reflected, or both. Which
//! frame is a function of the pairs above, and of nothing else, so
//! decoding is two suffix XORs over the levels
//! ([`Bits::suffix_xor`]; Hacker's Delight 16-2 in its parallel-prefix
//! form) and then a Morton decode: straight-line, about a dozen
//! operations, on any [`Word`].
//!
//! Encoding is a scan too, of a heavier kind. Read with the coordinates
//! as input, the frame at each level is an affine map of the frame
//! above it, `(a, c) ↦ (a ^ ¬(c ^ x), c)` when `x = y` and
//! `(a, c) ↦ (c ^ x, a ^ x)` when `x ≠ y`. The linear parts do not
//! commute (they generate `GL(2, 2) ≅ S₃`), so no parity and no adder
//! computes the product, but every map is known before any frame is,
//! and affine maps compose associatively, so Kogge–Stone does. Two
//! involutions multiply to an element of order three, so once levels
//! are paired every window is `v ↦ m·v + t` over GF(4) with `m` in
//! GF(4)*: four bit-sliced words, one GF(4) multiplication per word
//! pair per round, `⌈log₂(BITS / 2)⌉ − 1` rounds after the pairing.
//! That is the adder's carry chain, whose maps are `c ↦ p·c + g` over
//! GF(2), with GF(4) in place of GF(2). The construction is
//! rawrunprotected's (*2D Hilbert curves in O(log n)*,
//! threadlocalmutex.com, 2016, public domain); [`Hilbert2::from_morton`]
//! is it on any [`Word`], in the Morton layout, with laws. Both
//! directions are checked against the textbook `xy2d` / `d2xy` loops
//! and the four-state machine in [`crate::laws::reference`].
//!
//! The curve of order `n` starts at `(0, 0)`, ends at `(2^n − 1, 0)`,
//! and visits the quadrants lower-left, upper-left, upper-right,
//! lower-right, the order of Hacker's Delight figure 16-1 and of the
//! `xy2d` / `d2xy` loops everyone copies. Consecutive indices are
//! adjacent cells (`hilbert_consecutive_are_adjacent` in
//! [`crate::laws`]).
//!
//! ```
//! use hakmem::prelude::*;
//!
//! // Order 2: the 4 × 4 curve, index 12 is the cell (3, 1).
//! let h = Hilbert2::<u8>::encode_order(3, 1, 2);
//! assert_eq!(h.index(), 12);
//! assert_eq!(h.decode_order(2), (3, 1));
//! // The full-width curve of a u32 has 16 levels: a 65536 × 65536 grid.
//! let h = Hilbert2::<u32>::encode(40_000, 7);
//! assert_eq!(h.decode(), (40_000, 7));
//! ```

use crate::bits::Bits;
use crate::dilated::{Dilated, Morton2};
use crate::word::Word;

/// A 2D Hilbert index over the full width of `W`: `BITS / 2` levels,
/// a `2^(BITS/2)` square. Ordered as an integer, which is the curve
/// order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Hilbert2<W: Word>(W);

impl<W: Word> Hilbert2<W> {
    /// Levels of the full-width curve: `BITS / 2`.
    pub const LEVELS: u32 = W::BITS / 2;

    /// The index of `(x, y)` on the full-width curve. Coordinates
    /// above [`LEVELS`](Self::LEVELS) bits are dropped.
    #[inline]
    #[must_use]
    pub fn encode(x: W, y: W) -> Self {
        Self::from_morton(Morton2::encode(x, y))
    }

    /// The `(x, y)` of this index on the full-width curve.
    #[inline]
    #[must_use]
    pub fn decode(self) -> (W, W) {
        self.into_morton().decode()
    }

    /// The index of `(x, y)` on the curve of `order` levels, `order`
    /// in `0..=LEVELS`; coordinates must be below `2^order`
    /// (debug-asserted).
    ///
    /// The order-`n` curve is the first `4^n` cells of the full-width
    /// curve, transposed when `LEVELS − n` is odd: every level above
    /// reads a zero pair, and a zero pair transposes the frame below
    /// it (`hilbert_order_laws`). So this is [`encode`](Self::encode)
    /// with the arguments swapped for odd depth.
    #[inline]
    #[must_use]
    pub fn encode_order(x: W, y: W, order: u32) -> Self {
        debug_assert!(order <= Self::LEVELS, "order {order} > {}", Self::LEVELS);
        debug_assert!(
            x.and(W::low_ones(order).not()).is_zero() && y.and(W::low_ones(order).not()).is_zero(),
            "coordinates above 2^{order}"
        );
        if (Self::LEVELS - order) & 1 == 0 {
            Self::encode(x, y)
        } else {
            Self::encode(y, x)
        }
    }

    /// The `(x, y)` of this index on the curve of `order` levels; the
    /// index must be below `4^order` (debug-asserted).
    #[inline]
    #[must_use]
    pub fn decode_order(self, order: u32) -> (W, W) {
        debug_assert!(order <= Self::LEVELS, "order {order} > {}", Self::LEVELS);
        debug_assert!(
            self.0.and(W::low_ones(2 * order).not()).is_zero(),
            "index above 4^{order}"
        );
        let (x, y) = self.decode();
        if (Self::LEVELS - order) & 1 == 0 {
            (x, y)
        } else {
            (y, x)
        }
    }

    /// Wraps an index.
    #[inline]
    #[must_use]
    pub const fn from_index(index: W) -> Self {
        Self(index)
    }

    /// The raw index.
    #[inline]
    #[must_use]
    pub const fn index(self) -> W {
        self.0
    }

    /// From the Morton code of the same cell: the levels re-read in
    /// their frames, all at once.
    ///
    /// Per level, with `x`, `y` the coordinate bits and `lo = x ^ y`, the
    /// frame map is `(a, c) ↦ (c ^ x, a ^ x)` when `lo` is set, else
    /// `(a, c) ↦ (a ^ ¬(c ^ x), c)`. Windows of two levels have linear
    /// parts in GF(4)* and translations in GF(4); the suffix product of
    /// the windows is a Kogge–Stone scan with one GF(4) multiplication
    /// per word pair per round, and the frame at a level is the
    /// translation of the product above it, the top frame being zero.
    /// rawrunprotected's construction (2016), in the dilated layout.
    #[must_use]
    pub fn from_morton(morton: Morton2<W>) -> Self {
        let code = morton.code();
        let lanes = Dilated::<W, 2>::mask();
        let x = code.and(lanes);
        let y = code.shr(1).and(lanes);
        let lo = x.xor(y);
        let not_lo = lo.xor(lanes);
        let neither = x.or(y).xor(lanes);
        let x_only = x.and(y.xor(lanes));
        // Windows of two levels, composed by hand: their linear parts
        // are products of two involutions, which lie in the cyclic
        // group of order three, GF(4)*. From here on every window is
        // `v ↦ m·v + t` over GF(4), `m` in `(ma, mb)`, `t` in `(ta, tb)`,
        // and a round is one multiplication in GF(4) per word pair.
        let mut ma = lo.or(not_lo.shr(2));
        let mut mb = lo.shr(2).xor(lo);
        let mut ta = neither.shr(2).xor(not_lo.and(x_only.shr(2))).xor(neither);
        let mut tb = lo.and(neither.shr(2)).xor(x_only.shr(2)).xor(x_only);
        let mut step = 4;
        while step < W::BITS {
            let (pa, pb, pc, pd) = (ma, mb, ta, tb);
            let (qa, qb, qc, qd) = (pa.shr(step), pb.shr(step), pc.shr(step), pd.shr(step));
            // `t ^= m · t'`: `(a + bω)(c' + d'ω)` with `ω² = ω + 1`.
            ta = pc.xor(pa.and(qc)).xor(pb.and(qd));
            tb = pd.xor(pb.and(qc)).xor(pa.xor(pb).and(qd));
            // `m ·= m'`, not needed after the last round.
            if step * 2 < W::BITS {
                ma = pa.and(qa).xor(pb.and(qb));
                mb = pa.and(qb).xor(pb.and(qa.xor(qb)));
            }
            step <<= 1;
        }
        // The frames, read off the translations of the products above.
        let swap_like = ta.xor(ta.shr(2));
        let flip_like = tb.xor(tb.shr(2));
        let hi = flip_like.or(lo.or(swap_like).xor(lanes));
        Self(lo.or(hi.shl(1)))
    }

    /// To the Morton code of the same cell: the levels read back out
    /// of their frames, all at once.
    ///
    /// Per level, with `s` the pair: the frame below it transposes
    /// when `s ∈ {0, 3}` and reflects when `s = 3`, so `swap` is the
    /// parity of `¬(s_hi ^ s_lo)` and `flip` the parity of
    /// `s_hi & s_lo` over the levels above, two suffix XORs. Then
    /// `x = s_hi ^ swap·s_lo ^ flip` and `y = x ^ s_lo`.
    #[must_use]
    pub fn into_morton(self) -> Morton2<W> {
        let lanes = Dilated::<W, 2>::mask();
        let s_lo = self.0.and(lanes);
        let s_hi = self.0.shr(1).and(lanes);
        // Even bits of a suffix XOR over one lane hold the inclusive
        // parity; odd bit `2i + 1` the parity over the levels strictly
        // above `i`, the exclusive scan for free.
        let swap = s_hi.xor(s_lo).xor(lanes).suffix_xor().shr(1).and(lanes);
        let flip = s_hi.and(s_lo).suffix_xor().shr(1).and(lanes);
        let x = flip.xor(s_hi).xor(swap.and(s_lo));
        let y = x.xor(s_lo);
        Morton2::from_code(x.or(y.shl(1)))
    }
}

/// One level of the encode machine of `crate::laws::reference`, for the
/// batch table: frame `(swap, flip)` as bits 0 and 1, coordinate bits
/// `x`, `y`; returns the digit and the frame below.
#[cfg_attr(
    not(any(
        all(
            target_arch = "x86_64",
            target_feature = "avx512vbmi",
            not(feature = "portable")
        ),
        all(
            target_arch = "aarch64",
            target_feature = "neon",
            not(feature = "portable")
        )
    )),
    allow(dead_code)
)]
const fn level(frame: u8, x: u8, y: u8) -> (u8, u8) {
    let (swap, flip) = (frame & 1, (frame >> 1) & 1);
    let lo = x ^ y;
    let hi = x ^ flip ^ (swap & lo);
    let swap_below = swap ^ 1 ^ hi ^ lo;
    let flip_below = flip ^ (hi & lo);
    ((hi << 1) | lo, swap_below | (flip_below << 1))
}

/// Two levels of the encode machine a byte, for the batch kernels: the
/// entry at `frame << 4 | nibble`, the nibble being two levels of a
/// Morton code (`y x` of the upper level, then of the lower), holds
/// `frame_below << 4 | digits`. The frame stays in bits 4 and 5, so the
/// next index is the entry masked and the next nibble or-ed in. Sixty-four
/// bytes: one AVX-512 register, or four NEON registers for `tbl`.
#[cfg_attr(
    not(any(
        all(
            target_arch = "x86_64",
            target_feature = "avx512vbmi",
            not(feature = "portable")
        ),
        all(
            target_arch = "aarch64",
            target_feature = "neon",
            not(feature = "portable")
        )
    )),
    allow(dead_code)
)]
const TABLE4: [u8; 64] = {
    let mut table = [0u8; 64];
    let mut i = 0u8;
    while i < 64 {
        let (frame, nibble) = (i >> 4, i & 15);
        let (d_hi, f_mid) = level(frame, (nibble >> 2) & 1, (nibble >> 3) & 1);
        let (d_lo, f_below) = level(f_mid, nibble & 1, (nibble >> 1) & 1);
        table[i as usize] = (f_below << 4) | (d_hi << 2) | d_lo;
        i += 1;
    }
    table
};

macro_rules! hilbert2_batch {
    ($($w:ty => $kernel:ident),* $(,)?) => {$(
        impl Hilbert2<$w> {
            /// [`from_morton`](Self::from_morton) over a slice of keys, in
            /// place: each Morton code becomes the Hilbert index of the
            /// same cell.
            ///
            /// With AVX-512 VBMI a step is one `vpermb` through a 64-entry
            /// table per register of keys, two levels at a time, the frame
            /// riding in the index byte; with NEON the same table is four
            /// registers for `tbl`. Otherwise, and for the keys past the
            /// last whole batch, it is [`from_morton`](Self::from_morton)
            /// per key. Coordinates never enter: fill the slice with
            /// [`Morton2::encode`] from whatever layout the points are in.
            pub fn from_morton_in_place(keys: &mut [$w]) {
                let done = batch::$kernel(keys);
                for key in &mut keys[done..] {
                    *key = Self::from_morton(Morton2::from_code(*key)).index();
                }
            }

            /// [`into_morton`](Self::into_morton) over a slice of keys, in
            /// place: each Hilbert index becomes the Morton code of the
            /// same cell. Per key; the decode is two suffix XORs and has no
            /// table to batch.
            pub fn into_morton_in_place(keys: &mut [$w]) {
                for key in keys {
                    *key = Self::from_index(*key).into_morton().code();
                }
            }

            /// [`from_morton_in_place`](Self::from_morton_in_place) on the
            /// curve of `order` levels, as [`encode_order`](Self::encode_order)
            /// per key; `order` in `0..=LEVELS`, codes below `4^order`
            /// (debug-asserted). The order-`n` curve is the full-width one
            /// with `x` and `y` swapped when `LEVELS − n` is odd: a swap of
            /// the Morton lanes per key, then the full-width batch.
            pub fn from_morton_in_place_order(keys: &mut [$w], order: u32) {
                debug_assert!(order <= Self::LEVELS, "order {order} > {}", Self::LEVELS);
                debug_assert!(
                    keys.iter().all(|&k| k.checked_shr(2 * order).unwrap_or(0) == 0),
                    "codes above 4^{order}"
                );
                if (Self::LEVELS - order) & 1 == 1 {
                    let even = Dilated::<$w, 2>::mask();
                    for key in keys.iter_mut() {
                        *key = ((*key & even) << 1) | ((*key >> 1) & even);
                    }
                }
                Self::from_morton_in_place(keys);
            }

            /// [`into_morton_in_place`](Self::into_morton_in_place) on the
            /// curve of `order` levels, as [`decode_order`](Self::decode_order)
            /// per key; indices below `4^order` (debug-asserted).
            pub fn into_morton_in_place_order(keys: &mut [$w], order: u32) {
                debug_assert!(order <= Self::LEVELS, "order {order} > {}", Self::LEVELS);
                debug_assert!(
                    keys.iter().all(|&k| k.checked_shr(2 * order).unwrap_or(0) == 0),
                    "indices above 4^{order}"
                );
                Self::into_morton_in_place(keys);
                if (Self::LEVELS - order) & 1 == 1 {
                    let even = Dilated::<$w, 2>::mask();
                    for key in keys.iter_mut() {
                        *key = ((*key & even) << 1) | ((*key >> 1) & even);
                    }
                }
            }
        }
    )*};
}

hilbert2_batch!(u32 => from_morton_u32, u64 => from_morton_u64);

/// The batch kernels. Each returns how many keys from the front it
/// converted, a whole number of batches; the caller finishes the rest.
/// The intrinsics are `unsafe` solely because they require the target
/// feature, which `cfg` makes a compile-time fact, as in `word` and
/// `lanes`.
mod batch {
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx512vbmi",
        not(feature = "portable")
    ))]
    #[allow(unsafe_code)]
    mod vbmi {
        use core::arch::x86_64::{
            __m512i, _mm512_and_si512, _mm512_loadu_si512, _mm512_or_si512,
            _mm512_permutexvar_epi8, _mm512_set1_epi32, _mm512_set1_epi64, _mm512_setzero_si512,
            _mm512_slli_epi32, _mm512_slli_epi64, _mm512_srli_epi32, _mm512_srli_epi64,
            _mm512_storeu_si512,
        };

        use super::super::TABLE4;

        macro_rules! kernel {
            ($name:ident, $w:ty, $lanes:literal, $set1:ident, $srli:ident, $slli:ident) => {
                /// Batches of 64 keys, `64 / LANES` registers in flight.
                pub(in crate::hilbert) fn $name(keys: &mut [$w]) -> usize {
                    const REGS: usize = 64 / $lanes;
                    const STEPS: u32 = <$w>::BITS / 4;
                    let (chunks, _) = keys.as_chunks_mut::<64>();
                    let done = chunks.len() * 64;
                    for chunk in chunks {
                        // SAFETY: AVX-512 F, BW and VBMI are enabled by cfg;
                        // every load and store stays inside the 64 keys of
                        // `chunk`, unaligned forms.
                        unsafe {
                            let table = _mm512_loadu_si512(TABLE4.as_ptr().cast());
                            let nibble = $set1(15);
                            let frame_bits = $set1(0x30);
                            let mut code = [_mm512_setzero_si512(); REGS];
                            let mut acc = [_mm512_setzero_si512(); REGS];
                            let mut frame = [_mm512_setzero_si512(); REGS];
                            for (r, c) in code.iter_mut().enumerate() {
                                *c = _mm512_loadu_si512(chunk.as_ptr().add($lanes * r).cast());
                            }
                            // Levels from the top, two a step; the shift is
                            // an immediate per step, hence the unrolled match.
                            let mut step = STEPS;
                            while step > 0 {
                                step -= 1;
                                for r in 0..REGS {
                                    let shifted: __m512i = match step {
                                        0 => code[r],
                                        1 => $srli::<4>(code[r]),
                                        2 => $srli::<8>(code[r]),
                                        3 => $srli::<12>(code[r]),
                                        4 => $srli::<16>(code[r]),
                                        5 => $srli::<20>(code[r]),
                                        6 => $srli::<24>(code[r]),
                                        7 => $srli::<28>(code[r]),
                                        8 => $srli::<32>(code[r]),
                                        9 => $srli::<36>(code[r]),
                                        10 => $srli::<40>(code[r]),
                                        11 => $srli::<44>(code[r]),
                                        12 => $srli::<48>(code[r]),
                                        13 => $srli::<52>(code[r]),
                                        14 => $srli::<56>(code[r]),
                                        _ => $srli::<60>(code[r]),
                                    };
                                    let index = _mm512_or_si512(
                                        _mm512_and_si512(shifted, nibble),
                                        frame[r],
                                    );
                                    let entry = _mm512_permutexvar_epi8(index, table);
                                    acc[r] = _mm512_or_si512(
                                        $slli::<4>(acc[r]),
                                        _mm512_and_si512(entry, nibble),
                                    );
                                    frame[r] = _mm512_and_si512(entry, frame_bits);
                                }
                            }
                            for (r, a) in acc.iter().enumerate() {
                                _mm512_storeu_si512(chunk.as_mut_ptr().add($lanes * r).cast(), *a);
                            }
                        }
                    }
                    done
                }
            };
        }

        kernel!(
            from_morton_u32,
            u32,
            16,
            _mm512_set1_epi32,
            _mm512_srli_epi32,
            _mm512_slli_epi32
        );
        kernel!(
            from_morton_u64,
            u64,
            8,
            _mm512_set1_epi64,
            _mm512_srli_epi64,
            _mm512_slli_epi64
        );
    }

    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx512vbmi",
        not(feature = "portable")
    ))]
    pub(super) use vbmi::{from_morton_u32, from_morton_u64};

    #[cfg(all(
        target_arch = "aarch64",
        target_feature = "neon",
        not(feature = "portable")
    ))]
    #[allow(unsafe_code)]
    mod neon {
        use core::arch::aarch64::{
            vandq_u32, vandq_u64, vdupq_n_s32, vdupq_n_s64, vdupq_n_u32, vdupq_n_u64, vld1q_u8_x4,
            vld1q_u32, vld1q_u64, vorrq_u32, vorrq_u64, vqtbl4q_u8, vreinterpretq_u8_u32,
            vreinterpretq_u8_u64, vreinterpretq_u32_u8, vreinterpretq_u64_u8, vshlq_n_u32,
            vshlq_n_u64, vshlq_u32, vshlq_u64, vst1q_u32, vst1q_u64,
        };

        use super::super::TABLE4;

        /// Batches of 16 keys: eight registers of two.
        pub(in crate::hilbert) fn from_morton_u64(keys: &mut [u64]) -> usize {
            let (chunks, _) = keys.as_chunks_mut::<16>();
            let done = chunks.len() * 16;
            for chunk in chunks {
                // SAFETY: NEON is enabled by cfg; every load and store stays
                // inside the 16 keys of `chunk`.
                unsafe {
                    let table = vld1q_u8_x4(TABLE4.as_ptr());
                    let nibble = vdupq_n_u64(15);
                    let frame_bits = vdupq_n_u64(0x30);
                    let mut code = [vdupq_n_u64(0); 8];
                    let mut acc = [vdupq_n_u64(0); 8];
                    let mut frame = [vdupq_n_u64(0); 8];
                    for (r, c) in code.iter_mut().enumerate() {
                        *c = vld1q_u64(chunk.as_ptr().add(2 * r));
                    }
                    let mut step = 16i64;
                    while step > 0 {
                        step -= 1;
                        let shift = vdupq_n_s64(-4 * step);
                        for r in 0..8 {
                            let index =
                                vorrq_u64(vandq_u64(vshlq_u64(code[r], shift), nibble), frame[r]);
                            let entry = vreinterpretq_u64_u8(vqtbl4q_u8(
                                table,
                                vreinterpretq_u8_u64(index),
                            ));
                            acc[r] = vorrq_u64(vshlq_n_u64::<4>(acc[r]), vandq_u64(entry, nibble));
                            frame[r] = vandq_u64(entry, frame_bits);
                        }
                    }
                    for (r, a) in acc.iter().enumerate() {
                        vst1q_u64(chunk.as_mut_ptr().add(2 * r), *a);
                    }
                }
            }
            done
        }

        /// Batches of 32 keys: eight registers of four.
        pub(in crate::hilbert) fn from_morton_u32(keys: &mut [u32]) -> usize {
            let (chunks, _) = keys.as_chunks_mut::<32>();
            let done = chunks.len() * 32;
            for chunk in chunks {
                // SAFETY: as above.
                unsafe {
                    let table = vld1q_u8_x4(TABLE4.as_ptr());
                    let nibble = vdupq_n_u32(15);
                    let frame_bits = vdupq_n_u32(0x30);
                    let mut code = [vdupq_n_u32(0); 8];
                    let mut acc = [vdupq_n_u32(0); 8];
                    let mut frame = [vdupq_n_u32(0); 8];
                    for (r, c) in code.iter_mut().enumerate() {
                        *c = vld1q_u32(chunk.as_ptr().add(4 * r));
                    }
                    let mut step = 8i32;
                    while step > 0 {
                        step -= 1;
                        let shift = vdupq_n_s32(-4 * step);
                        for r in 0..8 {
                            let index =
                                vorrq_u32(vandq_u32(vshlq_u32(code[r], shift), nibble), frame[r]);
                            let entry = vreinterpretq_u32_u8(vqtbl4q_u8(
                                table,
                                vreinterpretq_u8_u32(index),
                            ));
                            acc[r] = vorrq_u32(vshlq_n_u32::<4>(acc[r]), vandq_u32(entry, nibble));
                            frame[r] = vandq_u32(entry, frame_bits);
                        }
                    }
                    for (r, a) in acc.iter().enumerate() {
                        vst1q_u32(chunk.as_mut_ptr().add(4 * r), *a);
                    }
                }
            }
            done
        }
    }

    #[cfg(all(
        target_arch = "aarch64",
        target_feature = "neon",
        not(feature = "portable")
    ))]
    pub(super) use neon::{from_morton_u32, from_morton_u64};

    /// No batch path: the caller converts every key.
    #[cfg(not(any(
        all(
            target_arch = "x86_64",
            target_feature = "avx512vbmi",
            not(feature = "portable")
        ),
        all(
            target_arch = "aarch64",
            target_feature = "neon",
            not(feature = "portable")
        )
    )))]
    pub(super) const fn from_morton_u32(_: &mut [u32]) -> usize {
        0
    }

    #[cfg(not(any(
        all(
            target_arch = "x86_64",
            target_feature = "avx512vbmi",
            not(feature = "portable")
        ),
        all(
            target_arch = "aarch64",
            target_feature = "neon",
            not(feature = "portable")
        )
    )))]
    pub(super) const fn from_morton_u64(_: &mut [u64]) -> usize {
        0
    }
}

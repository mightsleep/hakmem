//! 3D Hilbert curves: the decode as a Kogge–Stone scan over
//! `AGL(1, 4)`, the encode as the twelve-state machine in the same
//! arithmetic, both without tables.
//!
//! The 3D curve of rawrunprotected's *3D Hilbert curves in O(log n)
//! optimised* (2020), which he ships as two 96-byte tables: at each
//! level the index triple `i` names an octant `q(i) = i ^ (i >> 1)`
//! (the Gray code) in a frame, and the frame below is the frame above
//! composed with a map `g(i)`. The frames form the alternating group
//! `A₄`, acting on octants as a cyclic rotation of the axes followed by
//! an even number of reflections, `o ↦ rot_r(o) ^ c`. That group is
//! `AGL(1, 4)`: split an octant into its parity and its even part, the
//! even octants `{000, 011, 110, 101}` are `GF(4)` with rotation as
//! multiplication by `ω`, and a frame is `e ↦ m·e + t` with `m` in
//! `GF(4)*` and `t` in `GF(4)`, the parity riding along untouched. So
//! the frames compose like the windows of the 2D encode, one GF(4)
//! multiplication per word pair, and the decode, whose maps depend on
//! the index alone, is a suffix product: `⌈log₂ LEVELS⌉` rounds of
//! about 20 bit-sliced operations, 10 ns a point on a `u64` against
//! 16 ns for the table loop. He wrote in 2020 that 3D stays linear
//! because the mod-3 circuit is too costly; the mod-3 circuit is GF(4)
//! multiplication, five operations.
//!
//! The encode is not a scan. Its maps on the twelve frames are not
//! permutations (each has eight images), the monoid they generate
//! passes a million elements at word length seven, and the all-zero
//! word never resets it, so neither a group representation nor a
//! window exists. [`Hilbert3::from_morton`] walks the levels through
//! a table of 96 bytes that a `const fn` builds from
//! the same GF(4) step ([`encode_step`]); the algebraic loop is three
//! to four times slower, its chain being ten operations a level
//! against one load. Both directions are checked against the tables
//! (`tests/hilbert3.rs`), against the algebraic machine in
//! [`crate::laws::reference`], and by the path property.
//!
//! ```
//! use hakmem::prelude::*;
//!
//! let h = Hilbert3::<u64>::encode(1000, 2000, 3000);
//! assert_eq!(h.decode(), (1000, 2000, 3000));
//! // Order 2: the 4 × 4 × 4 curve.
//! let cells: Vec<(u8, u8, u8)> = (0..64u8)
//!     .map(|h| Hilbert3::from_index(h).decode_order(2))
//!     .collect();
//! assert_eq!(cells[0], (0, 0, 0));
//! for w in cells.windows(2) {
//!     let ((x0, y0, z0), (x1, y1, z1)) = (w[0], w[1]);
//!     assert_eq!(x0.abs_diff(x1) + y0.abs_diff(y1) + z0.abs_diff(z1), 1);
//! }
//! ```

use crate::dilated::{Dilated, Morton3};
use crate::word::Word;

/// A 3D Hilbert index over `BITS / 3` levels of `W`: a cube of side
/// `2^(BITS/3)`. Ordered as an integer, which is the curve order.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Hilbert3<W: Word>(W);

/// GF(4) product of bit-sliced elements `a + bω`, `ω² = ω + 1`.
#[inline]
fn gf4_mul<W: Word>((a, b): (W, W), (c, d): (W, W)) -> (W, W) {
    (a.and(c).xor(b.and(d)), a.and(d).xor(b.and(c)).xor(b.and(d)))
}

/// The map `g(i)` of every level at once, in the lane, from the index
/// bits `i0, i1, i2` of every level: `m = (par ^ maj, ¬(i2 ^ maj))`,
/// `t = (i0·i1 | i2·¬(i0|i1), i2·(i0|i1))`, read off the tables and
/// verified against them (`tests/hilbert3.rs`). With `u8` scalars
/// holding one bit and `lane = 1` it is the machine's step.
#[inline]
fn level_maps<W: Word>(i0: W, i1: W, i2: W, lane: W) -> ((W, W), (W, W)) {
    let both = i0.and(i1);
    let either = i0.or(i1);
    let maj = both.or(i2.and(i0.xor(i1)));
    let par = i0.xor(i1).xor(i2);
    let m = (par.xor(maj), i2.xor(maj).xor(lane));
    let t = (both.or(i2.and(either.xor(lane))), i2.and(either));
    (m, t)
}

/// Cyclic rotation of the axes, `(x, y, z) ↦ (z, x, y)`, `k` times:
/// the frame after `k` zero index triples.
#[inline]
const fn rotate_axes<W: Word>(c: (W, W, W), k: u32) -> (W, W, W) {
    match k % 3 {
        0 => c,
        1 => (c.2, c.0, c.1),
        _ => (c.1, c.2, c.0),
    }
}

/// [`rotate_axes`] on Morton codes in place: `k` in `0..3`, every
/// triple of bits `x y z` (low to high) rotated so that the coordinate
/// order becomes that of `rotate_axes(_, k)`.
fn rotate_triples<W: Word>(keys: &mut [W], k: u32) {
    let x = Dilated::<W, 3>::mask();
    let (y, z) = (x.shl(1), x.shl(2));
    match k {
        // (x, y, z) ↦ (z, x, y): x moves up to y, y to z, z down to x.
        1 => {
            for key in keys {
                *key = key.shl(1).and(y.or(z)).or(key.shr(2).and(x));
            }
        }
        // (x, y, z) ↦ (y, z, x): y moves down to x, z to y, x up to z.
        2 => {
            for key in keys {
                *key = key.shr(1).and(x.or(y)).or(key.and(x).shl(2));
            }
        }
        _ => {}
    }
}

impl<W: Word> Hilbert3<W> {
    /// Levels of the full-width curve: `BITS / 3`.
    pub const LEVELS: u32 = W::BITS / 3;

    /// The index of `(x, y, z)` on the full-width curve. Coordinates
    /// above [`LEVELS`](Self::LEVELS) bits are dropped.
    #[inline]
    #[must_use]
    pub fn encode(x: W, y: W, z: W) -> Self {
        Self::from_morton(Morton3::encode(x, y, z))
    }

    /// The `(x, y, z)` of this index on the full-width curve.
    #[inline]
    #[must_use]
    pub fn decode(self) -> (W, W, W) {
        self.into_morton().decode()
    }

    /// The index on the curve of `order` levels, `order` in
    /// `0..=LEVELS`; coordinates must be below `2^order`
    /// (debug-asserted). The order-`n` curve is the first `8^n` cells
    /// of the full-width curve in the frame the `LEVELS − n` zero
    /// triples above leave, a rotation of the axes by that count
    /// modulo three (`hilbert3_order_laws`).
    #[inline]
    #[must_use]
    pub fn encode_order(x: W, y: W, z: W, order: u32) -> Self {
        debug_assert!(order <= Self::LEVELS, "order {order} > {}", Self::LEVELS);
        let above = W::low_ones(order).not();
        debug_assert!(
            x.and(above).is_zero() && y.and(above).is_zero() && z.and(above).is_zero(),
            "coordinates above 2^{order}"
        );
        // The zero levels above leave the frame rotated by their count;
        // the order-`n` curve is the full curve read through that frame.
        let (x, y, z) = rotate_axes((x, y, z), Self::LEVELS - order);
        Self::encode(x, y, z)
    }

    /// The `(x, y, z)` of this index on the curve of `order` levels;
    /// the index must be below `8^order` (debug-asserted).
    #[inline]
    #[must_use]
    pub fn decode_order(self, order: u32) -> (W, W, W) {
        debug_assert!(order <= Self::LEVELS, "order {order} > {}", Self::LEVELS);
        debug_assert!(
            self.0.and(W::low_ones(3 * order).not()).is_zero(),
            "index above 8^{order}"
        );
        rotate_axes(self.decode(), 2 * (Self::LEVELS - order))
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

    /// The lane of level bits: positions `0, 3, 6, …` below `3 · LEVELS`.
    #[inline]
    fn lane() -> W {
        Dilated::<W, 3>::mask().and(W::low_ones(3 * Self::LEVELS))
    }

    /// To the Morton code of the same cell: the frames as a suffix
    /// product of the level maps, then each octant placed in its frame.
    ///
    /// The maps are stored as `n = m + 1`, so the zero a shift brings
    /// in above the top level is the identity; a round composes each
    /// window with the window `s` levels above it, outer: `n_c = n ^ n'
    /// ^ n·n'`, `t_c = t ^ t' ^ n'·t`. The frame at a level is the
    /// product of the maps above it, the top frame being the identity.
    #[must_use]
    pub fn into_morton(self) -> Morton3<W> {
        let lane = Self::lane();
        let i0 = self.0.and(lane);
        let i1 = self.0.shr(1).and(lane);
        let i2 = self.0.shr(2).and(lane);
        let ((ma, mb), (mut ta, mut tb)) = level_maps(i0, i1, i2, lane);
        let (mut na, mut nb) = (ma.xor(lane), mb);
        let mut step = 3;
        while step < 3 * Self::LEVELS {
            let above_n = (na.shr(step), nb.shr(step));
            let above_t = (ta.shr(step), tb.shr(step));
            let (pa, pb) = gf4_mul(above_n, (na, nb));
            let (qa, qb) = gf4_mul(above_n, (ta, tb));
            (ta, tb) = (ta.xor(above_t.0).xor(qa), tb.xor(above_t.1).xor(qb));
            (na, nb) = (na.xor(above_n.0).xor(pa), nb.xor(above_n.1).xor(pb));
            step <<= 1;
        }
        // The frame at a level: the product of the maps above it.
        let frame_m = (na.shr(3).xor(lane), nb.shr(3));
        let frame_t = (ta.shr(3), tb.shr(3));
        // The octant `q(i) = i ^ (i >> 1)` as `(e, p)`: `e = (i1, i0 ^ i2)`,
        // `p = i0`; placed in its frame, `e ↦ m·e + t`; back to bits.
        let (ea, eb) = gf4_mul(frame_m, (i1, i0.xor(i2)));
        let (ea, eb) = (ea.xor(frame_t.0), eb.xor(frame_t.1));
        let p = i0;
        let v0 = ea.xor(p);
        let v1 = ea.xor(eb).xor(p);
        let v2 = eb.xor(p);
        Morton3::from_code(v0.or(v1.shl(1)).or(v2.shl(2)))
    }

    /// From the Morton code of the same cell: the twelve-state machine,
    /// one level at a time, memoised.
    ///
    /// The machine's step is GF(4) arithmetic on the frame, about 25
    /// operations with a chain of ten, and a level cannot start before
    /// the frame above it is known; a table lookup is one load. So this
    /// is a table: [`ENCODE_TABLE`], 96 bytes,
    /// `state · 8 + octant → state · 8 + triple`, built at compile time
    /// from the same arithmetic ([`encode_step`]) and checked against
    /// the published constants (`tests/hilbert3.rs`). The machine
    /// itself is `laws::reference::hilbert3_index_machine`.
    #[must_use]
    pub fn from_morton(m: Morton3<W>) -> Self {
        let code = m.code();
        let mut state = 0usize;
        let mut index = W::ZERO;
        let mut level = Self::LEVELS;
        while level > 0 {
            level -= 1;
            let octant = code.shr(3 * level).and(W::low_ones(3)).low_byte() as usize;
            // Padded to 128 entries so the mask proves the index in range:
            // no bounds check, and the loop over keys still vectorises
            // (gathers under AVX-512), which a restructured body did not.
            let entry = usize::from(ENCODE_PADDED[(state | octant) & 127]);
            state = entry & !7;
            // Truncation is the point: the low three bits are the triple.
            #[allow(clippy::cast_possible_truncation)]
            let triple = W::splat_byte(entry as u8 & 7).and(W::low_ones(3));
            index = index.or(triple.shl(3 * level));
        }
        Self(index)
    }
}

/// GF(4) product on one-bit scalars, `ω² = ω + 1`, for the compile-time
/// table.
const fn gf4_scalar(a: u8, b: u8, c: u8, d: u8) -> (u8, u8) {
    (a & c ^ b & d, a & d ^ b & c ^ b & d)
}

/// One step of the twelve-state encode machine.
///
/// The frame is `(m, t)` in GF(4), numbered `m_index · 4 + t` with `m`
/// in `1, ω, ω²` as `0, 1, 2` and `t = t_a | t_b << 1`. Returns the
/// index triple of the octant `v0 | v1 << 1 | v2 << 2` and the frame
/// below.
///
/// The octant `(e, p)` in the frame is `e' = m⁻¹·(e + t)` with
/// `m⁻¹ = m²`; the triple is `(p, e'_a, e'_b ^ p)`, the Gray decode of
/// the frame octant; the frame below is `(m·m_g, t + m·t_g)` with `g`
/// the map of the triple (the same formulas as `level_maps`).
#[must_use]
pub const fn encode_step(state: u8, octant: u8) -> (u8, u8) {
    let (ma, mb) = match state / 4 {
        0 => (1, 0),
        1 => (0, 1),
        _ => (1, 1),
    };
    let (ta, tb) = (state & 1, (state >> 1) & 1);
    let (v0, v1, v2) = (octant & 1, (octant >> 1) & 1, (octant >> 2) & 1);
    let p = v0 ^ v1 ^ v2;
    let (fa, fb) = gf4_scalar(ma ^ mb, mb, v0 ^ p ^ ta, v2 ^ p ^ tb);
    let (i0, i1, i2) = (p, fa, fb ^ p);
    let both = i0 & i1;
    let either = i0 | i1;
    let maj = both | (i2 & (i0 ^ i1));
    let par = i0 ^ i1 ^ i2;
    let (ga, gb) = (par ^ maj, (i2 ^ maj) ^ 1);
    let (ha, hb) = (both | (i2 & (either ^ 1)), i2 & either);
    let (sa, sb) = gf4_scalar(ma, mb, ha, hb);
    let (na, nb) = gf4_scalar(ma, mb, ga, gb);
    let m_index = match (na, nb) {
        (1, 0) => 0,
        (0, 1) => 1,
        _ => 2,
    };
    (
        i0 | i1 << 1 | i2 << 2,
        m_index * 4 + ((ta ^ sa) | (tb ^ sb) << 1),
    )
}

/// The encode machine memoised: entry `state · 8 + octant` holds
/// `state_below · 8 + triple`. State 0 is the identity frame.
pub const ENCODE_TABLE: [u8; 96] = {
    let mut table = [0u8; 96];
    let mut state = 0u8;
    while state < 12 {
        let mut octant = 0u8;
        while octant < 8 {
            let (triple, below) = encode_step(state, octant);
            table[(state * 8 + octant) as usize] = below * 8 + triple;
            octant += 1;
        }
        state += 1;
    }
    table
};

/// [`ENCODE_TABLE`] padded to 128 entries: an index masked with `127`
/// is provably in range, and two AVX-512 registers hold it for
/// `vpermi2b`.
const ENCODE_PADDED: [u8; 128] = {
    let mut t = [0u8; 128];
    let mut i = 0;
    while i < 96 {
        t[i] = ENCODE_TABLE[i];
        i += 1;
    }
    t
};

/// The decode machine as a table, for the batch kernels: entry
/// `state · 8 + triple` holds `state_below · 8 + octant`. For a fixed
/// state [`encode_step`] is a bijection of the octants, so this is its
/// inverse level by level, with the same states; built here and
/// checked to be a bijection at compile time. Padded to 128 like
/// [`ENCODE_PADDED`].
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
const DECODE_PADDED: [u8; 128] = {
    let mut t = [0u8; 128];
    let mut seen = [false; 96];
    let mut state = 0u8;
    while state < 12 {
        let mut octant = 0u8;
        while octant < 8 {
            let (triple, below) = encode_step(state, octant);
            let at = (state * 8 + triple) as usize;
            assert!(!seen[at], "encode_step is not a bijection of the octants");
            seen[at] = true;
            t[at] = below * 8 + octant;
            octant += 1;
        }
        state += 1;
    }
    t
};

macro_rules! hilbert3_batch {
    ($($w:ty => $encode:ident, $decode:ident),* $(,)?) => {$(
        impl Hilbert3<$w> {
            /// [`from_morton`](Self::from_morton) over a slice of keys, in
            /// place: each Morton code becomes the Hilbert index of the
            /// same cell.
            ///
            /// The encode has no algebra (the maps on the twelve frames
            /// are not permutations), which is the case a shuffle serves:
            /// with AVX-512 VBMI a level is one `vpermi2b` through
            /// [`ENCODE_TABLE`], padded to 128 bytes in two registers, per
            /// register of keys, the frame riding in the index byte as
            /// `state · 8`; with NEON the table is `tbl` over four
            /// registers and `tbx` over two. Otherwise, and for the keys
            /// past the last whole batch, it is
            /// [`from_morton`](Self::from_morton) per key.
            pub fn from_morton_in_place(keys: &mut [$w]) {
                let done = batch::$encode(keys);
                for key in &mut keys[done..] {
                    *key = Self::from_morton(Morton3::from_code(*key)).index();
                }
            }

            /// [`into_morton`](Self::into_morton) over a slice of keys, in
            /// place: each Hilbert index becomes the Morton code of the
            /// same cell.
            ///
            /// The decode as a machine is the same twelve states read the
            /// other way, so the same kernel runs it through the inverse
            /// table, a level a step; per key it is the algebraic scan
            /// ([`into_morton`](Self::into_morton)), which beats a table
            /// walk, and that finishes the keys past the last whole batch.
            pub fn into_morton_in_place(keys: &mut [$w]) {
                let done = batch::$decode(keys);
                for key in &mut keys[done..] {
                    *key = Self::from_index(*key).into_morton().code();
                }
            }

            /// [`from_morton_in_place`](Self::from_morton_in_place) on the
            /// curve of `order` levels, as [`encode_order`](Self::encode_order)
            /// per key; `order` in `0..=LEVELS`, codes below `8^order`
            /// (debug-asserted). The axes rotate by `LEVELS − order`
            /// modulo three, which in a Morton code rotates every triple of
            /// bits: one pass over the keys, then the full-width batch.
            pub fn from_morton_in_place_order(keys: &mut [$w], order: u32) {
                debug_assert!(order <= Self::LEVELS, "order {order} > {}", Self::LEVELS);
                debug_assert!(
                    keys.iter().all(|&k| k.checked_shr(3 * order).unwrap_or(0) == 0),
                    "codes above 8^{order}"
                );
                rotate_triples(keys, (Self::LEVELS - order) % 3);
                Self::from_morton_in_place(keys);
            }

            /// [`into_morton_in_place`](Self::into_morton_in_place) on the
            /// curve of `order` levels, as [`decode_order`](Self::decode_order)
            /// per key; indices below `8^order` (debug-asserted).
            pub fn into_morton_in_place_order(keys: &mut [$w], order: u32) {
                debug_assert!(order <= Self::LEVELS, "order {order} > {}", Self::LEVELS);
                debug_assert!(
                    keys.iter().all(|&k| k.checked_shr(3 * order).unwrap_or(0) == 0),
                    "indices above 8^{order}"
                );
                Self::into_morton_in_place(keys);
                rotate_triples(keys, 2 * (Self::LEVELS - order) % 3);
            }
        }
    )*};
}

hilbert3_batch!(
    u32 => from_morton_u32, into_morton_u32,
    u64 => from_morton_u64, into_morton_u64,
);

/// The batch kernels; each returns how many keys from the front it
/// converted, a whole number of batches. The intrinsics are `unsafe`
/// solely because they require the target feature, which `cfg` makes a
/// compile-time fact.
mod batch {
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx512vbmi",
        not(feature = "portable")
    ))]
    #[allow(unsafe_code)]
    mod vbmi {
        use core::arch::x86_64::{
            _mm512_and_si512, _mm512_loadu_si512, _mm512_or_si512, _mm512_permutex2var_epi8,
            _mm512_set1_epi32, _mm512_set1_epi64, _mm512_setzero_si512, _mm512_slli_epi32,
            _mm512_slli_epi64, _mm512_srlv_epi32, _mm512_srlv_epi64, _mm512_storeu_si512,
        };

        use super::super::{DECODE_PADDED, ENCODE_PADDED};

        macro_rules! kernel {
            ($name:ident, $table:ident, $w:ty, $lanes:literal, $set1:ident, $srlv:ident, $slli:ident) => {
                /// Batches of 64 keys, `64 / LANES` registers in flight.
                pub(in crate::hilbert3) fn $name(keys: &mut [$w]) -> usize {
                    const REGS: usize = 64 / $lanes;
                    #[allow(clippy::cast_possible_truncation)]
                    const LEVELS: u8 = (<$w>::BITS / 3) as u8;

                    let (chunks, _) = keys.as_chunks_mut::<64>();
                    let done = chunks.len() * 64;
                    for chunk in chunks {
                        // SAFETY: AVX-512 F, BW and VBMI are enabled by cfg;
                        // every load and store stays inside the 64 keys of
                        // `chunk` or the 128 bytes of the table.
                        unsafe {
                            let lo = _mm512_loadu_si512($table.as_ptr().cast());
                            let hi = _mm512_loadu_si512($table.as_ptr().add(64).cast());
                            let octant = $set1(7);
                            let state_bits = $set1(0x78);
                            let mut code = [_mm512_setzero_si512(); REGS];
                            let mut acc = [_mm512_setzero_si512(); REGS];
                            let mut state = [_mm512_setzero_si512(); REGS];
                            for (r, c) in code.iter_mut().enumerate() {
                                *c = _mm512_loadu_si512(chunk.as_ptr().add($lanes * r).cast());
                            }
                            let mut level = LEVELS;
                            while level > 0 {
                                level -= 1;
                                let shift = $set1((3 * level).into());
                                for r in 0..REGS {
                                    let index = _mm512_or_si512(
                                        _mm512_and_si512($srlv(code[r], shift), octant),
                                        state[r],
                                    );
                                    let entry = _mm512_permutex2var_epi8(lo, index, hi);
                                    acc[r] = _mm512_or_si512(
                                        $slli::<3>(acc[r]),
                                        _mm512_and_si512(entry, octant),
                                    );
                                    state[r] = _mm512_and_si512(entry, state_bits);
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
            ENCODE_PADDED,
            u32,
            16,
            _mm512_set1_epi32,
            _mm512_srlv_epi32,
            _mm512_slli_epi32
        );
        kernel!(
            from_morton_u64,
            ENCODE_PADDED,
            u64,
            8,
            _mm512_set1_epi64,
            _mm512_srlv_epi64,
            _mm512_slli_epi64
        );
        kernel!(
            into_morton_u32,
            DECODE_PADDED,
            u32,
            16,
            _mm512_set1_epi32,
            _mm512_srlv_epi32,
            _mm512_slli_epi32
        );
        kernel!(
            into_morton_u64,
            DECODE_PADDED,
            u64,
            8,
            _mm512_set1_epi64,
            _mm512_srlv_epi64,
            _mm512_slli_epi64
        );
    }

    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx512vbmi",
        not(feature = "portable")
    ))]
    pub(super) use vbmi::{from_morton_u32, from_morton_u64, into_morton_u32, into_morton_u64};

    #[cfg(all(
        target_arch = "aarch64",
        target_feature = "neon",
        not(feature = "portable")
    ))]
    #[allow(unsafe_code)]
    mod neon {
        use core::arch::aarch64::{
            vandq_u32, vandq_u64, vdupq_n_s32, vdupq_n_s64, vdupq_n_u8, vdupq_n_u32, vdupq_n_u64,
            vld1q_u8_x2, vld1q_u8_x4, vld1q_u32, vld1q_u64, vorrq_u32, vorrq_u64, vqtbl4q_u8,
            vqtbx2q_u8, vreinterpretq_u8_u32, vreinterpretq_u8_u64, vreinterpretq_u32_u8,
            vreinterpretq_u64_u8, vshlq_n_u32, vshlq_n_u64, vshlq_u32, vshlq_u64, vst1q_u32,
            vst1q_u64, vsubq_u8,
        };

        use super::super::{DECODE_PADDED, ENCODE_PADDED};

        macro_rules! kernel {
            (
                $name:ident, $table:ident, $w:ty, $per_reg:literal, $dup:ident, $dup_s:ident, $s:ty,
                $ld:ident, $st:ident, $and:ident, $or:ident, $shl:ident, $shl_n:ident,
                $to_u8:ident, $from_u8:ident
            ) => {
                /// Batches of eight registers.
                pub(in crate::hilbert3) fn $name(keys: &mut [$w]) -> usize {
                    #[allow(clippy::cast_possible_truncation)]
                    const LEVELS: u8 = (<$w>::BITS / 3) as u8;

                    let (chunks, _) = keys.as_chunks_mut::<{ 8 * $per_reg }>();
                    let done = chunks.len() * 8 * $per_reg;
                    for chunk in chunks {
                        // SAFETY: NEON is enabled by cfg; every load and
                        // store stays inside `chunk` or the 96 bytes of
                        // `ENCODE_TABLE`.
                        unsafe {
                            let low = vld1q_u8_x4($table.as_ptr());
                            let high = vld1q_u8_x2($table.as_ptr().add(64));
                            let sixty_four = vdupq_n_u8(64);
                            let octant = $dup(7);
                            let state_bits = $dup(0x78);
                            let mut code = [$dup(0); 8];
                            let mut acc = [$dup(0); 8];
                            let mut state = [$dup(0); 8];
                            for (r, c) in code.iter_mut().enumerate() {
                                *c = $ld(chunk.as_ptr().add($per_reg * r));
                            }
                            let mut level = LEVELS;
                            while level > 0 {
                                level -= 1;
                                let shift = $dup_s(-<$s>::from(3 * level));
                                for r in 0..8 {
                                    let index =
                                        $to_u8($or($and($shl(code[r], shift), octant), state[r]));
                                    // Entries 64..96 by `tbx` over the second
                                    // table: indices below 64 wrap past its
                                    // range and keep the `tbl` result.
                                    let entry = $from_u8(vqtbx2q_u8(
                                        vqtbl4q_u8(low, index),
                                        high,
                                        vsubq_u8(index, sixty_four),
                                    ));
                                    acc[r] = $or($shl_n::<3>(acc[r]), $and(entry, octant));
                                    state[r] = $and(entry, state_bits);
                                }
                            }
                            for (r, a) in acc.iter().enumerate() {
                                $st(chunk.as_mut_ptr().add($per_reg * r), *a);
                            }
                        }
                    }
                    done
                }
            };
        }

        kernel!(
            from_morton_u32,
            ENCODE_PADDED,
            u32,
            4,
            vdupq_n_u32,
            vdupq_n_s32,
            i32,
            vld1q_u32,
            vst1q_u32,
            vandq_u32,
            vorrq_u32,
            vshlq_u32,
            vshlq_n_u32,
            vreinterpretq_u8_u32,
            vreinterpretq_u32_u8
        );
        kernel!(
            from_morton_u64,
            ENCODE_PADDED,
            u64,
            2,
            vdupq_n_u64,
            vdupq_n_s64,
            i64,
            vld1q_u64,
            vst1q_u64,
            vandq_u64,
            vorrq_u64,
            vshlq_u64,
            vshlq_n_u64,
            vreinterpretq_u8_u64,
            vreinterpretq_u64_u8
        );
        kernel!(
            into_morton_u32,
            DECODE_PADDED,
            u32,
            4,
            vdupq_n_u32,
            vdupq_n_s32,
            i32,
            vld1q_u32,
            vst1q_u32,
            vandq_u32,
            vorrq_u32,
            vshlq_u32,
            vshlq_n_u32,
            vreinterpretq_u8_u32,
            vreinterpretq_u32_u8
        );
        kernel!(
            into_morton_u64,
            DECODE_PADDED,
            u64,
            2,
            vdupq_n_u64,
            vdupq_n_s64,
            i64,
            vld1q_u64,
            vst1q_u64,
            vandq_u64,
            vorrq_u64,
            vshlq_u64,
            vshlq_n_u64,
            vreinterpretq_u8_u64,
            vreinterpretq_u64_u8
        );
    }

    #[cfg(all(
        target_arch = "aarch64",
        target_feature = "neon",
        not(feature = "portable")
    ))]
    pub(super) use neon::{from_morton_u32, from_morton_u64, into_morton_u32, into_morton_u64};

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
    pub(super) const fn into_morton_u32(_: &mut [u32]) -> usize {
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
    pub(super) const fn into_morton_u64(_: &mut [u64]) -> usize {
        0
    }
}

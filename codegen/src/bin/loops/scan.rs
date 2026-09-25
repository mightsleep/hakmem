//! Bytes to masks, sixty-four at a time, on the widest registers the
//! machine has: a taste of the lanes hakmem 0.3 wants, written here first.
//!
//! hakmem's [`Lanes`] stop at sixteen, so sixty-four bytes are four
//! movemasks a class; a class costs one extraction a register, and the
//! extractions are the bottleneck (about one a cycle). [`U8x32`] halves
//! them with AVX2, [`U8x64`] makes it one with AVX-512BW. Same shape as
//! `Lanes` (`load`, `splat`, `cmp_eq`, `add`, `lut16_nibbles`,
//! `to_bitmask`), same soundness argument: a token's value proves the CPU
//! has the instructions, so a register made with one may use them, and
//! the kernels run inside a `#[target_feature]` function that everything
//! here inlines into. hakmem's own `U8x16` plugs into the same [`Bytes`]
//! trait, so every width runs the one kernel and `--speed` compares them.
//!
//! Two kernels. [`Level::newlines`] is the whole text's newline mask,
//! which is all the line walk needs: one compare and one extraction a
//! register. [`Level::classes`] classifies one block into every class,
//! and the parser asks for it only on the lines it reads.

use hakmem::isa::Isa;
use hakmem::lanes::Lanes;

/// The characters objdump's grammar turns on, one class each.
pub const CHARS: [u8; 6] = *b"\n\t#:<>";
pub const NL: usize = 0;
pub const TAB: usize = 1;
pub const HASH: usize = 2;
pub const COLON: usize = 3;
pub const LT: usize = 4;
pub const GT: usize = 5;

/// A byte is in class `k` iff `LO[low nibble] & HI[high nibble]` has bit
/// `7 - k`: the classes fill the byte from the top, so the movemask of
/// the lookup is class 0 and each `x + x` brings up the next. Built from
/// `CHARS`; `tables_are_exact` checks that no other byte lands in a class.
pub const LO: [u8; 16] = table(false);
pub const HI: [u8; 16] = table(true);

const fn table(high: bool) -> [u8; 16] {
    let mut t = [0u8; 16];
    let mut k = 0;
    while k < CHARS.len() {
        let c = CHARS[k];
        let i = if high { c >> 4 } else { c & 15 };
        t[i as usize] |= 0x80 >> k;
        k += 1;
    }
    t
}

/// One bit per byte of a block, per class.
pub type Masks = [u64; CHARS.len()];

/// A register of byte lanes, `Lanes` cut down to what the kernels use.
/// Carriers are made only from a token, which is what makes the
/// intrinsics in their methods sound.
pub trait Bytes: Copy {
    type Token: Copy;
    const LANES: usize;
    fn load(t: Self::Token, bytes: &[u8]) -> Self;
    fn splat(t: Self::Token, b: u8) -> Self;
    fn cmp_eq(self, o: Self) -> Self;
    fn add(self, o: Self) -> Self;
    fn lut16_nibbles(self, lo: [u8; 16], hi: [u8; 16]) -> Self;
    /// Bit `i` = the top bit of lane `i`.
    fn to_bitmask(self) -> u64;
}

/// The newline mask of every 64 bytes, the tail padded with zeros.
#[inline(always)]
fn newlines_in<B: Bytes>(t: B::Token, text: &[u8], out: &mut Vec<u64>) {
    out.reserve(text.len().div_ceil(64));
    let nl = B::splat(t, b'\n');
    let (blocks, tail) = text.as_chunks::<64>();
    for block in blocks {
        let mut m = 0;
        let mut i = 0;
        while i < 64 {
            m |= B::load(t, &block[i..i + B::LANES]).cmp_eq(nl).to_bitmask() << i;
            i += B::LANES;
        }
        out.push(m);
    }
    if !tail.is_empty() {
        out.push(
            tail.iter()
                .enumerate()
                .fold(0, |m, (i, &b)| m | u64::from(b == b'\n') << i),
        );
    }
}

/// Every class of one block: one lookup a register, then per class one
/// extraction a register and an add to bring up the next class. Plain
/// loops over a fixed array, no `array::map`: its closure stayed out of
/// line once, outside the target features, and every lookup was a call.
#[inline(always)]
fn classes_in<B: Bytes>(t: B::Token, block: &[u8; 64]) -> Masks {
    let n = 64 / B::LANES;
    let mut x = [B::splat(t, 0); 4];
    for (i, r) in x.iter_mut().enumerate().take(n) {
        *r = B::load(t, &block[i * B::LANES..(i + 1) * B::LANES]).lut16_nibbles(LO, HI);
    }
    let mut out = [0u64; CHARS.len()];
    for m in &mut out {
        for (i, r) in x.iter_mut().enumerate().take(n) {
            *m |= r.to_bitmask() << (i * B::LANES);
            *r = r.add(*r);
        }
    }
    out
}

#[inline(always)]
fn classes_all_in<B: Bytes>(t: B::Token, text: &[u8], out: &mut Vec<Masks>) {
    let (blocks, tail) = text.as_chunks::<64>();
    out.reserve(blocks.len() + 1);
    for block in blocks {
        out.push(classes_in::<B>(t, block));
    }
    if !tail.is_empty() {
        out.push(classes_in::<B>(t, &pad(tail)));
    }
}

pub fn pad(bytes: &[u8]) -> [u8; 64] {
    let mut b = [0u8; 64];
    b[..bytes.len()].copy_from_slice(bytes);
    b
}

// ---------------------------------------------------------------------------
// hakmem's sixteen lanes, at whatever level `dispatch!` picks.

#[derive(Clone, Copy)]
pub struct Sixteen<I: Isa>(I::U8x16);

impl<I: Isa> Bytes for Sixteen<I> {
    type Token = I;
    const LANES: usize = 16;
    #[inline(always)]
    fn load(t: I, bytes: &[u8]) -> Self {
        Self(I::U8x16::load(t, bytes))
    }
    #[inline(always)]
    fn splat(t: I, b: u8) -> Self {
        Self(I::U8x16::splat(t, b))
    }
    #[inline(always)]
    fn cmp_eq(self, o: Self) -> Self {
        Self(self.0.cmp_eq(o.0))
    }
    #[inline(always)]
    fn add(self, o: Self) -> Self {
        Self(self.0.add(o.0))
    }
    #[inline(always)]
    fn lut16_nibbles(self, lo: [u8; 16], hi: [u8; 16]) -> Self {
        Self(self.0.lut16_nibbles(lo, hi))
    }
    #[inline(always)]
    fn to_bitmask(self) -> u64 {
        u64::from(self.0.to_bitmask())
    }
}

// ---------------------------------------------------------------------------
// Thirty-two and sixty-four lanes: AVX2 and AVX-512BW.

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
mod x86 {
    use core::arch::x86_64::*;

    use super::Bytes;

    /// Proof that the CPU has AVX2: only `detect` makes one.
    #[derive(Clone, Copy)]
    pub struct Avx2(());
    impl Avx2 {
        pub fn detect() -> Option<Self> {
            is_x86_feature_detected!("avx2").then_some(Self(()))
        }
    }

    /// Proof that the CPU has AVX-512F and BW.
    #[derive(Clone, Copy)]
    pub struct Avx512(());
    impl Avx512 {
        pub fn detect() -> Option<Self> {
            (is_x86_feature_detected!("avx512f") && is_x86_feature_detected!("avx512bw"))
                .then_some(Self(()))
        }
    }

    #[derive(Clone, Copy)]
    pub struct U8x32(__m256i);

    // SAFETY, every block below: a `U8x32` exists only from an `Avx2`,
    // which exists only on a CPU with AVX2, and the kernels that use it
    // are compiled with the feature on.
    impl Bytes for U8x32 {
        type Token = Avx2;
        const LANES: usize = 32;
        #[inline(always)]
        fn load(_: Avx2, bytes: &[u8]) -> Self {
            assert!(bytes.len() == 32);
            Self(unsafe { _mm256_loadu_si256(bytes.as_ptr().cast()) })
        }
        #[inline(always)]
        fn splat(_: Avx2, b: u8) -> Self {
            Self(unsafe { _mm256_set1_epi8(b.cast_signed()) })
        }
        #[inline(always)]
        fn cmp_eq(self, o: Self) -> Self {
            Self(unsafe { _mm256_cmpeq_epi8(self.0, o.0) })
        }
        #[inline(always)]
        fn add(self, o: Self) -> Self {
            Self(unsafe { _mm256_add_epi8(self.0, o.0) })
        }
        #[inline(always)]
        fn lut16_nibbles(self, lo: [u8; 16], hi: [u8; 16]) -> Self {
            unsafe {
                let lo = _mm256_broadcastsi128_si256(_mm_loadu_si128(lo.as_ptr().cast()));
                let hi = _mm256_broadcastsi128_si256(_mm_loadu_si128(hi.as_ptr().cast()));
                let low = _mm256_set1_epi8(0x0F);
                let l = _mm256_and_si256(self.0, low);
                let h = _mm256_and_si256(_mm256_srli_epi16(self.0, 4), low);
                Self(_mm256_and_si256(
                    _mm256_shuffle_epi8(lo, l),
                    _mm256_shuffle_epi8(hi, h),
                ))
            }
        }
        #[inline(always)]
        fn to_bitmask(self) -> u64 {
            u64::from(unsafe { _mm256_movemask_epi8(self.0) }.cast_unsigned())
        }
    }

    #[derive(Clone, Copy)]
    pub struct U8x64(__m512i);

    // SAFETY, every block below: as for `U8x32`, with `Avx512`.
    impl Bytes for U8x64 {
        type Token = Avx512;
        const LANES: usize = 64;
        #[inline(always)]
        fn load(_: Avx512, bytes: &[u8]) -> Self {
            assert!(bytes.len() == 64);
            Self(unsafe { _mm512_loadu_si512(bytes.as_ptr().cast()) })
        }
        #[inline(always)]
        fn splat(_: Avx512, b: u8) -> Self {
            Self(unsafe { _mm512_set1_epi8(b.cast_signed()) })
        }
        #[inline(always)]
        fn cmp_eq(self, o: Self) -> Self {
            Self(unsafe { _mm512_movm_epi8(_mm512_cmpeq_epi8_mask(self.0, o.0)) })
        }
        #[inline(always)]
        fn add(self, o: Self) -> Self {
            Self(unsafe { _mm512_add_epi8(self.0, o.0) })
        }
        #[inline(always)]
        fn lut16_nibbles(self, lo: [u8; 16], hi: [u8; 16]) -> Self {
            unsafe {
                let lo = _mm512_broadcast_i32x4(_mm_loadu_si128(lo.as_ptr().cast()));
                let hi = _mm512_broadcast_i32x4(_mm_loadu_si128(hi.as_ptr().cast()));
                let low = _mm512_set1_epi8(0x0F);
                let l = _mm512_and_si512(self.0, low);
                let h = _mm512_and_si512(_mm512_srli_epi16(self.0, 4), low);
                Self(_mm512_and_si512(
                    _mm512_shuffle_epi8(lo, l),
                    _mm512_shuffle_epi8(hi, h),
                ))
            }
        }
        #[inline(always)]
        fn to_bitmask(self) -> u64 {
            unsafe { _mm512_movepi8_mask(self.0) }
        }
    }

    // The kernels, compiled with each width's features. Callers hold the
    // token, which is the proof `unsafe` asks for.

    #[target_feature(enable = "avx2")]
    pub fn newlines_avx2(t: Avx2, text: &[u8], out: &mut Vec<u64>) {
        super::newlines_in::<U8x32>(t, text, out);
    }
    #[target_feature(enable = "avx2")]
    pub fn classes_avx2(t: Avx2, block: &[u8; 64]) -> super::Masks {
        super::classes_in::<U8x32>(t, block)
    }
    #[target_feature(enable = "avx2")]
    pub fn classes_all_avx2(t: Avx2, text: &[u8], out: &mut Vec<super::Masks>) {
        super::classes_all_in::<U8x32>(t, text, out);
    }
    #[target_feature(enable = "avx512f,avx512bw")]
    pub fn newlines_avx512(t: Avx512, text: &[u8], out: &mut Vec<u64>) {
        super::newlines_in::<U8x64>(t, text, out);
    }
    #[target_feature(enable = "avx512f,avx512bw")]
    pub fn classes_avx512(t: Avx512, block: &[u8; 64]) -> super::Masks {
        super::classes_in::<U8x64>(t, block)
    }
    #[target_feature(enable = "avx512f,avx512bw")]
    pub fn classes_all_avx512(t: Avx512, text: &[u8], out: &mut Vec<super::Masks>) {
        super::classes_all_in::<U8x64>(t, text, out);
    }
}

#[cfg(target_arch = "x86_64")]
pub use x86::{Avx2, Avx512};

/// Which registers classify: the widest the CPU has, or any of them for
/// `--check` and `--speed`.
#[derive(Clone, Copy, Debug)]
pub enum Level {
    #[cfg(target_arch = "x86_64")]
    Avx512(#[allow(dead_code)] Avx512),
    #[cfg(target_arch = "x86_64")]
    Avx2(#[allow(dead_code)] Avx2),
    /// hakmem's sixteen lanes at one of its levels.
    Sixteen(hakmem::isa::Level),
}

impl core::fmt::Debug for Avx512 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("64 lanes")
    }
}
impl core::fmt::Debug for Avx2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("32 lanes")
    }
}

impl Level {
    pub fn detect() -> Self {
        Self::all()
            .into_iter()
            .next()
            .unwrap_or(Self::Sixteen(hakmem::isa::detect()))
    }

    /// Widest first.
    pub fn all() -> Vec<Self> {
        let mut v = Vec::new();
        #[cfg(target_arch = "x86_64")]
        {
            v.extend(Avx512::detect().map(Self::Avx512));
            v.extend(Avx2::detect().map(Self::Avx2));
        }
        let mut sixteen: Vec<_> = hakmem::isa::available().collect();
        sixteen.reverse();
        v.extend(sixteen.into_iter().map(Self::Sixteen));
        v
    }

    pub fn newlines(self, text: &[u8]) -> Vec<u64> {
        let mut out = Vec::new();
        self.newlines_into(text, &mut out);
        out
    }

    #[allow(unsafe_code)]
    pub fn newlines_into(self, text: &[u8], out: &mut Vec<u64>) {
        match self {
            // SAFETY: the token is the proof of the feature.
            #[cfg(target_arch = "x86_64")]
            Self::Avx512(t) => unsafe { x86::newlines_avx512(t, text, out) },
            #[cfg(target_arch = "x86_64")]
            Self::Avx2(t) => unsafe { x86::newlines_avx2(t, text, out) },
            Self::Sixteen(l) => hakmem::dispatch!(l, |cpu| sixteen_newlines(cpu, text, out)),
        }
    }

    #[allow(unsafe_code)]
    pub fn classes(self, block: &[u8; 64]) -> Masks {
        match self {
            // SAFETY: as above.
            #[cfg(target_arch = "x86_64")]
            Self::Avx512(t) => unsafe { x86::classes_avx512(t, block) },
            #[cfg(target_arch = "x86_64")]
            Self::Avx2(t) => unsafe { x86::classes_avx2(t, block) },
            Self::Sixteen(l) => hakmem::dispatch!(l, |cpu| sixteen_classes(cpu, block)),
        }
    }

    /// Every class of every block: the bulk pass the tool used to make,
    /// kept for `--speed` and `--check`.
    pub fn classes_all(self, text: &[u8]) -> Vec<Masks> {
        let mut out = Vec::new();
        self.classes_all_into(text, &mut out);
        out
    }

    #[allow(unsafe_code)]
    pub fn classes_all_into(self, text: &[u8], out: &mut Vec<Masks>) {
        match self {
            // SAFETY: as above.
            #[cfg(target_arch = "x86_64")]
            Self::Avx512(t) => unsafe { x86::classes_all_avx512(t, text, out) },
            #[cfg(target_arch = "x86_64")]
            Self::Avx2(t) => unsafe { x86::classes_all_avx2(t, text, out) },
            Self::Sixteen(l) => hakmem::dispatch!(l, |cpu| sixteen_classes_all(cpu, text, out)),
        }
    }
}

#[inline(always)]
fn sixteen_newlines<I: Isa>(cpu: I, text: &[u8], out: &mut Vec<u64>) {
    newlines_in::<Sixteen<I>>(cpu, text, out);
}
#[inline(always)]
fn sixteen_classes<I: Isa>(cpu: I, block: &[u8; 64]) -> Masks {
    classes_in::<Sixteen<I>>(cpu, block)
}
#[inline(always)]
fn sixteen_classes_all<I: Isa>(cpu: I, text: &[u8], out: &mut Vec<Masks>) {
    classes_all_in::<Sixteen<I>>(cpu, text, out);
}

/// A byte at a time: what `--check` holds every level to.
pub fn classes_naive(text: &[u8]) -> Vec<Masks> {
    let mut out = vec![[0u64; CHARS.len()]; text.len().div_ceil(64)];
    for (i, &b) in text.iter().enumerate() {
        for (k, &c) in CHARS.iter().enumerate() {
            if b == c {
                out[i / 64][k] |= 1 << (i % 64);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tables_are_exact() {
        for b in 0..=255u8 {
            let class = LO[usize::from(b & 15)] & HI[usize::from(b >> 4)];
            let want = CHARS.iter().position(|&c| c == b).map_or(0, |k| 0x80 >> k);
            assert_eq!(class, want, "byte {b:#04x}");
        }
    }

    #[test]
    fn every_level_matches_the_byte_scan() {
        let mut s = 0x9E37_79B9_7F4A_7C15u64;
        let alphabet = b"\n\t #:<>0123456789abcdefxyz(%),.$*+_{}[]\x00\xff\x8a";
        let text: Vec<u8> = (0..10_000)
            .map(|_| {
                s ^= s << 13;
                s ^= s >> 7;
                s ^= s << 17;
                alphabet[(s % alphabet.len() as u64) as usize]
            })
            .collect();
        for level in Level::all() {
            for len in [0, 1, 15, 16, 31, 32, 63, 64, 65, 127, 1000, 10_000] {
                let t = &text[..len];
                let want = classes_naive(t);
                assert!(level.classes_all(t) == want, "{level:?} classes, len {len}");
                let nl: Vec<u64> = want.iter().map(|m| m[NL]).collect();
                assert!(level.newlines(t) == nl, "{level:?} newlines, len {len}");
            }
        }
    }
}

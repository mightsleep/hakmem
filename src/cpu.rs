//! Run-time detection of the x86 features the batch kernels use, for
//! builds that did not turn them on at compile time. `no_std`: CPUID and
//! XGETBV are in `core::arch`, the cache is one atomic.
//!
//! Each question first asks the compiler: with the feature enabled at
//! compile time the answer is a constant `true` and the detection is
//! never built. Otherwise CPUID says what the processor has and XCR0
//! what the operating system saves on a context switch; a processor with
//! AVX-512 under an OS that does not save `zmm` has no AVX-512. The
//! answer is computed once and cached. Under Miri, which has no CPUID,
//! only the compile-time answer counts.
//!
//! Only whole batch kernels are chosen this way, once a call over
//! thousands of keys. The per-word combinators keep their compile-time
//! selection (design notes section 2.2).

#![allow(unsafe_code)]
// `unreachable_pub` wants `pub(super)` here and this lint wants `pub`;
// the rustc lint is the one the crate chose.
#![allow(clippy::redundant_pub_crate)]
// With the `portable` feature the batch kernels do not ask, the tokens do.
#![cfg_attr(feature = "portable", allow(dead_code))]

use core::sync::atomic::{AtomicU32, Ordering};

const AVX2: u32 = 1 << 0;
/// AVX-512 F, BW and VBMI, and an OS that saves `zmm`.
const AVX512VBMI: u32 = 1 << 1;
/// GFNI on top of [`AVX512VBMI`], for its 512-bit form.
const AVX512GFNI: u32 = 1 << 2;
/// x86-64-v3 and PCLMULQDQ, with the OS saving `ymm`: the `X86V3` token.
const X86V3: u32 = 1 << 3;
/// Set once the detection has run.
const KNOWN: u32 = 1 << 31;

static CACHE: AtomicU32 = AtomicU32::new(0);

/// AVX2 with the OS saving `ymm`.
#[inline]
pub(super) fn avx2() -> bool {
    cfg!(target_feature = "avx2") || detected(AVX2)
}

/// AVX-512 F, BW and VBMI with the OS saving `zmm`.
#[inline]
pub(super) fn avx512vbmi() -> bool {
    cfg!(all(
        target_feature = "avx512f",
        target_feature = "avx512bw",
        target_feature = "avx512vbmi"
    )) || detected(AVX512VBMI)
}

/// Every feature of `isa::X86V3::FEATURES`, and the OS saving `ymm`.
#[inline]
pub(super) fn x86v3() -> bool {
    cfg!(all(
        target_feature = "sse3",
        target_feature = "ssse3",
        target_feature = "sse4.1",
        target_feature = "sse4.2",
        target_feature = "popcnt",
        target_feature = "cmpxchg16b",
        target_feature = "avx",
        target_feature = "avx2",
        target_feature = "bmi1",
        target_feature = "bmi2",
        target_feature = "fma",
        target_feature = "lzcnt",
        target_feature = "movbe",
        target_feature = "f16c",
        target_feature = "xsave",
        target_feature = "pclmulqdq"
    )) || detected(X86V3)
}

/// [`avx512vbmi`] and GFNI.
#[inline]
pub(super) fn avx512vbmi_gfni() -> bool {
    cfg!(all(
        target_feature = "avx512f",
        target_feature = "avx512bw",
        target_feature = "avx512vbmi",
        target_feature = "gfni"
    )) || detected(AVX512GFNI)
}

#[inline]
fn detected(feature: u32) -> bool {
    let mut bits = CACHE.load(Ordering::Relaxed);
    if bits & KNOWN == 0 {
        bits = detect() | KNOWN;
        // Racing threads compute the same bits; any store wins.
        CACHE.store(bits, Ordering::Relaxed);
    }
    bits & feature != 0
}

#[cfg(miri)]
const fn detect() -> u32 {
    0
}

#[cfg(not(miri))]
#[cold]
// `__cpuid` is safe on recent compilers and `unsafe` on the MSRV.
#[allow(unused_unsafe)]
fn detect() -> u32 {
    use core::arch::x86_64::{__cpuid, __cpuid_count, _xgetbv};

    // SAFETY: CPUID exists on every x86_64 processor.
    let max_leaf = unsafe { __cpuid(0) }.eax;
    if max_leaf < 7 {
        return 0;
    }
    // SAFETY: as above; leaves 1 and 7 exist, checked just now.
    let (leaf1, leaf7) = unsafe { (__cpuid(1), __cpuid_count(7, 0)) };
    // OSXSAVE: the OS enabled XGETBV and manages the extended state.
    if leaf1.ecx & (1 << 27) == 0 {
        return 0;
    }
    // SAFETY: OSXSAVE is set, so XGETBV exists and XCR0 is readable.
    let xcr0 = unsafe { _xgetbv(0) };
    // SSE and AVX state; then opmask, the upper halves of zmm0..15 and
    // zmm16..31.
    let ymm = xcr0 & 0b110 == 0b110;
    let zmm = ymm && xcr0 & 0b1110_0000 == 0b1110_0000;
    let bit = |word: u32, n: u32| word & (1 << n) != 0;
    let mut out = 0;
    if ymm && bit(leaf7.ebx, 5) {
        out |= AVX2;
    }
    // x86-64-v3: leaf 1 ECX SSE3 0, PCLMULQDQ 1, SSSE3 9, FMA 12,
    // CMPXCHG16B 13, SSE4.1 19, SSE4.2 20, MOVBE 22, POPCNT 23, XSAVE 26,
    // AVX 28, F16C 29; leaf 7 EBX BMI1 3, AVX2 5, BMI2 8; LZCNT is bit 5
    // of ECX in leaf 0x8000_0001.
    let v3_leaf1 = [0, 1, 9, 12, 13, 19, 20, 22, 23, 26, 28, 29];
    // SAFETY: CPUID exists on every x86_64 processor; the extended leaf
    // is asked for only if the processor reports it.
    let lzcnt = unsafe { __cpuid(0x8000_0000) }.eax >= 0x8000_0001
        && bit(unsafe { __cpuid(0x8000_0001) }.ecx, 5);
    if ymm
        && v3_leaf1.iter().all(|&n| bit(leaf1.ecx, n))
        && [3, 5, 8].iter().all(|&n| bit(leaf7.ebx, n))
        && lzcnt
    {
        out |= X86V3;
    }
    // F is EBX bit 16, BW bit 30, VBMI is ECX bit 1.
    if zmm && bit(leaf7.ebx, 16) && bit(leaf7.ebx, 30) && bit(leaf7.ecx, 1) {
        out |= AVX512VBMI;
        // GFNI is ECX bit 8.
        if bit(leaf7.ecx, 8) {
            out |= AVX512GFNI;
        }
    }
    out
}

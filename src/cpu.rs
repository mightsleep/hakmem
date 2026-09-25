//! Run-time detection of the x86 levels [`crate::isa`] hands out as
//! tokens, and the batch kernels run on. `no_std`: CPUID and XGETBV are in
//! `core::arch`, the cache is one atomic.
//!
//! Each question first asks the compiler: with the features enabled at
//! compile time the answer is a constant `true` and the detection is
//! never built. Otherwise CPUID says what the processor has and XCR0
//! what the operating system saves on a context switch; a processor with
//! AVX-512 under an OS that does not save `zmm` has no AVX-512. The
//! answer is computed once and cached. Under Miri, which has no CPUID,
//! only the compile-time answer counts.

#![allow(unsafe_code)]
// `unreachable_pub` wants `pub(super)` here and this lint wants `pub`;
// the rustc lint is the one the crate chose.
#![allow(clippy::redundant_pub_crate)]

use core::sync::atomic::{AtomicU32, Ordering};

/// x86-64-v3 and PCLMULQDQ, with the OS saving `ymm`: `X86V3`.
pub(super) const X86V3: u32 = 1 << 0;
/// X86V3, x86-64-v4, VBMI and GFNI, with the OS saving `zmm`: `X86V4`.
pub(super) const X86V4: u32 = 1 << 1;
/// x86-64-v2: `X86V2`. The OS saves SSE state on every `x86_64`.
pub(super) const X86V2: u32 = 1 << 2;
/// Set once the detection has run.
const KNOWN: u32 = 1 << 31;

static CACHE: AtomicU32 = AtomicU32::new(0);

/// Every feature of `isa::X86V2::FEATURES`.
#[inline]
pub(super) fn x86v2() -> bool {
    x86v2_in_build() || detected(X86V2)
}

/// Every feature of `isa::X86V3::FEATURES`, and the OS saving `ymm`.
#[inline]
pub(super) fn x86v3() -> bool {
    x86v3_in_build() || detected(X86V3)
}

/// Every feature of `isa::X86V4::FEATURES`, and the OS saving `zmm`.
#[inline]
pub(super) fn x86v4() -> bool {
    x86v4_in_build() || detected(X86V4)
}

/// The build itself enables every feature of `isa::X86V2::FEATURES`.
pub(super) const fn x86v2_in_build() -> bool {
    cfg!(all(
        target_feature = "sse3",
        target_feature = "ssse3",
        target_feature = "sse4.1",
        target_feature = "sse4.2",
        target_feature = "popcnt",
        target_feature = "cmpxchg16b"
    ))
}

/// The build itself enables every feature of `isa::X86V3::FEATURES`.
pub(super) const fn x86v3_in_build() -> bool {
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
    ))
}

/// The build itself enables every feature of `isa::X86V4::FEATURES`.
pub(super) const fn x86v4_in_build() -> bool {
    x86v3_in_build()
        && cfg!(all(
            target_feature = "avx512f",
            target_feature = "avx512dq",
            target_feature = "avx512cd",
            target_feature = "avx512bw",
            target_feature = "avx512vl",
            target_feature = "avx512vbmi",
            target_feature = "gfni"
        ))
}

/// The levels the CPU has, as [`X86V2`], [`X86V3`] and [`X86V4`] bits: one load of
/// the cache, and the detection the first time.
#[inline]
pub(super) fn levels() -> u32 {
    let mut bits = CACHE.load(Ordering::Relaxed);
    if bits & KNOWN == 0 {
        bits = detect() | KNOWN;
        // Racing threads compute the same bits; any store wins.
        CACHE.store(bits, Ordering::Relaxed);
    }
    bits
}

#[inline]
fn detected(feature: u32) -> bool {
    levels() & feature != 0
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
    if max_leaf < 1 {
        return 0;
    }
    // SAFETY: as above; leaf 1 exists, checked just now.
    let leaf1 = unsafe { __cpuid(1) };
    let bit = |word: u32, n: u32| word & (1 << n) != 0;
    let mut out = 0;
    // x86-64-v2: leaf 1 ECX SSE3 0, SSSE3 9, CMPXCHG16B 13, SSE4.1 19,
    // SSE4.2 20, POPCNT 23. SSE state needs no XCR0, which Nehalem, a
    // v2 CPU, would not have.
    if [0, 9, 13, 19, 20, 23].iter().all(|&n| bit(leaf1.ecx, n)) {
        out |= X86V2;
    }
    // OSXSAVE: the OS enabled XGETBV and manages the extended state.
    if max_leaf < 7 || !bit(leaf1.ecx, 27) {
        return out;
    }
    // SAFETY: as above; leaf 7 exists, checked just now.
    let leaf7 = unsafe { __cpuid_count(7, 0) };
    // SAFETY: OSXSAVE is set, so XGETBV exists and XCR0 is readable.
    let xcr0 = unsafe { _xgetbv(0) };
    // SSE and AVX state; then opmask, the upper halves of zmm0..15 and
    // zmm16..31.
    let ymm = xcr0 & 0b110 == 0b110;
    let zmm = ymm && xcr0 & 0b1110_0000 == 0b1110_0000;
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
    // x86-64-v4 on top: leaf 7 EBX AVX512F 16, DQ 17, CD 28, BW 30, VL 31;
    // ECX VBMI 1, GFNI 8.
    if out & X86V3 != 0
        && zmm
        && [16, 17, 28, 30, 31].iter().all(|&n| bit(leaf7.ebx, n))
        && [1, 8].iter().all(|&n| bit(leaf7.ecx, n))
    {
        out |= X86V4;
    }
    out
}

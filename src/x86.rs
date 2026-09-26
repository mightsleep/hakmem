//! The leaves: one instruction each, as safe `#[target_feature]`
//! functions tagged with exactly the features they use.
//!
//! For code under somebody else's dispatch: a `multiversion` clone, a
//! `pulp` or `fearless_simd` kernel, a hand-written CPUID check. LLVM
//! inlines one of these only into a caller whose features include its
//! own, so a leaf tagged `bmi2` becomes a bare PDEP inside anything that
//! has BMI2, whatever else it has or lacks. Called from a function
//! without the feature it needs `unsafe`, which is the point: the
//! compiler says where the instruction would have been a call.
//!
//! ```
//! # #[cfg(target_arch = "x86_64")] {
//! use hakmem::x86::bmi2;
//!
//! #[target_feature(enable = "bmi2")]
//! fn gather(xs: &[u64], mask: u64) -> u64 {
//!     xs.iter().fold(0, |a, &x| a ^ bmi2::pext_u64(x, mask))
//! }
//! if std::is_x86_feature_detected!("bmi2") {
//!     // SAFETY: BMI2 detected just now.
//!     assert_eq!(unsafe { gather(&[0b1011], 0b0110) }, 0b01);
//! }
//! # }
//! ```
//!
//! Tokens ([`crate::isa`]) and [`crate::isa::Native`] call these, so a
//! path has one implementation whichever door it is reached through.

#![allow(unsafe_code)]
// The intrinsics are safe inside `#[target_feature]` on recent compilers
// and `unsafe` on the MSRV.
#![allow(unused_unsafe)]
// Safe `#[target_feature]` functions: one precondition for all of them,
// the CPU has the feature, and it is the module's first paragraph.
#![allow(clippy::missing_safety_doc)]

/// PEXT and PDEP.
pub mod bmi2 {
    use core::arch::x86_64::{_pdep_u32, _pdep_u64, _pext_u32, _pext_u64};

    /// [`Word::pext`](crate::Word::pext) on `u32`: one PEXT.
    #[target_feature(enable = "bmi2")]
    #[inline]
    #[must_use]
    pub fn pext_u32(x: u32, mask: u32) -> u32 {
        // SAFETY: bmi2 is enabled on this function.
        unsafe { _pext_u32(x, mask) }
    }

    /// [`Word::pext`](crate::Word::pext) on `u64`: one PEXT.
    #[target_feature(enable = "bmi2")]
    #[inline]
    #[must_use]
    pub fn pext_u64(x: u64, mask: u64) -> u64 {
        // SAFETY: bmi2 is enabled on this function.
        unsafe { _pext_u64(x, mask) }
    }

    /// [`Word::pdep`](crate::Word::pdep) on `u32`: one PDEP.
    #[target_feature(enable = "bmi2")]
    #[inline]
    #[must_use]
    pub fn pdep_u32(x: u32, mask: u32) -> u32 {
        // SAFETY: bmi2 is enabled on this function.
        unsafe { _pdep_u32(x, mask) }
    }

    /// [`Word::pdep`](crate::Word::pdep) on `u64`: one PDEP.
    #[target_feature(enable = "bmi2")]
    #[inline]
    #[must_use]
    pub fn pdep_u64(x: u64, mask: u64) -> u64 {
        // SAFETY: bmi2 is enabled on this function.
        unsafe { _pdep_u64(x, mask) }
    }

    /// [`Word::select_lowest`](crate::Word::select_lowest) on `u64`:
    /// `trailing_zeros(pdep(1 << k, x))`, 64 when there is no `k`-th bit
    /// (Pandey, Bender and Johnson, 2017).
    #[target_feature(enable = "bmi2")]
    #[inline]
    #[must_use]
    pub fn select_u64(x: u64, k: u32) -> u32 {
        // Past the last set bit PDEP deposits nothing and TZCNT says 64,
        // as long as `1 << k` is not asked to exist first.
        pdep_u64(1u64.checked_shl(k).unwrap_or(0), x).trailing_zeros()
    }
}

/// Carry-less multiplication as a scan. Not on soft-float targets
/// (kernels, `x86_64-unknown-none`): an `xmm` there is somebody else's.
#[cfg(target_feature = "sse2")]
pub mod pclmulqdq {
    use core::arch::x86_64::{
        __m128i, _mm_clmulepi64_si128, _mm_cvtsi128_si64, _mm_set_epi64x, _mm_unpackhi_epi64,
    };

    /// `x` times all ones, carry-less: the low half is the prefix parity,
    /// the high half every right shift combined by XOR.
    #[target_feature(enable = "pclmulqdq")]
    #[inline]
    fn times_ones(x: u64) -> __m128i {
        // SAFETY: pclmulqdq is enabled on this function, and SSE2 is the
        // x86_64 baseline.
        unsafe {
            let a = _mm_set_epi64x(0, x.cast_signed());
            _mm_clmulepi64_si128(a, _mm_set_epi64x(0, -1), 0)
        }
    }

    /// [`Word::xor_scan`](crate::Word::xor_scan) on `u64`: one PCLMULQDQ
    /// by all ones.
    #[target_feature(enable = "pclmulqdq")]
    #[inline]
    #[must_use]
    pub fn xor_scan_u64(x: u64) -> u64 {
        // SAFETY: SSE2 moves, the x86_64 baseline.
        unsafe { _mm_cvtsi128_si64(times_ones(x)).cast_unsigned() }
    }

    /// [`Word::xor_scan_down`](crate::Word::xor_scan_down) on `u64`: the
    /// high half of the same product is the exclusive suffix parity, one
    /// XOR from the inclusive.
    #[target_feature(enable = "pclmulqdq")]
    #[inline]
    #[must_use]
    pub fn xor_scan_down_u64(x: u64) -> u64 {
        let p = times_ones(x);
        // SAFETY: SSE2 moves, the x86_64 baseline.
        let exclusive = unsafe { _mm_cvtsi128_si64(_mm_unpackhi_epi64(p, p)) }.cast_unsigned();
        exclusive ^ x
    }
}

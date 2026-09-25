//! Instruction sets as values: pick one where a hot loop starts, pass it
//! down, and the primitives inside compile to its instructions.
//!
//! A token is a zero-sized proof that the CPU has a set of features.
//! [`Portable`] and [`Native`] exist everywhere; `X86V3` (x86) only comes out
//! of [`detect`] (or an `unsafe` promise). [`Isa::run`] runs a closure
//! compiled with the token's features, and [`dispatch!`](crate::dispatch)
//! does the detection and the `run` in one go:
//!
//! ```
//! use hakmem::isa::Isa;
//!
//! fn gather<I: Isa>(cpu: I, xs: &[u64], mask: u64) -> u64 {
//!     xs.iter().fold(0, |a, &x| a ^ cpu.pext(x, mask))
//! }
//!
//! let xs = [0b1011u64, 0b0110];
//! let r = hakmem::dispatch!(|cpu| gather(cpu, &xs, 0b0110));
//! assert_eq!(r, 0b01 ^ 0b11);
//! ```
//!
//! Detection happens once per `dispatch!`, so it belongs above the loop,
//! not in it. Everything the closure calls has to inline into it: a
//! helper kept out of line (`#[inline(never)]`, or too big for LLVM's
//! taste) is compiled without the features, and each primitive in it
//! becomes a call. Design notes, section 11.

// `inline(always)` on everything a token does: a primitive that stays out
// of line is compiled without the features, which is the whole bug.
#![allow(clippy::inline_always)]

use core::fmt::Debug;

use crate::word::{
    compress_broadword, expand_broadword, select_broadword64, xor_smear, xor_smear_down,
};

mod sealed {
    pub trait Sealed {}
}

/// A set of CPU features, as a zero-sized value that proves the CPU has
/// them. Sealed: the levels are hakmem's.
///
/// The per-width methods are the hooks [`Word`](crate::Word) routes
/// through; the generic ones (`pext`, `select`, ...) are what code reads.
pub trait Isa:
    Copy + Eq + core::hash::Hash + Send + Sync + Debug + 'static + sealed::Sealed
{
    /// Sixteen byte lanes with this token's instructions: a PSHUFB register
    /// on x86 with SSSE3, NEON on aarch64, two SWAR words elsewhere.
    type U8x16: crate::lanes::Lanes<Isa = Self, Bitmask = u16>;

    /// `f`, compiled with this token's target features. Everything `f`
    /// inlines gets them too.
    fn run<R>(self, f: impl FnOnce(Self) -> R) -> R;

    #[doc(hidden)]
    fn pext_u32(self, x: u32, mask: u32) -> u32;
    #[doc(hidden)]
    fn pext_u64(self, x: u64, mask: u64) -> u64;
    #[doc(hidden)]
    fn pdep_u32(self, x: u32, mask: u32) -> u32;
    #[doc(hidden)]
    fn pdep_u64(self, x: u64, mask: u64) -> u64;
    #[doc(hidden)]
    fn select_u64(self, x: u64, k: u32) -> u32;
    #[doc(hidden)]
    fn xor_scan_u64(self, x: u64) -> u64;
    #[doc(hidden)]
    fn xor_scan_down_u64(self, x: u64) -> u64;

    /// [`Word::pext`](crate::Word::pext) with this token's instructions.
    #[inline(always)]
    fn pext<W: crate::Word>(self, x: W, mask: W) -> W {
        x.pext_in(mask, self)
    }
    /// [`Word::pdep`](crate::Word::pdep) with this token's instructions.
    #[inline(always)]
    fn pdep<W: crate::Word>(self, x: W, mask: W) -> W {
        x.pdep_in(mask, self)
    }
    /// [`Bits::select`](crate::Bits::select) with this token's instructions.
    #[inline(always)]
    fn select<W: crate::Word>(self, x: W, k: u32) -> Option<u32> {
        (k < x.count_ones()).then(|| x.select_lowest_in(k, self))
    }
    /// [`Word::xor_scan`](crate::Word::xor_scan) with this token's instructions.
    #[inline(always)]
    fn xor_scan<W: crate::Word>(self, x: W) -> W {
        x.xor_scan_in(self)
    }
    /// [`Word::xor_scan_down`](crate::Word::xor_scan_down) with this
    /// token's instructions.
    #[inline(always)]
    fn xor_scan_down<W: crate::Word>(self, x: W) -> W {
        x.xor_scan_down_in(self)
    }
}

/// No features: the broadword definitions, everywhere.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Portable;

impl sealed::Sealed for Portable {}

impl Isa for Portable {
    type U8x16 = crate::lanes::Swar16<Self>;
    #[inline(always)]
    fn run<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
    #[inline(always)]
    fn pext_u32(self, x: u32, mask: u32) -> u32 {
        compress_broadword(x, mask)
    }
    #[inline(always)]
    fn pext_u64(self, x: u64, mask: u64) -> u64 {
        compress_broadword(x, mask)
    }
    #[inline(always)]
    fn pdep_u32(self, x: u32, mask: u32) -> u32 {
        expand_broadword(x, mask)
    }
    #[inline(always)]
    fn pdep_u64(self, x: u64, mask: u64) -> u64 {
        expand_broadword(x, mask)
    }
    #[inline(always)]
    fn select_u64(self, x: u64, k: u32) -> u32 {
        select_broadword64(x, k)
    }
    #[inline(always)]
    fn xor_scan_u64(self, x: u64) -> u64 {
        xor_smear(x)
    }
    #[inline(always)]
    fn xor_scan_down_u64(self, x: u64) -> u64 {
        xor_smear_down(x)
    }
}

/// What the build proves, primitive by primitive.
///
/// PEXT where `target_feature = "bmi2"`, PCLMULQDQ where `"pclmulqdq"`,
/// broadword elsewhere or with the `portable` feature. The methods on
/// words (`x.select(k)`) are this token. `run` changes nothing.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Native;

impl sealed::Sealed for Native {}

/// The build has BMI2, and the hardware path is not turned off.
macro_rules! native_bmi2 {
    ($hw:expr, $sw:expr) => {{
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "bmi2",
            not(feature = "portable")
        ))]
        {
            // SAFETY: the build enables BMI2 for every function (the cfg
            // on this arm); rustc still wants it said.
            unsafe { $hw }
        }
        #[cfg(not(all(
            target_arch = "x86_64",
            target_feature = "bmi2",
            not(feature = "portable")
        )))]
        {
            $sw
        }
    }};
}

/// The build has PCLMULQDQ, and the hardware path is not turned off.
macro_rules! native_clmul {
    ($hw:expr, $sw:expr) => {{
        #[cfg(all(
            target_arch = "x86_64",
            target_feature = "pclmulqdq",
            not(feature = "portable")
        ))]
        {
            // SAFETY: the build enables PCLMULQDQ for every function (the
            // cfg on this arm); rustc still wants it said.
            unsafe { $hw }
        }
        #[cfg(not(all(
            target_arch = "x86_64",
            target_feature = "pclmulqdq",
            not(feature = "portable")
        )))]
        {
            $sw
        }
    }};
}

// A build-wide target feature does not make a `#[target_feature]` call
// safe: rustc wants the feature on the calling function, which a trait
// method cannot carry. The `cfg` is the proof instead.
#[allow(unsafe_code)]
impl Isa for Native {
    type U8x16 = crate::lanes::NativeU8x16;
    #[inline(always)]
    fn run<R>(self, f: impl FnOnce(Self) -> R) -> R {
        f(self)
    }
    #[inline(always)]
    fn pext_u32(self, x: u32, mask: u32) -> u32 {
        native_bmi2!(
            crate::x86::bmi2::pext_u32(x, mask),
            compress_broadword(x, mask)
        )
    }
    #[inline(always)]
    fn pext_u64(self, x: u64, mask: u64) -> u64 {
        native_bmi2!(
            crate::x86::bmi2::pext_u64(x, mask),
            compress_broadword(x, mask)
        )
    }
    #[inline(always)]
    fn pdep_u32(self, x: u32, mask: u32) -> u32 {
        native_bmi2!(
            crate::x86::bmi2::pdep_u32(x, mask),
            expand_broadword(x, mask)
        )
    }
    #[inline(always)]
    fn pdep_u64(self, x: u64, mask: u64) -> u64 {
        native_bmi2!(
            crate::x86::bmi2::pdep_u64(x, mask),
            expand_broadword(x, mask)
        )
    }
    #[inline(always)]
    fn select_u64(self, x: u64, k: u32) -> u32 {
        native_bmi2!(crate::x86::bmi2::select_u64(x, k), select_broadword64(x, k))
    }
    #[inline(always)]
    fn xor_scan_u64(self, x: u64) -> u64 {
        native_clmul!(crate::x86::pclmulqdq::xor_scan_u64(x), xor_smear(x))
    }
    #[inline(always)]
    fn xor_scan_down_u64(self, x: u64) -> u64 {
        native_clmul!(
            crate::x86::pclmulqdq::xor_scan_down_u64(x),
            xor_smear_down(x)
        )
    }
}

#[cfg(target_arch = "x86_64")]
pub use x86v3::X86V3;

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
mod x86v3 {
    use super::{Isa, sealed};
    use crate::x86::{bmi2, pclmulqdq};

    /// x86-64-v3 and PCLMULQDQ: Haswell, Zen 1 and later.
    ///
    /// PEXT, PDEP and the carry-less scan; POPCNT, TZCNT and LZCNT for everything the
    /// compiler derives. PCLMULQDQ is in no psABI level; every CPU with
    /// v3 we know of has it, and the check asks anyway.
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct X86V3(());

    impl core::fmt::Debug for X86V3 {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            f.write_str("X86V3")
        }
    }

    impl X86V3 {
        /// The token, if the CPU and the OS have every feature of the
        /// level (the OS: it saves the `ymm` registers).
        #[inline]
        #[must_use]
        pub fn detect() -> Option<Self> {
            crate::cpu::x86v3().then_some(Self(()))
        }

        /// The token, unchecked.
        ///
        /// # Safety
        ///
        /// The CPU has every feature in [`X86V3::FEATURES`] and the OS
        /// saves `ymm`.
        #[inline]
        #[must_use]
        pub const unsafe fn new_unchecked() -> Self {
            Self(())
        }

        /// The `target_feature` string of the level.
        pub const FEATURES: &str = "sse3,ssse3,sse4.1,sse4.2,popcnt,cmpxchg16b,avx,avx2,\
            bmi1,bmi2,fma,lzcnt,movbe,f16c,xsave,pclmulqdq";
    }

    impl sealed::Sealed for X86V3 {}

    // SAFETY, for every method: the token exists, so the CPU has the
    // level's features (`detect` or the caller of `new_unchecked` said so).
    impl Isa for X86V3 {
        type U8x16 = crate::lanes::X86x16<Self>;
        #[inline]
        fn run<R>(self, f: impl FnOnce(Self) -> R) -> R {
            #[target_feature(
                enable = "sse3,ssse3,sse4.1,sse4.2,popcnt,cmpxchg16b,avx,avx2,bmi1,bmi2,fma,lzcnt,movbe,f16c,xsave,pclmulqdq"
            )]
            #[inline]
            fn trampoline<R>(cpu: X86V3, f: impl FnOnce(X86V3) -> R) -> R {
                f(cpu)
            }
            // SAFETY: see above.
            unsafe { trampoline(self, f) }
        }
        #[inline(always)]
        fn pext_u32(self, x: u32, mask: u32) -> u32 {
            // SAFETY: see above.
            unsafe { bmi2::pext_u32(x, mask) }
        }
        #[inline(always)]
        fn pext_u64(self, x: u64, mask: u64) -> u64 {
            // SAFETY: see above.
            unsafe { bmi2::pext_u64(x, mask) }
        }
        #[inline(always)]
        fn pdep_u32(self, x: u32, mask: u32) -> u32 {
            // SAFETY: see above.
            unsafe { bmi2::pdep_u32(x, mask) }
        }
        #[inline(always)]
        fn pdep_u64(self, x: u64, mask: u64) -> u64 {
            // SAFETY: see above.
            unsafe { bmi2::pdep_u64(x, mask) }
        }
        #[inline(always)]
        fn select_u64(self, x: u64, k: u32) -> u32 {
            // SAFETY: see above.
            unsafe { bmi2::select_u64(x, k) }
        }
        #[inline(always)]
        fn xor_scan_u64(self, x: u64) -> u64 {
            // SAFETY: see above.
            unsafe { pclmulqdq::xor_scan_u64(x) }
        }
        #[inline(always)]
        fn xor_scan_down_u64(self, x: u64) -> u64 {
            // SAFETY: see above.
            unsafe { pclmulqdq::xor_scan_down_u64(x) }
        }
    }
}

/// The levels, as one value to match on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Level {
    /// [`Portable`].
    Portable(Portable),
    /// [`X86V3`].
    #[cfg(target_arch = "x86_64")]
    X86V3(X86V3),
}

/// The highest level this CPU has. With the `portable` feature, and under
/// Miri without the features in the build, [`Level::Portable`].
#[inline]
#[must_use]
// A constant in some builds, CPUID in the rest.
#[allow(clippy::missing_const_for_fn)]
pub fn detect() -> Level {
    #[cfg(all(target_arch = "x86_64", not(feature = "portable")))]
    if let Some(cpu) = X86V3::detect() {
        return Level::X86V3(cpu);
    }
    Level::Portable(Portable)
}

/// Every level this CPU has, lowest first: for tests that want each path
/// the machine can run.
pub fn available() -> impl Iterator<Item = Level> {
    let top = detect();
    let portable = Level::Portable(Portable);
    [Some(portable), (top != portable).then_some(top)]
        .into_iter()
        .flatten()
}

/// Detects the level once (or takes one) and runs the body with a token
/// of it, compiled once per level.
///
/// ```
/// use hakmem::isa::{Isa, Level};
///
/// let x = 0b1011_0100u64;
/// assert_eq!(hakmem::dispatch!(|cpu| cpu.select(x, 2)), Some(5));
/// for level in hakmem::isa::available() {
///     assert_eq!(hakmem::dispatch!(level, |cpu| cpu.pext(x, 0xF0)), 0b1011);
/// }
/// ```
#[macro_export]
macro_rules! dispatch {
    (|$cpu:ident| $body:expr) => {
        $crate::dispatch!($crate::isa::detect(), |$cpu| $body)
    };
    ($level:expr, |$cpu:ident| $body:expr) => {
        match $level {
            $crate::isa::Level::Portable(t) => $crate::isa::Isa::run(t, |$cpu| $body),
            #[cfg(target_arch = "x86_64")]
            $crate::isa::Level::X86V3(t) => $crate::isa::Isa::run(t, |$cpu| $body),
        }
    };
}

/// The x86 levels with SSSE3, which is what [`X86x16`](crate::lanes::X86x16)
/// needs. Sealed like [`Isa`].
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
pub trait X86Level: Isa {
    /// The level has GFNI: byte maps are one `gf2p8affineqb`.
    const GFNI: bool;

    /// The token, from a value that already proves the level (a carrier
    /// of it exists).
    ///
    /// # Safety
    ///
    /// The CPU has the level's features.
    #[doc(hidden)]
    unsafe fn assume() -> Self;
}

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_code)]
impl X86Level for X86V3 {
    const GFNI: bool = false;
    unsafe fn assume() -> Self {
        // SAFETY: the caller's.
        unsafe { Self::new_unchecked() }
    }
}

// `Native` is an x86 level with SSSE3 when the build says so.
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "ssse3",
    not(feature = "portable")
))]
#[allow(unsafe_code)]
impl X86Level for Native {
    const GFNI: bool = cfg!(target_feature = "gfni");
    unsafe fn assume() -> Self {
        Self
    }
}

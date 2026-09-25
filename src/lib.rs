#![doc = include_str!("../README.md")]
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod affine;
pub mod bits;
pub mod cookbook;
mod cover;
#[cfg(target_arch = "x86_64")]
mod cpu;
pub mod curve;
pub mod dilated;
pub mod grid;
pub mod hilbert;
pub mod hilbert3;
pub mod isa;
pub mod lanes;
pub mod laws;
pub mod myers;
pub mod permute;
pub mod rank9;
pub mod set;
pub mod slice;
pub mod wide;
pub mod word;
#[cfg(target_arch = "x86_64")]
pub mod x86;

/// Everything most code needs, one `use` away.
///
/// The traits whose methods you call ([`Word`], [`Bits`], [`Words`] on
/// slices, [`Lanes`], the curves [`Curve2`] and [`Curve3`]) and the types
/// you name ([`Wide`], the Morton and Hilbert keys, [`Dilated`]). Iterator
/// and view types come back from methods and are not here; name them from
/// their modules.
pub mod prelude {
    pub use crate::bits::Bits;
    pub use crate::curve::{Curve2, Curve3};
    pub use crate::dilated::{Dilated, Morton2, Morton3};
    pub use crate::hilbert::Hilbert2;
    pub use crate::hilbert3::Hilbert3;
    pub use crate::lanes::Lanes;
    pub use crate::slice::Words;
    pub use crate::wide::Wide;
    pub use crate::word::Word;
}

pub use prelude::*;

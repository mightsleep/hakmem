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
#[cfg(all(target_arch = "x86_64", not(feature = "portable")))]
mod cpu;
pub mod dilated;
pub mod grid;
pub mod hilbert;
pub mod hilbert3;
pub mod lanes;
pub mod laws;
pub mod myers;
pub mod permute;
pub mod rank9;
pub mod set;
pub mod slice;
pub mod wide;
pub mod word;

/// Everything most code needs: the [`Bits`] trait, the [`Word`]
/// carrier trait, and the dilated / Morton / positions types.
pub mod prelude {
    pub use crate::bits::Bits;
    pub use crate::dilated::{Dilated, Morton2, Morton3};
    pub use crate::hilbert::Hilbert2;
    pub use crate::hilbert3::Hilbert3;
    pub use crate::set::Positions;
    pub use crate::wide::Wide;
    pub use crate::word::Word;
}

pub use prelude::*;

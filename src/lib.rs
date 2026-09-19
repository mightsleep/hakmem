#![doc = include_str!("../README.md")]
#![no_std]
#![deny(unsafe_code)]

pub mod bits;
pub mod cookbook;
pub mod dilated;
pub mod grid;
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
    pub use crate::dilated::{Dilated, Morton2};
    pub use crate::set::Positions;
    pub use crate::wide::Wide;
    pub use crate::word::Word;
}

pub use prelude::*;

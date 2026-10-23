#![no_std]

pub(crate) mod splitmix;

#[cfg(feature = "biski")]
mod biski;
#[cfg(feature = "biski")]
pub use crate::biski::Biski64;

#[cfg(feature = "sfmt")]
mod sfmt;
#[cfg(feature = "sfmt")]
pub use crate::sfmt::{
    Sfmt607, Sfmt1279, Sfmt2281, Sfmt4253, Sfmt11213, Sfmt19937, Sfmt44497, Sfmt86243, Sfmt132049,
    Sfmt216091,
};

#[cfg(feature = "xorwow")]
mod xorwow;
#[cfg(feature = "xorwow")]
pub use crate::xorwow::Xorwow;

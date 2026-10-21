mod splitmix;

#[cfg(feature = "biski")]
mod biski;
#[cfg(feature = "biski")]
pub use crate::biski::Biski64;

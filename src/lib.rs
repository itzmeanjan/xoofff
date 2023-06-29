#![cfg_attr(feature = "simd", feature(portable_simd))]

mod rolling;

#[cfg(feature = "dev")]
pub mod xoodoo;
#[cfg(not(feature = "dev"))]
mod xoodoo;

#[allow(unused)]
mod xoofff;

#[cfg(not(feature = "simd"))]
pub use crate::xoofff::Xoofff;

#[cfg(feature = "simd")]
mod simd;

#[cfg(feature = "simd")]
pub use crate::simd::Xoofff;

#[cfg(test)]
mod tests;

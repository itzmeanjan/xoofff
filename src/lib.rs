mod rolling;

#[cfg(feature = "dev")]
pub mod xoodoo;
#[cfg(not(feature = "dev"))]
mod xoodoo;

mod xoofff;
pub use xoofff::Xoofff;

#[cfg(test)]
mod tests;

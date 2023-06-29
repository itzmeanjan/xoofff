mod xoodoo;

// target_arch || target_feature || x   || runtime detection possible
// wasm32      || simd128        || x4  || no, make two modules
// x86/x86_64  || avx2           || x8  || yes
// x86/x86_64  || avx512         || x16 || yes

#[cfg(feature = "dev")]
pub use xoodoo::permutex;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx512"
))]
mod x16;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx512"
))]
pub use x16::Xoofff;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
mod x8;

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "avx2"
))]
pub use x8::Xoofff;

#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
mod x4;

#[cfg(all(target_arch = "wasm32", target_feature = "simd128"))]
pub use x4::Xoofff;

#![cfg_attr(docs, feature(doc_cfg))]
#![cfg_attr(coverage, feature(no_coverage))]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(
    future_incompatible,
    nonstandard_style,
    rust_2018_idioms,
    rust_2021_compatibility,
    unused,
    // missing_docs,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    // unreachable_pub,
    unused_qualifications,
    variant_size_differences
)]
#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/teoxoy/encase/3d6d2e4d7670863e97463a15ceeafac6d13ee73e/logo.svg"
)]

/// Used to implement `WgslType` for structs
///
/// # Attributes
///
/// Struct attributes
///
/// - `#[assert_uniform_compat]`
/// used to assert at compile time that the struct meets the requirements of the
/// [uniform address space restrictions on stored values](https://gpuweb.github.io/gpuweb/wgsl/#address-spaces-uniform) and the
/// [uniform address space layout constraints](https://gpuweb.github.io/gpuweb/wgsl/#address-space-layout-constraints).
///
///     You can also use [`WgslType::assert_uniform_compat()`] instead
///
/// Field attributes
///
/// - `#[align(X)]` where `X` is a power of 2 [`u32`] literal (equivalent to [WGSL align attribute](https://gpuweb.github.io/gpuweb/wgsl/#attribute-align))
///
///     Used to increase the alignment of the field
///
/// - `#[size(X)]` where `X` is a [`u32`] literal (equivalent to [WGSL size attribute](https://gpuweb.github.io/gpuweb/wgsl/#attribute-size))
///
///     Used to increase the size of the field
///
/// - `#[size(runtime)]` can only be attached to the last field of the struct
///
///     Used to denote the fact that the field it is attached to is a runtime-sized array
///
/// # Note about generics
///
/// While structs using generic type parameters are supported by this derive macro
///
/// - the `#[assert_uniform_compat]` attribute won't work with such a struct
///
/// - the `#[align(X)]` and `#[size(X)]` attributes will only work
/// if they are attached to fields whose type contains no generic type parameters
///
/// # Examples
///
/// Simple
///
/// ```
/// # use mint;
/// # use crate::encase::WgslType;
/// #[derive(WgslType)]
/// struct AffineTransform2D {
///     matrix: mint::ColumnMatrix2<f32>,
///     translate: mint::Vector2<f32>
/// }
/// ```
///
/// Contains a runtime-sized array
///
/// _The [`ArrayLength`] type can be used to explicitly write or read the length of the contained runtime-sized array_
///
/// ```
/// # use mint;
/// # use crate::encase::WgslType;
/// # use crate::encase::ArrayLength;
/// #[derive(WgslType)]
/// struct Positions {
///     length: ArrayLength,
///     #[size(runtime)]
///     positions: Vec<mint::Point2<f32>>
/// }
/// ```
///
/// Assert uniform address space requirements
///
/// _Will not compile since runtime-sized arrays are not compatible with the
/// uniform address space restrictions on stored values_
///
/// ```compile_fail,E0080
/// # use crate::encase::WgslType;
/// # use mint;
/// #[derive(WgslType)]
/// #[assert_uniform_compat]
/// struct Invalid {
///     #[size(runtime)]
///     vec: Vec<mint::Vector4<f32>>
/// }
/// ```
///
/// _Will not compile_
///
/// ```compile_fail,E0080
/// # use crate::encase::WgslType;
/// #[derive(WgslType)]
/// #[assert_uniform_compat]
/// struct Invalid {
///     a: f32,
///     b: f32, // invalid: offset of b is 4 bytes, but must be at least 16
/// }
/// ```
///
/// _Will compile (fixed via align attribute)_
///
/// ```
/// # use crate::encase::WgslType;
/// #[derive(WgslType)]
/// #[assert_uniform_compat]
/// struct Valid {
///     a: f32,
///     #[align(16)]
///     b: f32, // valid: offset of b is 16 bytes
/// }
/// ```
///
/// _Will compile (fixed via size attribute)_
///
/// ```
/// # use crate::encase::WgslType;
/// #[derive(WgslType)]
/// #[assert_uniform_compat]
/// struct Valid {
///     #[size(16)]
///     a: f32,
///     b: f32, // valid: offset of b is 16 bytes
/// }
/// ```
///
/// Complex
///
/// ```
/// # use crate::encase::WgslType;
/// #[derive(WgslType)]
/// struct Complex<
///     'a,
///     'b: 'a,
///     E: 'a + WgslType + encase::Size,
///     T: 'b + WgslType + encase::Size,
///     const N: usize,
/// > {
///     array: [&'a mut E; N],
///     #[size(runtime)]
///     rts_array: &'a mut Vec<&'b T>,
/// }
/// ```
///
pub use encase_derive::WgslType;

#[macro_use]
mod utils;
mod core;
mod types;

mod impls;

pub use crate::core::{
    CalculateSizeFor, DynamicStorageBuffer, DynamicUniformBuffer, Size, StorageBuffer,
    UniformBuffer, WgslType,
};
pub use types::runtime_sized_array::ArrayLength;

pub mod internal {
    pub use super::core::{
        AlignmentValue, BufferMut, BufferRef, CreateFrom, EnlargeError, Error, ReadContext,
        ReadFrom, Reader, Result, SizeValue, WriteContext, WriteInto, Writer,
    };
}

/// Module containing items necessary to implement `WgslType` for runtime-sized arrays
pub mod rts_array {
    #[doc(inline)]
    pub use super::impl_rts_array;
    pub use super::types::runtime_sized_array::{Length, Truncate};
}

/// Module containing items necessary to implement `WgslType` for vectors
pub mod vector {
    #[doc(inline)]
    pub use super::impl_vector;
    pub use super::types::vector::{
        AsMutVectorParts, AsRefVectorParts, FromVectorParts, VectorScalar,
    };
}

/// Module containing items necessary to implement `WgslType` for matrices
pub mod matrix {
    #[doc(inline)]
    pub use super::impl_matrix;
    pub use super::types::matrix::{
        AsMutMatrixParts, AsRefMatrixParts, FromMatrixParts, MatrixScalar,
    };
}

/// Private module used by macros
#[doc(hidden)]
pub mod private {
    pub use super::build_struct;
    pub use super::core::AlignmentValue;
    pub use super::core::BufferMut;
    pub use super::core::BufferRef;
    pub use super::core::CreateFrom;
    pub use super::core::Metadata;
    pub use super::core::ReadFrom;
    pub use super::core::Reader;
    pub use super::core::RuntimeSizedArray;
    pub use super::core::SizeValue;
    pub use super::core::WriteInto;
    pub use super::core::Writer;
    pub use super::types::array::ArrayMetadata;
    pub use super::types::matrix::*;
    pub use super::types::r#struct::StructMetadata;
    pub use super::types::runtime_sized_array::{ArrayLength, Length, Truncate};
    pub use super::types::vector::*;
    pub use super::utils::consume_zsts;
    pub use super::utils::ArrayExt;
    pub use super::CalculateSizeFor;
    pub use super::Size;
    pub use super::WgslType;
    pub use const_panic::concat_assert;
}

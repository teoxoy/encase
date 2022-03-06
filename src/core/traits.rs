use std::num::NonZeroU64;

use super::{AlignmentValue, BufferMut, BufferRef, Reader, SizeValue, Writer};

pub const MIN_UNIFORM_ALIGNMENT: AlignmentValue = AlignmentValue::new(16);

pub struct Metadata<E> {
    pub alignment: AlignmentValue,
    pub min_size: SizeValue,
    pub extra: E,
}

impl Metadata<()> {
    pub const fn from_alignment_and_size(alignment: u64, size: u64) -> Self {
        Self {
            alignment: AlignmentValue::new(alignment),
            min_size: SizeValue::new(size),
            extra: (),
        }
    }
}

// using forget() avoids "destructors cannot be evaluated at compile-time" error
// track #![feature(const_precise_live_drops)] (https://github.com/rust-lang/rust/issues/73255)

impl<E> Metadata<E> {
    pub const fn alignment(self) -> AlignmentValue {
        let value = self.alignment;
        core::mem::forget(self);
        value
    }

    pub const fn min_size(self) -> SizeValue {
        let value = self.min_size;
        core::mem::forget(self);
        value
    }
}

/// Base trait for all [WGSL host-shareable types](https://gpuweb.github.io/gpuweb/wgsl/#host-shareable-types)
pub trait WgslType {
    #[doc(hidden)]
    type ExtraMetadata;
    #[doc(hidden)]
    const METADATA: Metadata<Self::ExtraMetadata>;

    /// Represents the minimum size of `Self` (equivalent to [GPUBufferBindingLayout.minBindingSize](https://gpuweb.github.io/gpuweb/#dom-gpubufferbindinglayout-minbindingsize))
    ///
    /// For [WGSL fixed-footprint types](https://gpuweb.github.io/gpuweb/wgsl/#fixed-footprint-types)
    /// it represents [WGSL Size](https://gpuweb.github.io/gpuweb/wgsl/#alignment-and-size)
    /// (equivalent to [`Size::SIZE`])
    ///
    /// For
    /// [WGSL runtime-sized arrays](https://gpuweb.github.io/gpuweb/wgsl/#runtime-sized) and
    /// [WGSL structs containing runtime-sized arrays](https://gpuweb.github.io/gpuweb/wgsl/#struct-types)
    /// (non fixed-footprint types)
    /// this will be calculated by assuming the array has one element
    fn min_size() -> NonZeroU64 {
        Self::METADATA.min_size().0
    }

    /// Returns the size of `Self` at runtime
    ///
    /// For [WGSL fixed-footprint types](https://gpuweb.github.io/gpuweb/wgsl/#fixed-footprint-types)
    /// it's equivalent to [`Self::min_size`] and [`Size::SIZE`]
    fn size(&self) -> NonZeroU64 {
        Self::METADATA.min_size().0
    }

    #[doc(hidden)]
    const UNIFORM_COMPAT_ASSERT: () = ();

    /// Asserts at compile time that `Self` meets the requirements of the
    /// [uniform address space restrictions on stored values](https://gpuweb.github.io/gpuweb/wgsl/#address-spaces-uniform) and the
    /// [uniform address space layout constraints](https://gpuweb.github.io/gpuweb/wgsl/#address-space-layout-constraints)
    ///
    /// # Examples
    ///
    /// Will not compile since runtime-sized arrays are not compatible with the
    /// uniform address space restrictions on stored values
    ///
    /// ```compile_fail,E0080
    /// # use crate::encase::WgslType;
    /// <Vec<mint::Vector4<f32>>>::assert_uniform_compat();
    /// ```
    ///
    /// Will not compile since the stride is 4 bytes
    ///
    /// ```compile_fail,E0080
    /// # use crate::encase::WgslType;
    /// <[f32; 2]>::assert_uniform_compat();
    /// ```
    ///
    /// Will compile since the stride is 16 bytes
    ///
    /// ```
    /// # use crate::encase::WgslType;
    /// # use mint;
    /// <[mint::Vector4<f32>; 2]>::assert_uniform_compat();
    /// ```
    fn assert_uniform_compat() {
        Self::UNIFORM_COMPAT_ASSERT
    }

    // fn assert_can_write_into()
    // where
    //     Self: WriteInto,
    // {
    // }

    // fn assert_can_read_from()
    // where
    //     Self: ReadFrom,
    // {
    // }

    // fn assert_can_create_from()
    // where
    //     Self: CreateFrom,
    // {
    // }
}

/// Trait implemented for all [WGSL fixed-footprint types](https://gpuweb.github.io/gpuweb/wgsl/#fixed-footprint-types)
pub trait Size: WgslType {
    /// Represents [WGSL Size](https://gpuweb.github.io/gpuweb/wgsl/#alignment-and-size) (equivalent to [`WgslType::min_size`])
    const SIZE: NonZeroU64 = Self::METADATA.min_size().0;
}

/// Trait implemented for
/// [WGSL runtime-sized arrays](https://gpuweb.github.io/gpuweb/wgsl/#runtime-sized) and
/// [WGSL structs containing runtime-sized arrays](https://gpuweb.github.io/gpuweb/wgsl/#struct-types)
/// (non fixed-footprint types)
pub trait CalculateSizeFor {
    /// Returns the size of `Self` assuming the (contained) runtime-sized array has `nr_of_el` elements
    fn calculate_size_for(nr_of_el: u64) -> NonZeroU64;
}

#[allow(clippy::len_without_is_empty)]
pub trait RuntimeSizedArray {
    fn len(&self) -> usize;
}

pub trait WriteInto {
    fn write_into<B>(&self, writer: &mut Writer<B>)
    where
        B: BufferMut;
}

pub trait ReadFrom {
    fn read_from<B>(&mut self, reader: &mut Reader<B>)
    where
        B: BufferRef;
}

pub trait CreateFrom: Sized {
    fn create_from<B>(reader: &mut Reader<B>) -> Self
    where
        B: BufferRef;
}

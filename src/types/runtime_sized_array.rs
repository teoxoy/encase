use std::collections::{LinkedList, VecDeque};

use crate::core::{
    BufferMut, BufferRef, CreateFrom, Metadata, ReadFrom, Reader, RuntimeSizedArray, ShaderSize,
    WriteInto, Writer,
};
use crate::ShaderType;

/// Helper type meant to be used together with the [`derive@ShaderType`] derive macro
///
/// This type should be interpreted as an [`u32`] in the shader
///
/// # Problem
///
/// There are cases where the use of the WGSL function [`arrayLength()`](https://gpuweb.github.io/gpuweb/wgsl/#array-builtin-functions)
/// might be inadequate because of its return value
///
/// - being a minimum of 1 due to how [`minBindingSize` is calculated](https://gpuweb.github.io/gpuweb/#ref-for-dom-gpubufferbindinglayout-minbindingsize%E2%91%A7)
///
/// - possibly being higher than expected due to padding at the end of a struct or buffer being interpreted as array elements
///
/// - representing the capacity of the array for usecaseses that require oversized buffers
///
/// # Solution
///
/// Using this type on a field of a struct with the [`derive@ShaderType`] derive macro will automatically:
///
/// - on write, write the length of the contained runtime-sized array as an [`u32`] to the buffer
///
/// - on read, read the value as an [`u32`] from the buffer (rep as `LEN`) and when reading the elements of the contained runtime-sized array a max of `LEN` elements will be read
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ArrayLength;

impl ShaderType for ArrayLength {
    type ExtraMetadata = ();
    const METADATA: Metadata<Self::ExtraMetadata> = Metadata::from_alignment_and_size(4, 4);
}

impl ShaderSize for ArrayLength {}

impl WriteInto for ArrayLength {
    fn write_into<B: BufferMut>(&self, writer: &mut Writer<B>) {
        let length = writer.ctx.rts_array_length.unwrap();
        WriteInto::write_into(&length, writer);
    }
}

impl ReadFrom for ArrayLength {
    fn read_from<B: BufferRef>(&mut self, reader: &mut Reader<B>) {
        let length = CreateFrom::create_from(reader);
        reader.ctx.rts_array_max_el_to_read = Some(length);
    }
}

impl CreateFrom for ArrayLength {
    fn create_from<B: BufferRef>(reader: &mut Reader<B>) -> Self {
        let length = CreateFrom::create_from(reader);
        reader.ctx.rts_array_max_el_to_read = Some(length);
        ArrayLength
    }
}

pub trait Length {
    fn length(&self) -> usize;
}

pub trait Truncate {
    fn truncate(&mut self, _len: usize);
}

/// Used to implement `ShaderType` for the given runtime-sized array type
///
/// The given runtime-sized array type should implement [`Length`] and optionally [`Truncate`]
/// depending on needed capability (they can also be derived via `$using`)
///
/// # Args
///
/// - `$type` the type (representing a runtime-sized array) for which `ShaderType` will be imeplemented for
///
/// - `$generics` \[optional\] generics that will be passed into the `impl< >`
///
/// - `$using` \[optional\] can be any combination of `len truncate`
#[macro_export]
macro_rules! impl_rts_array {
    ($type:ty $( ; using $($using:tt)* )?) => {
        $crate::impl_rts_array_inner!(__inner, ($type, T); $( $($using)* )?);
    };
    ($type:ty; ($($generics:tt)*) $( ; using $($using:tt)* )?) => {
        $crate::impl_rts_array_inner!(__inner, ($type, $($generics)*); $( $($using)* )?);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_rts_array_inner {
    (__inner, ($($other:tt)*); len $($using:tt)*) => {
        $crate::impl_rts_array_inner!(__len, $($other)*);
        $crate::impl_rts_array_inner!(__inner, ($($other)*); $($using)*);
    };
    (__inner, ($($other:tt)*); truncate $($using:tt)*) => {
        $crate::impl_rts_array_inner!(__truncate, $($other)*);
        $crate::impl_rts_array_inner!(__inner, ($($other)*); $($using)*);
    };
    (__inner, ($type:ty, $($generics:tt)*); ) => {
        $crate::impl_rts_array_inner!(__main, $type, $($generics)*);
    };

    (__len, $type:ty, $($generics:tt)*) => {
        impl<$($generics)*> $crate::private::Length for $type {
            fn length(&self) -> ::core::primitive::usize {
                self.len()
            }
        }
    };
    (__truncate, $type:ty, $($generics:tt)*) => {
        impl<$($generics)*> $crate::private::Truncate for $type {
            fn truncate(&mut self, len: ::core::primitive::usize) {
                self.truncate(len)
            }
        }
    };
    (__main, $type:ty, $($generics:tt)*) => {
        impl<$($generics)*> $crate::private::ShaderType for $type
        where
            T: $crate::private::ShaderType + $crate::private::ShaderSize,
            Self: $crate::private::Length,
        {
            type ExtraMetadata = $crate::private::ArrayMetadata;
            const METADATA: $crate::private::Metadata<Self::ExtraMetadata> = {
                let alignment = T::METADATA.alignment();
                let el_size = $crate::private::SizeValue::from(T::SHADER_SIZE);

                let stride = alignment.round_up_size(el_size);
                let el_padding = alignment.padding_needed_for(el_size.get());

                $crate::private::Metadata {
                    alignment,
                    has_uniform_min_alignment: true,
                    min_size: el_size,
                    extra: $crate::private::ArrayMetadata { stride, el_padding },
                }
            };

            const UNIFORM_COMPAT_ASSERT: fn() = ||
                ::core::panic!("runtime-sized array can't be used in uniform buffers");

            fn size(&self) -> ::core::num::NonZeroU64 {
                use ::core::cmp::Ord;

                Self::METADATA.stride()
                    .mul($crate::private::Length::length(self).max(1) as ::core::primitive::u64)
                    .0
            }
        }

        impl<$($generics)*> $crate::private::RuntimeSizedArray for $type
        where
            Self: $crate::private::Length,
        {
            fn len(&self) -> ::core::primitive::usize {
                $crate::private::Length::length(self)
            }
        }

        impl<$($generics)*> $crate::private::CalculateSizeFor for $type
        where
            Self: $crate::private::ShaderType<ExtraMetadata = $crate::private::ArrayMetadata>,
        {
            fn calculate_size_for(nr_of_el: ::core::primitive::u64) -> ::core::num::NonZeroU64 {
                use ::core::cmp::Ord;

                <Self as $crate::private::ShaderType>::METADATA.stride().mul(nr_of_el.max(1)).0
            }
        }

        impl<$($generics)*> $crate::private::WriteInto for $type
        where
            T: $crate::private::WriteInto,
            Self: $crate::private::ShaderType<ExtraMetadata = $crate::private::ArrayMetadata>,
            for<'a> &'a Self: ::core::iter::IntoIterator<Item = &'a T>,
        {
            fn write_into<B: $crate::private::BufferMut>(&self, writer: &mut $crate::private::Writer<B>) {
                use ::core::iter::IntoIterator;

                for item in self.into_iter() {
                    $crate::private::WriteInto::write_into(item, writer);
                    writer.advance(<Self as $crate::private::ShaderType>::METADATA.el_padding() as ::core::primitive::usize);
                }
            }
        }

        impl<$($generics)*> $crate::private::ReadFrom for $type
        where
            T: $crate::private::ReadFrom + $crate::private::CreateFrom,
            Self: $crate::private::Truncate + $crate::private::Length + ::core::iter::Extend<T> + $crate::private::ShaderType<ExtraMetadata = $crate::private::ArrayMetadata>,
            for<'a> &'a mut Self: ::core::iter::IntoIterator<Item = &'a mut T>,
        {
            fn read_from<B: $crate::private::BufferRef>(&mut self, reader: &mut $crate::private::Reader<B>) {
                use ::core::cmp::Ord;
                use ::core::iter::{IntoIterator, Extend, Iterator};

                let max = reader.ctx.rts_array_max_el_to_read.unwrap_or(::core::primitive::u32::MAX) as ::core::primitive::usize;
                let count = max.min(reader.remaining() / <Self as $crate::private::ShaderType>::METADATA.stride().get() as ::core::primitive::usize);
                $crate::private::Truncate::truncate(self, count);

                for item in self.into_iter() {
                    $crate::private::ReadFrom::read_from(item, reader);
                    reader.advance(<Self as $crate::private::ShaderType>::METADATA.el_padding() as ::core::primitive::usize);
                }

                let remaining = count - $crate::private::Length::length(self);
                self.extend(
                    ::core::iter::repeat_with(|| {
                        let el = $crate::private::CreateFrom::create_from(reader);
                        reader.advance(<Self as $crate::private::ShaderType>::METADATA.el_padding() as ::core::primitive::usize);
                        el
                    })
                    .take(remaining),
                );
            }
        }

        impl<$($generics)*> $crate::private::CreateFrom for $type
        where
            T: $crate::private::CreateFrom,
            Self: ::core::iter::FromIterator<T> + $crate::private::ShaderType<ExtraMetadata = $crate::private::ArrayMetadata>,
        {
            fn create_from<B: $crate::private::BufferRef>(reader: &mut $crate::private::Reader<B>) -> Self {
                use ::core::cmp::Ord;
                use ::core::iter::Iterator;

                let max = reader.ctx.rts_array_max_el_to_read.unwrap_or(::core::primitive::u32::MAX) as ::core::primitive::usize;
                let count = max.min(reader.remaining() / <Self as $crate::private::ShaderType>::METADATA.stride().get() as ::core::primitive::usize);

                ::core::iter::FromIterator::from_iter(
                    ::core::iter::repeat_with(|| {
                        let el = $crate::private::CreateFrom::create_from(reader);
                        reader.advance(<Self as $crate::private::ShaderType>::METADATA.el_padding() as ::core::primitive::usize);
                        el
                    })
                    .take(count),
                )
            }
        }
    };
}

impl_rts_array!([T]; using len);
impl_rts_array!(Vec<T>; using len truncate);
impl_rts_array!(VecDeque<T>; using len truncate);
impl_rts_array!(LinkedList<T>; using len);

impl<T> Truncate for LinkedList<T> {
    fn truncate(&mut self, len: usize) {
        if len < self.len() {
            self.split_off(len);
        }
    }
}

#[cfg(test)]
mod array_length {
    use super::ArrayLength;

    #[test]
    fn derived_traits() {
        assert_eq!(ArrayLength::default(), ArrayLength.clone());

        assert_eq!(format!("{:?}", ArrayLength), "ArrayLength");
    }
}

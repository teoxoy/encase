use crate::core::{
    BufferMut, BufferRef, CreateFrom, IOType, Metadata, ReadFrom, Reader, ShaderSize, ShaderType,
    WriteInto, Writer,
};
use core::num::{NonZeroI32, NonZeroU32, Wrapping};
use core::sync::atomic::{AtomicI32, AtomicU32};

macro_rules! impl_basic_traits {
    ($type:ty) => {
        impl_basic_traits!(__main, $type, );
    };
    ($type:ty, is_pod) => {
        impl_basic_traits!(__main, $type, .pod());
    };
    (__main, $type:ty, $($tail:tt)*) => {
        impl ShaderType for $type {
            type ExtraMetadata = ();
            const METADATA: Metadata<Self::ExtraMetadata> = Metadata::from_alignment_and_size(4, 4) $($tail)*;
        }

        impl ShaderSize for $type {}
    };
}

macro_rules! impl_traits_for_pod {
    ($type:ty) => {
        impl_basic_traits!($type, is_pod);

        impl WriteInto for $type {
            #[inline]
            fn write_into<B: BufferMut>(&self, writer: &mut Writer<B>) {
                writer.write(&<$type>::to_le_bytes(*self));
            }
        }

        impl ReadFrom for $type {
            #[inline]
            fn read_from<B: BufferRef>(&mut self, reader: &mut Reader<B>) {
                *self = <$type>::from_le_bytes(*reader.read());
            }
        }

        impl CreateFrom for $type {
            #[inline]
            fn create_from<B: BufferRef>(reader: &mut Reader<B>) -> Self {
                <$type>::from_le_bytes(*reader.read())
            }
        }
    };
}

impl_traits_for_pod!(f32);
impl_traits_for_pod!(u32);
impl_traits_for_pod!(i32);

macro_rules! impl_traits_for_non_zero_option {
    ($type:ty) => {
        impl_basic_traits!(Option<$type>);

        impl WriteInto for Option<$type> {
            #[inline]
            fn write_into<B: BufferMut>(&self, writer: &mut Writer<B>) {
                let value = self.map(|num| num.get()).unwrap_or(0);
                WriteInto::write_into(&value, writer);
            }
        }

        impl ReadFrom for Option<$type> {
            #[inline]
            fn read_from<B: BufferRef>(&mut self, reader: &mut Reader<B>) {
                *self = <$type>::new(CreateFrom::create_from(reader));
            }
        }

        impl CreateFrom for Option<$type> {
            #[inline]
            fn create_from<B: BufferRef>(reader: &mut Reader<B>) -> Self {
                <$type>::new(CreateFrom::create_from(reader))
            }
        }
    };
}

impl_traits_for_non_zero_option!(NonZeroU32);
impl_traits_for_non_zero_option!(NonZeroI32);

macro_rules! impl_traits_for_wrapping {
    ($type:ty) => {
        impl_basic_traits!($type);

        impl WriteInto for $type {
            #[inline]
            fn write_into<B: BufferMut>(&self, writer: &mut Writer<B>) {
                WriteInto::write_into(&self.0, writer);
            }
        }

        impl ReadFrom for $type {
            #[inline]
            fn read_from<B: BufferRef>(&mut self, reader: &mut Reader<B>) {
                ReadFrom::read_from(&mut self.0, reader);
            }
        }

        impl CreateFrom for $type {
            #[inline]
            fn create_from<B: BufferRef>(reader: &mut Reader<B>) -> Self {
                Wrapping(CreateFrom::create_from(reader))
            }
        }
    };
}

impl_traits_for_wrapping!(Wrapping<u32>);
impl_traits_for_wrapping!(Wrapping<i32>);

macro_rules! impl_traits_for_atomic {
    ($type:ty) => {
        impl_basic_traits!($type);

        impl WriteInto for $type {
            #[inline]
            fn write_into<B: BufferMut>(&self, writer: &mut Writer<B>) {
                let value = self.load(std::sync::atomic::Ordering::Relaxed);
                WriteInto::write_into(&value, writer);
            }
        }

        impl ReadFrom for $type {
            #[inline]
            fn read_from<B: BufferRef>(&mut self, reader: &mut Reader<B>) {
                ReadFrom::read_from(self.get_mut(), reader);
            }
        }

        impl CreateFrom for $type {
            #[inline]
            fn create_from<B: BufferRef>(reader: &mut Reader<B>) -> Self {
                <$type>::new(CreateFrom::create_from(reader))
            }
        }
    };
}

impl_traits_for_atomic!(AtomicU32);
impl_traits_for_atomic!(AtomicI32);

macro_rules! impl_marker_trait_for_f32 {
    ($trait:path $({ $($impl_block:tt)* })? ) => {
        impl $trait for ::core::primitive::f32 { $($($impl_block)*)? }
    };
}

macro_rules! impl_marker_trait_for_u32 {
    ($trait:path $({ $($impl_block:tt)* })? ) => {
        impl $trait for ::core::primitive::u32 { $($($impl_block)*)? }
        impl $trait for ::core::option::Option<::core::num::NonZeroU32> { $($($impl_block)*)? }
        impl $trait for ::core::num::Wrapping<::core::primitive::u32> { $($($impl_block)*)? }
        impl $trait for ::core::sync::atomic::AtomicU32 { $($($impl_block)*)? }
    };
}

macro_rules! impl_marker_trait_for_i32 {
    ($trait:path $({ $($impl_block:tt)* })? ) => {
        impl $trait for ::core::primitive::i32 { $($($impl_block)*)? }
        impl $trait for ::core::option::Option<::core::num::NonZeroI32> { $($($impl_block)*)? }
        impl $trait for ::core::num::Wrapping<::core::primitive::i32> { $($($impl_block)*)? }
        impl $trait for ::core::sync::atomic::AtomicI32 { $($($impl_block)*)? }
    };
}

// f32 float
// i32 sint snorm
// u32 uint unorm

// x2,x4
// u8
// i8
// Unorm8
// Snorm8

// x2,x4
// u16
// i16
// Unorm16
// Snorm16
// f16

// x1,x2,x3,x4
// u32
// i32
// f32

#[repr(transparent)]
struct Unorm8(u8);
#[repr(transparent)]
struct Snorm8(i8);

#[repr(transparent)]
struct Unorm16(u16);
#[repr(transparent)]
struct Snorm16(i16);

// cgmath
// mint
// nalgebra
// vek

// missing f16, and all unorm/snorm
// use half::f16

impl_marker_trait_for_f32!(IOType {
    #[cfg(feature = "wgpu")]
    const FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Float32;
});
impl_marker_trait_for_u32!(IOType {
    #[cfg(feature = "wgpu")]
    const FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Uint32;
});
impl_marker_trait_for_i32!(IOType {
    #[cfg(feature = "wgpu")]
    const FORMAT: wgpu::VertexFormat = wgpu::VertexFormat::Sint32;
});

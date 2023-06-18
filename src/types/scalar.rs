use crate::core::{
    BufferMut, BufferRef, CreateFrom, Metadata, ReadFrom, Reader, ShaderSize, ShaderType,
    WriteInto, Writer,
};
use core::num::{NonZeroI32, NonZeroU32, Wrapping};
use core::sync::atomic::{AtomicI32, AtomicU32};

macro_rules! impl_basic_traits {
    ($type:ty, $wgsl:literal) => {
        impl ShaderType for $type {
            type ExtraMetadata = ();
            const METADATA: Metadata<Self::ExtraMetadata> = Metadata::from_alignment_and_size(4, 4);
            fn wgsl_type() -> ::std::string::String {
                $wgsl.to_string()
            }
        }

        impl ShaderSize for $type {}
    };
}

macro_rules! impl_traits {
    ($type:ty, $wgsl:literal) => {
        impl_basic_traits!($type, $wgsl);

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

impl_traits!(f32, "f32");
impl_traits!(u32, "u32");
impl_traits!(i32, "i32");

macro_rules! impl_traits_for_non_zero_option {
    ($type:ty, $wgsl:literal) => {
        impl_basic_traits!(Option<$type>, $wgsl);

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

impl_traits_for_non_zero_option!(NonZeroU32, "u32");
impl_traits_for_non_zero_option!(NonZeroI32, "i32");

macro_rules! impl_traits_for_wrapping {
    ($type:ty, $wgsl:literal) => {
        impl_basic_traits!($type, $wgsl);

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

impl_traits_for_wrapping!(Wrapping<u32>, "u32");
impl_traits_for_wrapping!(Wrapping<i32>, "i32");

macro_rules! impl_traits_for_atomic {
    ($type:ty, $wgsl:literal) => {
        impl_basic_traits!($type, $wgsl);

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

impl_traits_for_atomic!(AtomicU32, "atomic<u32>");
impl_traits_for_atomic!(AtomicI32, "atomic<i32>");

macro_rules! impl_marker_trait_for_f32 {
    ($trait:path) => {
        impl $trait for ::core::primitive::f32 {}
    };
}

macro_rules! impl_marker_trait_for_u32 {
    ($trait:path) => {
        impl $trait for ::core::primitive::u32 {}
        impl $trait for ::core::num::NonZeroU32 {}
        impl $trait for ::core::option::Option<::core::num::NonZeroU32> {}
        impl $trait for ::core::num::Wrapping<::core::primitive::u32> {}
        impl $trait for ::core::sync::atomic::AtomicU32 {}
    };
}

macro_rules! impl_marker_trait_for_i32 {
    ($trait:path) => {
        impl $trait for ::core::primitive::i32 {}
        impl $trait for ::core::num::NonZeroI32 {}
        impl $trait for ::core::option::Option<::core::num::NonZeroI32> {}
        impl $trait for ::core::num::Wrapping<::core::primitive::i32> {}
        impl $trait for ::core::sync::atomic::AtomicI32 {}
    };
}

use crate::core::{
    BufferMut, BufferRef, CreateFrom, Metadata, ReadFrom, Reader, ShaderSize, ShaderType,
    SizeValue, WriteInto, Writer,
};

use core::mem::{size_of, MaybeUninit};

pub struct ArrayMetadata {
    pub stride: SizeValue,
    pub el_padding: u64,
}

impl Metadata<ArrayMetadata> {
    pub const fn stride(self) -> SizeValue {
        self.extra.stride
    }

    pub const fn el_padding(self) -> u64 {
        self.extra.el_padding
    }
}

impl<T: ShaderType + ShaderSize, const N: usize> ShaderType for [T; N] {
    type ExtraMetadata = ArrayMetadata;
    const METADATA: Metadata<Self::ExtraMetadata> = {
        let alignment = T::METADATA.alignment();
        let el_size = SizeValue::from(T::SHADER_SIZE);

        let stride = alignment.round_up_size(el_size);
        let el_padding = alignment.padding_needed_for(el_size.get());

        let size = match N {
            0 => panic!("0 sized arrays are not supported!"),
            val => stride.mul(val as u64),
        };

        Metadata {
            alignment,
            has_uniform_min_alignment: true,
            min_size: size,
            is_pod: T::METADATA.is_pod() && el_padding == 0,
            extra: ArrayMetadata { stride, el_padding },
        }
    };

    const UNIFORM_COMPAT_ASSERT: fn() = || {
        crate::utils::consume_zsts([
            <T as ShaderType>::UNIFORM_COMPAT_ASSERT(),
            if let Some(min_alignment) = Self::METADATA.uniform_min_alignment() {
                const_panic::concat_assert!(
                    min_alignment.is_aligned(Self::METADATA.stride().get()),
                    "array stride must be a multiple of ",
                    min_alignment.get(),
                    " (current stride: ",
                    Self::METADATA.stride().get(),
                    ")"
                );
            },
        ]);
    };
}

impl<T: ShaderSize, const N: usize> ShaderSize for [T; N] {}

impl<T: WriteInto, const N: usize> WriteInto for [T; N]
where
    Self: ShaderType<ExtraMetadata = ArrayMetadata>,
{
    #[inline]
    fn write_into<B: BufferMut>(&self, writer: &mut Writer<B>) {
        if_pod_and_little_endian!(if pod_and_little_endian {
            let ptr = self.as_ptr() as *const u8;
            let byte_slice: &[u8] = unsafe { core::slice::from_raw_parts(ptr, size_of::<Self>()) };
            writer.write_slice(byte_slice);
        } else {
            for elem in self {
                WriteInto::write_into(elem, writer);
                writer.advance(Self::METADATA.el_padding() as usize);
            }
        });
    }
}

impl<T: ReadFrom, const N: usize> ReadFrom for [T; N]
where
    Self: ShaderType<ExtraMetadata = ArrayMetadata>,
{
    #[inline]
    fn read_from<B: BufferRef>(&mut self, reader: &mut Reader<B>) {
        if_pod_and_little_endian!(if pod_and_little_endian {
            let ptr = self.as_mut_ptr() as *mut u8;
            let byte_slice: &mut [u8] =
                unsafe { core::slice::from_raw_parts_mut(ptr, size_of::<Self>()) };
            reader.read_slice(byte_slice);
        } else {
            for elem in self {
                ReadFrom::read_from(elem, reader);
                reader.advance(Self::METADATA.el_padding() as usize);
            }
        });
    }
}

impl<T: CreateFrom, const N: usize> CreateFrom for [T; N]
where
    Self: ShaderType<ExtraMetadata = ArrayMetadata>,
{
    #[inline]
    fn create_from<B: BufferRef>(reader: &mut Reader<B>) -> Self {
        if_pod_and_little_endian!(if pod_and_little_endian {
            let mut me = MaybeUninit::zeroed();
            let ptr: *mut MaybeUninit<Self> = &mut me;
            let ptr = ptr.cast::<u8>();
            let byte_slice: &mut [u8] =
                unsafe { core::slice::from_raw_parts_mut(ptr, size_of::<Self>()) };
            reader.read_slice(byte_slice);
            // SAFETY: All values were properly initialized by reading the bytes.
            unsafe { me.assume_init() }
        } else {
            core::array::from_fn(|_| {
                let res = CreateFrom::create_from(reader);
                reader.advance(Self::METADATA.el_padding() as usize);
                res
            })
        })
    }
}

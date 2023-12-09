use super::{
    AlignmentValue, BufferMut, BufferRef, CreateFrom, ReadFrom, Reader, Result, ShaderType,
    WriteInto, Writer,
};

pub trait Buffer {
    type B;
    fn new(buffer: Self::B) -> Self;
    fn into_inner(self) -> Self::B;
}

pub trait WritableBuffer {
    // would do type WriteResultType = (); if associated type defaults were stable
    type WriteResultType;
    fn write<T>(&mut self, value: &T) -> Result<Self::WriteResultType>
    where
        T: ?Sized + ShaderType + WriteInto;
}

pub trait ReadableBuffer {
    fn read<T>(&self, value: &mut T) -> Result<()>
    where
        T: ?Sized + ShaderType + ReadFrom;
    fn create<T>(&self) -> Result<T>
    where
        T: ShaderType + CreateFrom;
}

pub trait MutReadableBuffer {
    fn read<T>(&mut self, value: &mut T) -> Result<()>
    where
        T: ?Sized + ShaderType + ReadFrom;
    fn create<T>(&mut self) -> Result<T>
    where
        T: ShaderType + CreateFrom;
}

/// Storage buffer wrapper facilitating RW operations
pub struct StorageBuffer<B> {
    inner: B,
}

impl<B> Buffer for StorageBuffer<B> {
    type B = B;
    fn new(buffer: B) -> Self {
        Self { inner: buffer }
    }

    fn into_inner(self) -> B {
        self.inner
    }
}

impl<B> From<B> for StorageBuffer<B> {
    fn from(buffer: B) -> Self {
        Self::new(buffer)
    }
}

impl<B> AsRef<B> for StorageBuffer<B> {
    fn as_ref(&self) -> &B {
        &self.inner
    }
}

impl<B> AsMut<B> for StorageBuffer<B> {
    fn as_mut(&mut self) -> &mut B {
        &mut self.inner
    }
}

impl<B: BufferMut> WritableBuffer for StorageBuffer<B> {
    type WriteResultType = ();
    fn write<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ShaderType + WriteInto,
    {
        let mut writer = Writer::new(value, &mut self.inner, 0)?;
        value.write_into(&mut writer);
        Ok(())
    }
}

impl<B: BufferRef> ReadableBuffer for StorageBuffer<B> {
    fn read<T>(&self, value: &mut T) -> Result<()>
    where
        T: ?Sized + ShaderType + ReadFrom,
    {
        let mut writer = Reader::new::<T>(&self.inner, 0)?;
        value.read_from(&mut writer);
        Ok(())
    }

    fn create<T>(&self) -> Result<T>
    where
        T: ShaderType + CreateFrom,
    {
        let mut writer = Reader::new::<T>(&self.inner, 0)?;
        Ok(T::create_from(&mut writer))
    }
}

/// Uniform buffer wrapper facilitating RW operations
pub struct UniformBuffer<B> {
    inner: StorageBuffer<B>,
}

impl<B> Buffer for UniformBuffer<B> {
    type B = B;
    fn new(buffer: B) -> Self {
        Self {
            inner: StorageBuffer::new(buffer),
        }
    }

    fn into_inner(self) -> B {
        self.inner.inner
    }
}

impl<B> From<B> for UniformBuffer<B> {
    fn from(buffer: B) -> Self {
        Self::new(buffer)
    }
}

impl<B> AsRef<B> for UniformBuffer<B> {
    fn as_ref(&self) -> &B {
        &self.inner.inner
    }
}

impl<B> AsMut<B> for UniformBuffer<B> {
    fn as_mut(&mut self) -> &mut B {
        &mut self.inner.inner
    }
}

impl<B: BufferMut> WritableBuffer for UniformBuffer<B> {
    type WriteResultType = ();
    fn write<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + ShaderType + WriteInto,
    {
        T::assert_uniform_compat();
        self.inner.write(value)
    }
}

impl<B: BufferRef> ReadableBuffer for UniformBuffer<B> {
    fn read<T>(&self, value: &mut T) -> Result<()>
    where
        T: ?Sized + ShaderType + ReadFrom,
    {
        T::assert_uniform_compat();
        self.inner.read(value)
    }

    fn create<T>(&self) -> Result<T>
    where
        T: ShaderType + CreateFrom,
    {
        T::assert_uniform_compat();
        self.inner.create()
    }
}

/// Dynamic storage buffer wrapper facilitating RW operations
pub struct DynamicStorageBuffer<B> {
    inner: B,
    alignment: AlignmentValue,
    offset: usize,
}

impl<B> Buffer for DynamicStorageBuffer<B> {
    type B = B;
    /// Creates a new dynamic storage buffer wrapper with an alignment of 256
    /// (default alignment in the WebGPU spec).
    fn new(buffer: B) -> Self {
        Self::new_with_alignment(buffer, 256)
    }
    fn into_inner(self) -> B {
        self.inner
    }
}

impl<B> DynamicStorageBuffer<B> {
    /// Creates a new dynamic storage buffer wrapper with a given alignment.
    /// # Panics
    ///
    /// - if `alignment` is not a power of two.
    /// - if `alignment` is less than 32 (min alignment imposed by the WebGPU spec).
    pub const fn new_with_alignment(buffer: B, alignment: u64) -> Self {
        if alignment < 32 {
            panic!("Alignment must be at least 32!");
        }
        Self {
            inner: buffer,
            alignment: AlignmentValue::new(alignment),
            offset: 0,
        }
    }

    pub fn set_offset(&mut self, offset: u64) {
        if !self.alignment.is_aligned(offset) {
            panic!(
                "offset of {} bytes is not aligned to alignment of {} bytes",
                offset,
                self.alignment.get()
            );
        }

        self.offset = offset as usize;
    }
}

impl<B> From<B> for DynamicStorageBuffer<B> {
    fn from(buffer: B) -> Self {
        Self::new(buffer)
    }
}

impl<B> AsRef<B> for DynamicStorageBuffer<B> {
    fn as_ref(&self) -> &B {
        &self.inner
    }
}

impl<B> AsMut<B> for DynamicStorageBuffer<B> {
    fn as_mut(&mut self) -> &mut B {
        &mut self.inner
    }
}

impl<B: BufferMut> WritableBuffer for DynamicStorageBuffer<B> {
    type WriteResultType = u64;
    fn write<T>(&mut self, value: &T) -> Result<u64>
    where
        T: ?Sized + ShaderType + WriteInto,
    {
        let offset = self.offset;

        let mut writer = Writer::new(value, &mut self.inner, offset)?;
        value.write_into(&mut writer);

        self.offset += self.alignment.round_up(value.size().get()) as usize;

        Ok(offset as u64)
    }
}

impl<B: BufferRef> MutReadableBuffer for DynamicStorageBuffer<B> {
    fn read<T>(&mut self, value: &mut T) -> Result<()>
    where
        T: ?Sized + ShaderType + ReadFrom,
    {
        let mut writer = Reader::new::<T>(&self.inner, self.offset)?;
        value.read_from(&mut writer);

        self.offset += self.alignment.round_up(value.size().get()) as usize;

        Ok(())
    }

    fn create<T>(&mut self) -> Result<T>
    where
        T: ShaderType + CreateFrom,
    {
        let mut writer = Reader::new::<T>(&self.inner, self.offset)?;
        let value = T::create_from(&mut writer);

        self.offset += self.alignment.round_up(value.size().get()) as usize;

        Ok(value)
    }
}

/// Dynamic uniform buffer wrapper facilitating RW operations
pub struct DynamicUniformBuffer<B> {
    inner: DynamicStorageBuffer<B>,
}

impl<B> Buffer for DynamicUniformBuffer<B> {
    type B = B;
    /// Creates a new dynamic uniform buffer wrapper with an alignment of 256
    /// (default alignment in the WebGPU spec).
    fn new(buffer: B) -> Self {
        Self {
            inner: DynamicStorageBuffer::new(buffer),
        }
    }

    fn into_inner(self) -> B {
        self.inner.inner
    }
}

impl<B> DynamicUniformBuffer<B> {
    /// Creates a new dynamic uniform buffer wrapper with a given alignment.
    /// # Panics
    ///
    /// - if `alignment` is not a power of two.
    /// - if `alignment` is less than 32 (min alignment imposed by the WebGPU spec).
    pub const fn new_with_alignment(buffer: B, alignment: u64) -> Self {
        Self {
            inner: DynamicStorageBuffer::new_with_alignment(buffer, alignment),
        }
    }

    pub fn set_offset(&mut self, offset: u64) {
        self.inner.set_offset(offset);
    }
}

impl<B> From<B> for DynamicUniformBuffer<B> {
    fn from(buffer: B) -> Self {
        Self::new(buffer)
    }
}

impl<B> AsRef<B> for DynamicUniformBuffer<B> {
    fn as_ref(&self) -> &B {
        &self.inner.inner
    }
}

impl<B> AsMut<B> for DynamicUniformBuffer<B> {
    fn as_mut(&mut self) -> &mut B {
        &mut self.inner.inner
    }
}

impl<B: BufferMut> WritableBuffer for DynamicUniformBuffer<B> {
    type WriteResultType = u64;
    fn write<T>(&mut self, value: &T) -> Result<u64>
    where
        T: ?Sized + ShaderType + WriteInto,
    {
        T::assert_uniform_compat();
        self.inner.write(value)
    }
}

impl<B: BufferRef> MutReadableBuffer for DynamicUniformBuffer<B> {
    fn read<T>(&mut self, value: &mut T) -> Result<()>
    where
        T: ?Sized + ShaderType + ReadFrom,
    {
        T::assert_uniform_compat();
        self.inner.read(value)
    }

    fn create<T>(&mut self) -> Result<T>
    where
        T: ShaderType + CreateFrom,
    {
        T::assert_uniform_compat();
        self.inner.create()
    }
}

pub trait BufferContent {
    fn buffer_content<BufferType: Buffer<B = Vec<u8>> + WritableBuffer>(&self) -> Vec<u8>;
}

impl<T> BufferContent for T where T: ShaderType + WriteInto {
    fn buffer_content<BufferType: Buffer<B = Vec<u8>> + WritableBuffer>(&self) -> Vec<u8> {
        let mut buffer = BufferType::new(Vec::new());
        buffer.write(self).unwrap();
        return buffer.into_inner();
    }
}
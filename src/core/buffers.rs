use super::{
    AlignmentValue, BufferMut, BufferRef, CreateFrom, ReadFrom, Reader, Result, ShaderType,
    WriteInto, Writer,
};

/// Storage buffer wrapper facilitating RW operations
pub struct StorageBuffer<B> {
    inner: B,
}

impl<B> StorageBuffer<B> {
    pub const fn new(buffer: B) -> Self {
        Self { inner: buffer }
    }

    pub fn into_inner(self) -> B {
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

impl<B: BufferMut> StorageBuffer<B> {
    pub fn write<T>(&mut self, value: &T) -> Result<()>
    where
        T: ShaderType + WriteInto,
    {
        let mut writer = Writer::new(value, &mut self.inner, 0)?;
        value.write_into(&mut writer);
        Ok(())
    }
}

impl<B: BufferRef> StorageBuffer<B> {
    pub fn read<T>(&self, value: &mut T) -> Result<()>
    where
        T: ShaderType + ReadFrom,
    {
        let mut writer = Reader::new::<T>(&self.inner, 0)?;
        value.read_from(&mut writer);
        Ok(())
    }

    pub fn create<T>(&self) -> Result<T>
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

impl<B> UniformBuffer<B> {
    pub const fn new(buffer: B) -> Self {
        Self {
            inner: StorageBuffer::new(buffer),
        }
    }

    pub fn into_inner(self) -> B {
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

impl<B: BufferMut> UniformBuffer<B> {
    pub fn write<T>(&mut self, value: &T) -> Result<()>
    where
        T: ShaderType + WriteInto,
    {
        T::assert_uniform_compat();
        self.inner.write(value)
    }
}

impl<B: BufferRef> UniformBuffer<B> {
    pub fn read<T>(&self, value: &mut T) -> Result<()>
    where
        T: ShaderType + ReadFrom,
    {
        T::assert_uniform_compat();
        self.inner.read(value)
    }

    pub fn create<T>(&self) -> Result<T>
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

impl<B> DynamicStorageBuffer<B> {
    pub const fn new(buffer: B) -> Self {
        Self::new_with_alignment(buffer, 256)
    }

    pub const fn new_with_alignment(buffer: B, alignment: u64) -> Self {
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

    pub fn into_inner(self) -> B {
        self.inner
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

impl<B: BufferMut> DynamicStorageBuffer<B> {
    pub fn write<T>(&mut self, value: &T) -> Result<u64>
    where
        T: ShaderType + WriteInto,
    {
        let offset = self.offset;

        let mut writer = Writer::new(value, &mut self.inner, offset)?;
        value.write_into(&mut writer);

        self.offset += self.alignment.round_up(value.size().get()) as usize;

        Ok(offset as u64)
    }
}

impl<B: BufferRef> DynamicStorageBuffer<B> {
    pub fn read<T>(&mut self, value: &mut T) -> Result<()>
    where
        T: ShaderType + ReadFrom,
    {
        let mut writer = Reader::new::<T>(&self.inner, self.offset)?;
        value.read_from(&mut writer);

        self.offset += self.alignment.round_up(value.size().get()) as usize;

        Ok(())
    }

    pub fn create<T>(&mut self) -> Result<T>
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

impl<B> DynamicUniformBuffer<B> {
    pub const fn new(buffer: B) -> Self {
        Self::new_with_alignment(buffer, 256)
    }

    pub const fn new_with_alignment(buffer: B, alignment: u64) -> Self {
        Self {
            inner: DynamicStorageBuffer::new_with_alignment(buffer, alignment),
        }
    }

    pub fn set_offset(&mut self, offset: u64) {
        self.inner.set_offset(offset);
    }

    pub fn into_inner(self) -> B {
        self.inner.inner
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

impl<B: BufferMut> DynamicUniformBuffer<B> {
    pub fn write<T>(&mut self, value: &T) -> Result<u64>
    where
        T: ShaderType + WriteInto,
    {
        T::assert_uniform_compat();
        self.inner.write(value)
    }
}

impl<B: BufferRef> DynamicUniformBuffer<B> {
    pub fn read<T>(&mut self, value: &mut T) -> Result<()>
    where
        T: ShaderType + ReadFrom,
    {
        T::assert_uniform_compat();
        self.inner.read(value)
    }

    pub fn create<T>(&mut self) -> Result<T>
    where
        T: ShaderType + CreateFrom,
    {
        T::assert_uniform_compat();
        self.inner.create()
    }
}

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
        T: ?Sized + ShaderType + WriteInto,
    {
        let mut writer = Writer::new(value, &mut self.inner, 0)?;
        value.write_into(&mut writer);
        Ok(())
    }
}

impl<B: BufferRef> StorageBuffer<B> {
    pub fn read<T>(&self, value: &mut T) -> Result<()>
    where
        T: ?Sized + ShaderType + ReadFrom,
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
        T: ?Sized + ShaderType + WriteInto,
    {
        T::assert_uniform_compat();
        self.inner.write(value)
    }
}

impl<B: BufferRef> UniformBuffer<B> {
    pub fn read<T>(&self, value: &mut T) -> Result<()>
    where
        T: ?Sized + ShaderType + ReadFrom,
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
    /// Creates a new dynamic storage buffer wrapper with an alignment of 256
    /// (default alignment in the WebGPU spec).
    pub const fn new(buffer: B) -> Self {
        Self::new_with_alignment(buffer, 256)
    }

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
    /// Layouts and writes an entire bound value into the buffer. The value is written at the
    /// next available offset after the buffer's current offset, which is aligned to the required
    /// dynamic binding alignment (defaults to 256).
    ///
    /// Use this to write the entire struct you will be binding as a dynamic-offset storage buffer.
    ///
    /// Returns the offset at which the value was written.
    pub fn write<T>(&mut self, value: &T) -> Result<u64>
    where
        T: ?Sized + ShaderType + WriteInto,
    {
        self.write_dynamic_struct_break();
        let offset = self.offset;

        let mut writer = Writer::new(value, &mut self.inner, offset)?;
        value.write_into(&mut writer);

        self.offset += value.size().get() as usize;

        Ok(offset as u64)
    }

    /// Layouts and writes a single member into the buffer. The value is written at the
    /// next available offset after the buffer's current offset, which is aligned the
    /// alignment of `T`.
    ///
    /// The use case is constructing a struct member by member in case the layout isn't
    /// known at compile time. Combine this with [`Self::write_dynamic_struct_break`] to "end"
    /// the struct being written and start the next one.
    ///
    /// Returns the offset at which the value was written.
    pub fn write_single_member<T>(&mut self, value: &T) -> Result<u64>
    where
        T: ShaderType + WriteInto,
    {
        self.offset = T::METADATA.alignment().round_up(self.offset as u64) as usize;
        let offset = self.offset;

        let mut writer = Writer::new(value, &mut self.inner, offset)?;
        value.write_into(&mut writer);

        self.offset += value.size().get() as usize;

        Ok(offset as u64)
    }

    /// Writes a "struct break" into the buffer. This takes the buffer offset and aligns it
    /// to the required dynamic binding alignment (defaults to 256).
    ///
    /// The use case is constructing a struct member by member in case the layout isn't
    /// known at compile time. Combine this with [`Self::write_single_member`] to add
    /// each individual member of the struct being written.
    ///
    /// Returns the offset which was rounded up to.
    pub fn write_dynamic_struct_break(&mut self) -> u64 {
        self.offset = self.alignment.round_up(self.offset as u64) as usize;
        self.offset as u64
    }
}

impl<B: BufferRef> DynamicStorageBuffer<B> {
    /// Reads and un-layouts an entire bound value from the buffer. The value is read from the
    /// next available offset after the buffer's current offset, which is aligned to the required
    /// dynamic binding alignment (defaults to 256).
    ///
    /// Use this to read the entire struct you bound as a dynamic-offset storage buffer.
    pub fn read<T>(&mut self, value: &mut T) -> Result<()>
    where
        T: ?Sized + ShaderType + ReadFrom,
    {
        self.read_dynamic_struct_break();

        let mut reader = Reader::new::<T>(&self.inner, self.offset)?;
        value.read_from(&mut reader);

        self.offset += value.size().get() as usize;

        Ok(())
    }

    /// Reads a single member from the buffer. The value is read at the
    /// next available offset after the buffer's current offset, which is aligned the
    /// alignment of `T`.
    ///
    /// The use case is deconstructing a struct member by member in case the layout isn't
    /// known at compile time. Combine this with [`Self::read_dynamic_struct_break`] to "end"
    /// the struct being read and start the next one.
    ///
    /// Returns the offset at which the value was written.
    pub fn read_single_member<T>(&mut self, value: &mut T) -> Result<()>
    where
        T: ShaderType + ReadFrom,
    {
        self.offset = T::METADATA.alignment().round_up(self.offset as u64) as usize;

        let mut reader = Reader::new::<T>(&self.inner, self.offset)?;
        value.read_from(&mut reader);

        self.offset += value.size().get() as usize;

        Ok(())
    }

    /// Reads a "struct break" from the buffer. This takes the buffer offset and aligns it
    /// to the required dynamic binding alignment (defaults to 256).
    ///
    /// The use case is constructing a struct member by member in case the layout isn't
    /// known at compile time. Combine this with [`Self::read_single_member`] to add
    /// each individual member of the struct being written.
    pub fn read_dynamic_struct_break(&mut self) {
        self.offset = self.alignment.round_up(self.offset as u64) as usize;
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
    /// Creates a new dynamic uniform buffer wrapper with an alignment of 256
    /// (default alignment in the WebGPU spec).
    pub const fn new(buffer: B) -> Self {
        Self {
            inner: DynamicStorageBuffer::new(buffer),
        }
    }

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
    /// Layouts and writes an entire bound value into the buffer. The value is written at the
    /// next available offset after the buffer's current offset, which is aligned to the required
    /// dynamic binding alignment (defaults to 256).
    ///
    /// Use this to write the entire struct you will be binding as a dynamic-offset storage buffer.
    ///
    /// Returns the offset at which the value was written.
    pub fn write<T>(&mut self, value: &T) -> Result<u64>
    where
        T: ?Sized + ShaderType + WriteInto,
    {
        T::assert_uniform_compat();
        self.inner.write(value)
    }

    /// Layouts and writes a single member into the buffer. The value is written at the
    /// next available offset after the buffer's current offset, which is aligned the
    /// alignment of `T`.
    ///
    /// The use case is constructing a struct member by member in case the layout isn't
    /// known at compile time. Combine this with [`Self::write_dynamic_struct_break`] to "end"
    /// the struct being written and start the next one.
    ///
    /// Returns the offset at which the value was written.
    pub fn write_single_member<T>(&mut self, value: &T) -> Result<u64>
    where
        T: ShaderType + WriteInto,
    {
        T::assert_uniform_compat();
        self.inner.write_single_member(value)
    }

    /// Writes a "struct break" into the buffer. This takes the buffer offset and aligns it
    /// to the required dynamic binding alignment (defaults to 256).
    ///
    /// The use case is constructing a struct member by member in case the layout isn't
    /// known at compile time. Combine this with [`Self::write_single_member`] to add
    /// each individual member of the struct being written.
    ///
    /// Returns the offset which was rounded up to.
    pub fn write_dynamic_struct_break(&mut self) -> u64 {
        self.inner.write_dynamic_struct_break()
    }
}

impl<B: BufferRef> DynamicUniformBuffer<B> {
    /// Reads and un-layouts an entire bound value from the buffer. The value is read from the
    /// next available offset after the buffer's current offset, which is aligned to the required
    /// dynamic binding alignment (defaults to 256).
    ///
    /// Use this to read the entire struct you bound as a dynamic-offset storage buffer.
    pub fn read<T>(&mut self, value: &mut T) -> Result<()>
    where
        T: ?Sized + ShaderType + ReadFrom,
    {
        T::assert_uniform_compat();
        self.inner.read(value)
    }

    /// Reads a single member from the buffer. The value is read at the
    /// next available offset after the buffer's current offset, which is aligned the
    /// alignment of `T`.
    ///
    /// The use case is deconstructing a struct member by member in case the layout isn't
    /// known at compile time. Combine this with [`Self::read_dynamic_struct_break`] to "end"
    /// the struct being read and start the next one.
    ///
    /// Returns the offset at which the value was written.
    pub fn read_single_member<T>(&mut self, value: &mut T) -> Result<()>
    where
        T: ShaderType + ReadFrom,
    {
        T::assert_uniform_compat();
        self.inner.read_single_member(value)
    }

    /// Reads a "struct break" from the buffer. This takes the buffer offset and aligns it
    /// to the required dynamic binding alignment (defaults to 256).
    ///
    /// The use case is constructing a struct member by member in case the layout isn't
    /// known at compile time. Combine this with [`Self::read_single_member`] to add
    /// each individual member of the struct being written.
    pub fn read_dynamic_struct_break(&mut self) {
        self.inner.read_dynamic_struct_break()
    }

    pub fn create<T>(&mut self) -> Result<T>
    where
        T: ShaderType + CreateFrom,
    {
        T::assert_uniform_compat();
        self.inner.create()
    }
}

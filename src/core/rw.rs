use super::ShaderType;
use core::mem::MaybeUninit;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Error)]
pub enum Error {
    #[error("could not read/write {expected} bytes from/into {found} byte sized buffer")]
    BufferTooSmall { expected: u64, found: u64 },
}

pub type Result<T> = core::result::Result<T, Error>;

pub struct WriteContext {
    /// length of the contained runtime sized array
    ///
    /// used by the derive macro
    pub rts_array_length: Option<u32>,
}

pub struct Writer<B: BufferMut> {
    pub ctx: WriteContext,
    cursor: Cursor<B>,
}

impl<B: BufferMut> Writer<B> {
    #[inline]
    pub fn new<T: ?Sized + ShaderType>(data: &T, buffer: B, offset: usize) -> Result<Self> {
        let mut cursor = Cursor::new(buffer, offset);
        let size = data.size().get();
        if cursor.try_enlarge(offset + size as usize).is_err() {
            Err(Error::BufferTooSmall {
                expected: size,
                found: cursor.capacity() as u64,
            })
        } else {
            Ok(Self {
                ctx: WriteContext {
                    rts_array_length: None,
                },
                cursor,
            })
        }
    }

    #[inline]
    pub fn advance(&mut self, amount: usize) {
        self.cursor.advance(amount);
    }

    #[inline]
    pub fn write<const N: usize>(&mut self, val: &[u8; N]) {
        self.cursor.write(val);
    }

    #[inline]
    pub fn write_slice(&mut self, val: &[u8]) {
        self.cursor.write_slice(val)
    }
}

pub struct ReadContext {
    /// max elements to read into the contained runtime sized array
    ///
    /// used by the derive macro
    pub rts_array_max_el_to_read: Option<u32>,
}

pub struct Reader<B: BufferRef> {
    pub ctx: ReadContext,
    cursor: Cursor<B>,
}

impl<B: BufferRef> Reader<B> {
    #[inline]
    pub fn new<T: ?Sized + ShaderType>(buffer: B, offset: usize) -> Result<Self> {
        let cursor = Cursor::new(buffer, offset);
        if cursor.remaining() < T::min_size().get() as usize {
            Err(Error::BufferTooSmall {
                expected: T::min_size().get(),
                found: cursor.remaining() as u64,
            })
        } else {
            Ok(Self {
                ctx: ReadContext {
                    rts_array_max_el_to_read: None,
                },
                cursor,
            })
        }
    }

    #[inline]
    pub fn advance(&mut self, amount: usize) {
        self.cursor.advance(amount);
    }

    #[inline]
    pub fn read<const N: usize>(&mut self) -> &[u8; N] {
        self.cursor.read()
    }

    #[inline]
    pub fn read_slice(&mut self, val: &mut [u8]) {
        self.cursor.read_slice(val)
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.cursor.remaining()
    }
}

struct Cursor<B> {
    buffer: B,
    pos: usize,
}

impl<B> Cursor<B> {
    #[inline]
    fn new(buffer: B, offset: usize) -> Self {
        Self {
            buffer,
            pos: offset,
        }
    }
    #[inline]
    fn advance(&mut self, amount: usize) {
        self.pos += amount;
    }
}

impl<B: BufferRef> Cursor<B> {
    #[inline]
    fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.pos)
    }

    #[inline]
    fn read<const N: usize>(&mut self) -> &[u8; N] {
        let res = self.buffer.read(self.pos);
        self.pos += N;
        res
    }

    #[inline]
    fn read_slice(&mut self, val: &mut [u8]) {
        self.buffer.read_slice(self.pos, val);
        self.pos += val.len();
    }
}

impl<B: BufferMut> Cursor<B> {
    #[inline]
    fn capacity(&self) -> usize {
        self.buffer.capacity().saturating_sub(self.pos)
    }

    #[inline]
    fn write<const N: usize>(&mut self, val: &[u8; N]) {
        self.buffer.write(self.pos, val);
        self.pos += N;
    }

    #[inline]
    fn write_slice(&mut self, val: &[u8]) {
        self.buffer.write_slice(self.pos, val);
        self.pos += val.len();
    }

    #[inline]
    fn try_enlarge(&mut self, wanted: usize) -> core::result::Result<(), EnlargeError> {
        self.buffer.try_enlarge(wanted)
    }
}

#[derive(Clone, Copy, Debug, Error)]
#[error("could not enlarge buffer")]
pub struct EnlargeError;

impl From<std::collections::TryReserveError> for EnlargeError {
    fn from(_: std::collections::TryReserveError) -> Self {
        Self
    }
}

#[allow(clippy::len_without_is_empty)]
pub trait BufferRef {
    fn len(&self) -> usize;

    fn read<const N: usize>(&self, offset: usize) -> &[u8; N];

    fn read_slice(&self, offset: usize, val: &mut [u8]);
}

pub trait BufferMut {
    fn capacity(&self) -> usize;

    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]);

    fn write_slice(&mut self, offset: usize, val: &[u8]);

    #[inline]
    fn try_enlarge(&mut self, wanted: usize) -> core::result::Result<(), EnlargeError> {
        if wanted > self.capacity() {
            Err(EnlargeError)
        } else {
            Ok(())
        }
    }
}

impl BufferRef for [u8] {
    fn len(&self) -> usize {
        self.len()
    }

    #[inline]
    fn read<const N: usize>(&self, offset: usize) -> &[u8; N] {
        use crate::utils::SliceExt;
        self.array(offset)
    }

    #[inline]
    fn read_slice(&self, offset: usize, val: &mut [u8]) {
        val.copy_from_slice(&self[offset..offset + val.len()])
    }
}

impl<const LEN: usize> BufferRef for [u8; LEN] {
    #[inline]
    fn len(&self) -> usize {
        <[u8] as BufferRef>::len(self)
    }

    #[inline]
    fn read<const N: usize>(&self, offset: usize) -> &[u8; N] {
        <[u8] as BufferRef>::read(self, offset)
    }

    #[inline]
    fn read_slice(&self, offset: usize, val: &mut [u8]) {
        <[u8] as BufferRef>::read_slice(self, offset, val)
    }
}

impl BufferRef for Vec<u8> {
    #[inline]
    fn len(&self) -> usize {
        <[u8] as BufferRef>::len(self)
    }

    #[inline]
    fn read<const N: usize>(&self, offset: usize) -> &[u8; N] {
        <[u8] as BufferRef>::read(self, offset)
    }

    #[inline]
    fn read_slice(&self, offset: usize, val: &mut [u8]) {
        <[u8] as BufferRef>::read_slice(self, offset, val)
    }
}

impl BufferMut for [u8] {
    #[inline]
    fn capacity(&self) -> usize {
        self.len()
    }

    #[inline]
    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
        use crate::utils::SliceExt;
        *self.array_mut(offset) = *val;
    }

    #[inline]
    fn write_slice(&mut self, offset: usize, val: &[u8]) {
        self[offset..offset + val.len()].copy_from_slice(val);
    }
}

impl BufferMut for [MaybeUninit<u8>] {
    #[inline]
    fn capacity(&self) -> usize {
        self.len()
    }

    #[inline]
    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
        use crate::utils::SliceExt;
        // SAFETY: &[u8; N] and &[MaybeUninit<u8>; N] have the same layout
        let val: &[MaybeUninit<u8>; N] = unsafe { core::mem::transmute(val) };
        *self.array_mut(offset) = *val;
    }
}

impl<const LEN: usize> BufferMut for [u8; LEN] {
    #[inline]
    fn capacity(&self) -> usize {
        <[u8] as BufferMut>::capacity(self)
    }

    #[inline]
    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
        <[u8] as BufferMut>::write(self, offset, val);
    }

    #[inline]
    fn write_slice(&mut self, offset: usize, val: &[u8]) {
        <[u8] as BufferMut>::write_slice(self, offset, val)
    }
}

impl<const LEN: usize> BufferMut for [MaybeUninit<u8>; LEN] {
    #[inline]
    fn capacity(&self) -> usize {
        <[MaybeUninit<u8>] as BufferMut>::capacity(self)
    }

    #[inline]
    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
        <[MaybeUninit<u8>] as BufferMut>::write(self, offset, val)
    }
}

impl BufferMut for Vec<u8> {
    #[inline]
    fn capacity(&self) -> usize {
        self.capacity()
    }

    #[inline]
    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
        <[u8] as BufferMut>::write(self, offset, val);
    }

    #[inline]
    fn write_slice(&mut self, offset: usize, val: &[u8]) {
        <[u8] as BufferMut>::write_slice(self, offset, val)
    }

    #[inline]
    fn try_enlarge(&mut self, wanted: usize) -> core::result::Result<(), EnlargeError> {
        use crate::utils::ByteVecExt;
        self.try_extend(wanted).map_err(EnlargeError::from)
    }
}

impl BufferMut for Vec<MaybeUninit<u8>> {
    #[inline]
    fn capacity(&self) -> usize {
        self.capacity()
    }

    #[inline]
    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
        <[MaybeUninit<u8>] as BufferMut>::write(self, offset, val)
    }

    #[inline]
    fn try_enlarge(&mut self, wanted: usize) -> core::result::Result<(), EnlargeError> {
        use crate::utils::ByteVecExt;
        self.try_extend(wanted).map_err(EnlargeError::from)
    }
}

macro_rules! impl_buffer_ref_for_wrappers {
    ($($type:ty),*) => {$(
        impl<T: ?Sized + BufferRef> BufferRef for $type {
            #[inline]
            fn len(&self) -> usize {
                T::len(self)
            }

            #[inline]
            fn read<const N: usize>(&self, offset: usize) -> &[u8; N] {
                T::read(self, offset)
            }

            #[inline]
            fn read_slice(&self, offset: usize, val: &mut [u8]) {
                T::read_slice(self, offset, val)
            }
        }
    )*};
}

impl_buffer_ref_for_wrappers!(&T, &mut T, Box<T>, std::rc::Rc<T>, std::sync::Arc<T>);

macro_rules! impl_buffer_mut_for_wrappers {
    ($($type:ty),*) => {$(
        impl<T: ?Sized + BufferMut> BufferMut for $type {
            #[inline]
            fn capacity(&self) -> usize {
                T::capacity(self)
            }

            #[inline]
            fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
                T::write(self, offset, val)
            }

            #[inline]
            fn write_slice(&mut self, offset: usize, val: &[u8]) {
                T::write_slice(self, offset, val)
            }

            #[inline]
            fn try_enlarge(&mut self, wanted: usize) -> core::result::Result<(), EnlargeError> {
                T::try_enlarge(self, wanted)
            }
        }
    )*};
}

impl_buffer_mut_for_wrappers!(&mut T, Box<T>);

#[cfg(test)]
mod buffer_ref {
    use super::BufferRef;

    #[test]
    fn array() {
        let arr = [0, 1, 2, 3, 4, 5];

        assert_eq!(BufferRef::len(&arr), 6);
        assert_eq!(BufferRef::read(&arr, 3), &[3, 4]);
    }

    #[test]
    fn vec() {
        let vec = Vec::from([0, 1, 2, 3, 4, 5]);

        assert_eq!(BufferRef::len(&vec), 6);
        assert_eq!(BufferRef::read(&vec, 3), &[3, 4]);
    }
}

#[cfg(test)]
mod buffer_mut {
    use super::BufferMut;
    use crate::core::EnlargeError;

    #[test]
    fn array() {
        let mut arr = [0, 1, 2, 3, 4, 5];

        assert_eq!(BufferMut::capacity(&arr), 6);

        BufferMut::write(&mut arr, 3, &[9, 1]);
        assert_eq!(arr, [0, 1, 2, 9, 1, 5]);

        assert!(matches!(BufferMut::try_enlarge(&mut arr, 6), Ok(())));
        assert!(matches!(
            BufferMut::try_enlarge(&mut arr, 7),
            Err(EnlargeError)
        ));
    }

    #[test]
    fn vec() {
        let mut vec = Vec::from([0, 1, 2, 3, 4, 5]);

        assert_eq!(BufferMut::capacity(&vec), vec.capacity());

        BufferMut::write(&mut vec, 3, &[9, 1]);
        assert_eq!(vec, Vec::from([0, 1, 2, 9, 1, 5]));

        assert!(matches!(BufferMut::try_enlarge(&mut vec, 100), Ok(())));
        assert!(matches!(
            BufferMut::try_enlarge(&mut vec, usize::MAX),
            Err(EnlargeError)
        ));
    }
}

#[cfg(test)]
mod error {
    use super::Error;

    #[test]
    fn derived_traits() {
        let err = Error::BufferTooSmall {
            expected: 4,
            found: 2,
        };

        {
            use std::error::Error;
            assert!(err.source().is_none());
        }

        assert_eq!(
            format!("{}", err.clone()),
            "could not read/write 4 bytes from/into 2 byte sized buffer"
        );

        assert_eq!(
            format!("{:?}", err.clone()),
            "BufferTooSmall { expected: 4, found: 2 }"
        );
    }
}

#[cfg(test)]
mod enlarge_error {
    use super::EnlargeError;

    #[test]
    fn derived_traits() {
        // can't construct a TryReserveError due to TryReserveErrorKind being unstable
        let try_reserve_error = {
            let mut vec = Vec::<u8>::new();
            vec.try_reserve(usize::MAX).err().unwrap()
        };
        let err = EnlargeError::from(try_reserve_error);

        use std::error::Error;
        assert!(err.source().is_none());

        assert_eq!(format!("{}", err.clone()), "could not enlarge buffer");

        assert_eq!(format!("{:?}", err.clone()), "EnlargeError");
    }
}

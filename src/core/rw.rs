use super::ShaderType;
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
    pub fn new<T: ShaderType>(data: &T, buffer: B, offset: usize) -> Result<Self> {
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

    pub fn advance(&mut self, amount: usize) {
        self.cursor.advance(amount)
    }

    pub fn write<const N: usize>(&mut self, val: &[u8; N]) {
        self.cursor.write(val)
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
    pub fn new<T: ShaderType + ?Sized>(buffer: B, offset: usize) -> Result<Self> {
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

    pub fn advance(&mut self, amount: usize) {
        self.cursor.advance(amount)
    }

    pub fn read<const N: usize>(&mut self) -> &[u8; N] {
        self.cursor.read()
    }

    pub fn remaining(&self) -> usize {
        self.cursor.remaining()
    }
}

struct Cursor<B> {
    buffer: B,
    pos: usize,
}

impl<B> Cursor<B> {
    fn new(buffer: B, offset: usize) -> Self {
        Self {
            buffer,
            pos: offset,
        }
    }
    fn advance(&mut self, amount: usize) {
        self.pos += amount;
    }
}

impl<B: BufferRef> Cursor<B> {
    fn remaining(&self) -> usize {
        self.buffer.len().saturating_sub(self.pos)
    }

    fn read<const N: usize>(&mut self) -> &[u8; N] {
        let res = self.buffer.read(self.pos);
        self.pos += N;
        res
    }
}

impl<B: BufferMut> Cursor<B> {
    fn capacity(&self) -> usize {
        self.buffer.capacity().saturating_sub(self.pos)
    }

    fn write<const N: usize>(&mut self, val: &[u8; N]) {
        self.buffer.write(self.pos, val);
        self.pos += N;
    }

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
}

pub trait BufferMut {
    fn capacity(&self) -> usize;

    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]);

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

    fn read<const N: usize>(&self, offset: usize) -> &[u8; N] {
        use crate::utils::SliceExt;
        self.array(offset)
    }
}

impl<const LEN: usize> BufferRef for [u8; LEN] {
    fn len(&self) -> usize {
        <[u8] as BufferRef>::len(self)
    }

    fn read<const N: usize>(&self, offset: usize) -> &[u8; N] {
        <[u8] as BufferRef>::read(self, offset)
    }
}

impl BufferRef for Vec<u8> {
    fn len(&self) -> usize {
        <[u8] as BufferRef>::len(self)
    }

    fn read<const N: usize>(&self, offset: usize) -> &[u8; N] {
        <[u8] as BufferRef>::read(self, offset)
    }
}

impl BufferMut for [u8] {
    fn capacity(&self) -> usize {
        self.len()
    }

    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
        use crate::utils::SliceExt;
        self.copy_from_array(offset, val);
    }
}

impl<const LEN: usize> BufferMut for [u8; LEN] {
    fn capacity(&self) -> usize {
        <[u8] as BufferMut>::capacity(self)
    }

    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
        <[u8] as BufferMut>::write(self, offset, val)
    }
}

impl BufferMut for Vec<u8> {
    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
        <[u8] as BufferMut>::write(self, offset, val)
    }

    fn try_enlarge(&mut self, wanted: usize) -> core::result::Result<(), EnlargeError> {
        use crate::utils::ByteVecExt;
        self.try_extend_zeroed(wanted).map_err(EnlargeError::from)
    }
}

macro_rules! impl_buffer_ref_for_wrappers {
    ($($type:ty),*) => {$(
        impl<T: ?Sized + BufferRef> BufferRef for $type {
            fn len(&self) -> usize {
                T::len(self)
            }

            fn read<const N: usize>(&self, offset: usize) -> &[u8; N] {
                T::read(self, offset)
            }
        }
    )*};
}

impl_buffer_ref_for_wrappers!(&T, &mut T, Box<T>, std::rc::Rc<T>, std::sync::Arc<T>);

macro_rules! impl_buffer_mut_for_wrappers {
    ($($type:ty),*) => {$(
        impl<T: ?Sized + BufferMut> BufferMut for $type {
            fn capacity(&self) -> usize {
                T::capacity(self)
            }

            fn write<const N: usize>(&mut self, offset: usize, val: &[u8; N]) {
                T::write(self, offset, val)
            }

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
            assert!(matches!(err.source(), None));
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
        assert!(matches!(err.source(), None));

        assert_eq!(format!("{}", err.clone()), "could not enlarge buffer");

        assert_eq!(format!("{:?}", err.clone()), "EnlargeError");
    }
}

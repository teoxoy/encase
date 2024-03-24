use core::mem::MaybeUninit;

#[track_caller]
pub const fn consume_zsts<const N: usize>(_: [(); N]) {}

#[doc(hidden)]
#[macro_export]
macro_rules! build_struct {
    ($type:ty, $( $field_idents:ident ),*) => {{
        let mut uninit_struct = ::core::mem::MaybeUninit::<$type>::uninit();

        let ptr = ::core::mem::MaybeUninit::as_mut_ptr(&mut uninit_struct);

        $( $crate::build_struct!(__write_to_field; ptr, $field_idents, $field_idents); )*

        // SAFETY: Everything has been initialized
        unsafe { ::core::mem::MaybeUninit::assume_init(uninit_struct) }
    }};

    (__write_to_field; $ptr:ident, $field_name:ident, $data:expr) => {
        // SAFETY: the pointer `ptr` returned by `as_mut_ptr` is a valid pointer,
        // so it's safe to get a pointer to a field through `addr_of_mut!`
        let field_ptr = unsafe { ::core::ptr::addr_of_mut!((*$ptr).$field_name) };
        // SAFETY: writing to `field_ptr` is safe because it's a pointer
        // to one of the struct's fields (therefore valid and aligned)
        unsafe { field_ptr.write($data) };
    };
}

#[cfg(any(feature = "glam", feature = "ultraviolet", feature = "vek"))]
macro_rules! array_ref_to_2d_array_ref {
    ($array:expr, $ty:ty, $c:literal, $r:literal) => {
        // SAFETY:
        // transmuting from &[T; R * C] to &[[T; R]; C] is sound since:
        //  the references have the same size
        //   size_of::<&[T; R * C]>()                           = size_of::<usize>()
        //   size_of::<&[[T; R]; C]>()                          = size_of::<usize>()
        //  the values behind the references have the same size and alignment
        //   size_of::<[T; R * C]>()                            = size_of::<T>() * R * C
        //   size_of::<[[T; R]; C]>() = size_of::<[T; R]>() * C = size_of::<T>() * R * C
        //   align_of::<[T; R * C]>()                           = align_of::<T>()
        //   align_of::<[[T; R]; C]>() = align_of::<[T; R]>()   = align_of::<T>()
        // ref: https://doc.rust-lang.org/reference/type-layout.html
        unsafe { ::core::mem::transmute::<&[$ty; $r * $c], &[[$ty; $r]; $c]>($array) }
    };
}

#[cfg(any(feature = "glam", feature = "ultraviolet", feature = "vek"))]
macro_rules! array_mut_to_2d_array_mut {
    ($array:expr, $ty:ty, $c:literal, $r:literal) => {
        // SAFETY:
        // transmuting from &mut [T; R * C] to &mut [[T; R]; C] is sound since:
        //  the references have the same size
        //   size_of::<&mut [T; R * C]>()                       = size_of::<usize>()
        //   size_of::<&mut [[T; R]; C]>()                      = size_of::<usize>()
        //  the values behind the references have the same size and alignment
        //   size_of::<[T; R * C]>()                            = size_of::<T>() * R * C
        //   size_of::<[[T; R]; C]>() = size_of::<[T; R]>() * C = size_of::<T>() * R * C
        //   align_of::<[T; R * C]>()                           = align_of::<T>()
        //   align_of::<[[T; R]; C]>() = align_of::<[T; R]>()   = align_of::<T>()
        // ref: https://doc.rust-lang.org/reference/type-layout.html
        unsafe { ::core::mem::transmute::<&mut [$ty; $r * $c], &mut [[$ty; $r]; $c]>($array) }
    };
}

pub(crate) trait ByteVecExt {
    /// Tries to extend `self` with `0`s up to `new_len`, using memset.
    fn try_extend(&mut self, new_len: usize) -> Result<(), std::collections::TryReserveError>;
}

impl ByteVecExt for Vec<u8> {
    fn try_extend(&mut self, new_len: usize) -> Result<(), std::collections::TryReserveError> {
        let additional = new_len.saturating_sub(self.len());
        if additional > 0 {
            self.try_reserve(additional)?;

            let end = unsafe { self.as_mut_ptr().add(self.len()) };
            // SAFETY
            // 1. dst ptr is valid for writes of count * size_of::<T>() bytes since the call to Vec::reserve() succeeded
            // 2. dst ptr is properly aligned since we got it via Vec::as_mut_ptr_range()
            unsafe { end.write_bytes(u8::MIN, additional) }
            // SAFETY
            // 1. new_len is less than or equal to Vec::capacity() since we reserved at least `additional` elements
            // 2. The elements at old_len..new_len are initialized since we wrote `additional` bytes
            unsafe { self.set_len(new_len) }
        }
        Ok(())
    }
}

impl<T> ByteVecExt for Vec<MaybeUninit<T>> {
    fn try_extend(&mut self, new_len: usize) -> Result<(), std::collections::TryReserveError> {
        let additional = new_len.saturating_sub(self.len());
        if additional > 0 {
            self.try_reserve(additional)?;

            // It's OK to not initialize the extended elements as MaybeUninit allows
            // uninitialized memory.

            // SAFETY
            // 1. new_len is less than or equal to Vec::capacity() since we reserved at least `additional` elements
            // 2. The elements at old_len..new_len are initialized since we wrote `additional` bytes
            // 3. MaybeUninit
            unsafe { self.set_len(new_len) }
        }
        Ok(())
    }
}

pub(crate) trait SliceExt<T> {
    /// Returns a "window" (shared reference to an array of length `N`) into this slice.
    ///
    /// # Panics
    ///
    /// Panics if the range `offset..offset + N` is out of bounds.
    fn array<const N: usize>(&self, offset: usize) -> &[T; N];

    /// Returns a "window" (mutable reference to an array of length `N`) into this slice.
    ///
    /// # Panics
    ///
    /// Panics if the range `offset..offset + N` is out of bounds.
    fn array_mut<const N: usize>(&mut self, offset: usize) -> &mut [T; N];
}

impl<T> SliceExt<T> for [T] {
    // from rust core lib https://github.com/rust-lang/rust/blob/11d96b59307b1702fffe871bfc2d0145d070881e/library/core/src/slice/mod.rs#L1794
    // track #![feature(split_array)] (https://github.com/rust-lang/rust/issues/90091)

    #[inline]
    fn array<const N: usize>(&self, offset: usize) -> &[T; N] {
        let src = &self[offset..offset + N];

        // SAFETY
        // casting to &[T; N] is safe since src is a &[T] of length N
        unsafe { &*(src.as_ptr() as *const [T; N]) }
    }

    // from rust core lib https://github.com/rust-lang/rust/blob/11d96b59307b1702fffe871bfc2d0145d070881e/library/core/src/slice/mod.rs#L1827
    // track #![feature(split_array)] (https://github.com/rust-lang/rust/issues/90091)

    #[inline]
    fn array_mut<const N: usize>(&mut self, offset: usize) -> &mut [T; N] {
        let src = &mut self[offset..offset + N];

        // SAFETY
        // casting to &mut [T; N] is safe since src is a &mut [T] of length N
        unsafe { &mut *(src.as_mut_ptr() as *mut [T; N]) }
    }
}

#[cfg(test)]
mod byte_vec_ext {
    use crate::utils::ByteVecExt;

    #[test]
    fn try_extend() {
        let mut vec: Vec<u8> = Vec::new();

        vec.try_extend(10).unwrap();

        assert_eq!(vec.len(), 10);
        assert!(vec.iter().all(|val| *val == 0));
    }

    #[test]
    fn try_extend_noop() {
        let mut vec = vec![0; 12];

        vec.try_extend(10).unwrap();

        assert_eq!(vec.len(), 12);
    }

    #[test]
    fn try_extend_err() {
        let mut vec = vec![0; 12];

        assert!(vec.try_extend(usize::MAX).is_err());
    }
}

#[cfg(test)]
mod slice_ext {
    use crate::utils::SliceExt;

    #[test]
    fn array() {
        let arr = [1, 3, 7, 6, 9, 7];
        let slice = arr.as_ref();

        let sub_arr: &[i32; 2] = slice.array(3);

        assert_eq!(sub_arr, &[6, 9]);
    }

    #[test]
    fn array_mut() {
        let mut arr = [1, 3, 7, 6, 9, 7];
        let slice = arr.as_mut();

        let sub_arr: &mut [i32; 2] = slice.array_mut(3);

        assert_eq!(sub_arr, &mut [6, 9]);
    }
}

#[track_caller]
#[cfg_attr(coverage, no_coverage)]
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
    fn try_extend_zeroed(
        &mut self,
        new_len: usize,
    ) -> Result<(), std::collections::TryReserveError>;
}

impl ByteVecExt for Vec<u8> {
    fn try_extend_zeroed(
        &mut self,
        new_len: usize,
    ) -> Result<(), std::collections::TryReserveError> {
        let additional = new_len.saturating_sub(self.len());
        if additional > 0 {
            self.try_reserve(additional)?;

            let end = self.as_mut_ptr_range().end;
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

pub trait ArrayExt<T, const N: usize> {
    /// Copies all elements from `src` into `self`, using memcpy.
    fn copy_from(&mut self, src: &Self)
    where
        T: Copy;

    /// Creates an array `[T; N]` where each array element `T` is returned by the `cb` call.
    ///
    /// # Arguments
    ///
    /// * `cb`: Callback where the passed argument is the current array index.
    fn from_fn<F>(cb: F) -> Self
    where
        Self: Sized,
        F: FnMut(usize) -> T;
}

impl<T, const N: usize> ArrayExt<T, N> for [T; N] {
    fn copy_from(&mut self, src: &Self)
    where
        T: Copy,
    {
        // SAFETY
        // 1. src is valid for reads of count * size_of::<T>() bytes
        //      since it's a shared pointer to an array with count elements
        // 2. dst is valid for writes of count * size_of::<T>() bytes
        //      since it's a mutable pointer to an array with count elements
        // 3. Both src and dst are properly aligned
        //      since they are both pointers to arrays with the same element type T
        // 4. The region of memory beginning at src with a size of count * size_of::<T>() bytes
        // does not overlap with the region of memory beginning at dst with the same size
        //      since dst is a mutable reference (therefore exclusive)
        unsafe {
            std::ptr::copy_nonoverlapping(src.as_ptr(), self.as_mut_ptr(), N);
        }
    }

    // from rust core lib https://github.com/rust-lang/rust/blob/d5a9bc947617babe3833458f3e09f3d6d5e3d736/library/core/src/array/mod.rs#L41
    // track #![feature(array_from_fn)] (https://github.com/rust-lang/rust/issues/89379)

    fn from_fn<F>(mut cb: F) -> Self
    where
        F: FnMut(usize) -> T,
    {
        let mut idx = 0;
        [(); N].map(|_| {
            let res = cb(idx);
            idx += 1;
            res
        })
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

    /// Copies all elements from `src` into `self`, using memcpy.
    ///
    /// # Panics
    ///
    /// Panics if the range `offset..offset + N` is out of bounds.
    fn copy_from_array<const N: usize>(&mut self, offset: usize, src: &[T; N])
    where
        T: Copy;
}

impl<T> SliceExt<T> for [T] {
    fn array<const N: usize>(&self, offset: usize) -> &[T; N] {
        let src = &self[offset..offset + N];

        // SAFETY
        // casting to &[T; N] is safe since src is a &[T] of length N
        unsafe { &*(src.as_ptr() as *const [T; N]) }
    }

    fn array_mut<const N: usize>(&mut self, offset: usize) -> &mut [T; N] {
        let src = &mut self[offset..offset + N];

        // SAFETY
        // casting to &mut [T; N] is safe since src is a &mut [T] of length N
        unsafe { &mut *(src.as_mut_ptr() as *mut [T; N]) }
    }

    fn copy_from_array<const N: usize>(&mut self, offset: usize, src: &[T; N])
    where
        T: Copy,
    {
        let dst = self.array_mut(offset);
        dst.copy_from(src);
    }
}

#[cfg(test)]
mod byte_vec_ext {
    use crate::utils::ByteVecExt;

    #[test]
    fn try_extend_zeroed() {
        let mut vec = Vec::new();

        vec.try_extend_zeroed(10).unwrap();

        assert_eq!(vec.len(), 10);
        assert!(vec.iter().all(|val| *val == 0));
    }

    #[test]
    fn try_extend_zeroed_noop() {
        let mut vec = vec![0; 12];

        vec.try_extend_zeroed(10).unwrap();

        assert_eq!(vec.len(), 12);
    }

    #[test]
    fn try_extend_zeroed_err() {
        let mut vec = vec![0; 12];

        assert!(matches!(vec.try_extend_zeroed(usize::MAX), Err(_)));
    }
}

#[cfg(test)]
mod array_ext {
    use crate::utils::ArrayExt;

    #[test]
    fn copy_from() {
        let src = [1, 3, 7, 6];
        let mut dst = [0; 4];

        assert_ne!(src, dst);

        dst.copy_from(&src);

        assert_eq!(src, dst);
    }

    #[test]
    fn from_fn() {
        let arr: [usize; 5] = ArrayExt::from_fn(|i| i);

        assert_eq!(arr, [0, 1, 2, 3, 4]);
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

    #[test]
    fn copy_from_array() {
        let src = [1, 3, 7];
        let mut arr = [0; 6];
        let slice = arr.as_mut();

        slice.copy_from_array(3, &src);

        assert_eq!(arr, [0, 0, 0, 1, 3, 7]);
    }
}

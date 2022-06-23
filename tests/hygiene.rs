#![no_implicit_prelude]
#![allow(non_camel_case_types)]

macro_rules! decl_primitives_as_traits {
    ($($primitive:ident),*) => {$(trait $primitive {})*};
}

// from core::primitive
decl_primitives_as_traits!(
    bool, char, f32, f64, i128, i16, i32, i64, i8, isize, str, u128, u16, u32, u64, u8, usize
);

mod impl_vector {
    use ::core::{
        convert::{AsMut, AsRef, From},
        marker::PhantomData,
        unimplemented,
    };

    pub struct Test<'a, T> {
        data: PhantomData<&'a T>,
    }

    impl<'a, T, const N: usize> AsRef<[T; N]> for Test<'a, T> {
        fn as_ref(&self) -> &[T; N] {
            unimplemented!()
        }
    }
    impl<'a, T, const N: usize> AsMut<[T; N]> for Test<'a, T> {
        fn as_mut(&mut self) -> &mut [T; N] {
            unimplemented!()
        }
    }
    impl<'a, T, const N: usize> From<[T; N]> for Test<'a, T> {
        fn from(_: [T; N]) -> Self {
            unimplemented!()
        }
    }
}

::encase::impl_vector!(2, impl_vector::Test<'a, T>; ('a, T: 'a); using AsRef AsMut From);

mod impl_matrix {
    use ::core::{
        convert::{AsMut, AsRef, From},
        marker::PhantomData,
        unimplemented,
    };

    pub struct Test<'a, T> {
        data: PhantomData<&'a T>,
    }

    impl<'a, T, const N: usize, const M: usize> AsRef<[[T; M]; N]> for Test<'a, T> {
        fn as_ref(&self) -> &[[T; M]; N] {
            unimplemented!()
        }
    }
    impl<'a, T, const N: usize, const M: usize> AsMut<[[T; M]; N]> for Test<'a, T> {
        fn as_mut(&mut self) -> &mut [[T; M]; N] {
            unimplemented!()
        }
    }
    impl<'a, T, const N: usize, const M: usize> From<[[T; M]; N]> for Test<'a, T> {
        fn from(_: [[T; M]; N]) -> Self {
            unimplemented!()
        }
    }
}

::encase::impl_matrix!(2, 2, impl_matrix::Test<'a, T>; ('a, T: 'a); using AsRef AsMut From);

mod impl_rts_array {
    use ::core::{marker::PhantomData, unimplemented};

    pub trait Array {
        type Item;
    }

    pub struct Test<A: Array> {
        data: PhantomData<A>,
    }

    impl<A: Array> Test<A> {
        pub fn len(&self) -> usize {
            unimplemented!()
        }

        pub fn truncate(&mut self, _len: usize) {
            unimplemented!()
        }
    }
}

::encase::impl_rts_array!(impl_rts_array::Test<A>; (T, A: impl_rts_array::Array<Item = T>); using len truncate);

#[derive(::encase::ShaderType)]
struct Test {
    a: [::mint::Vector3<::core::primitive::f32>; 2],
    b: ::core::primitive::u32,
}

#[derive(::encase::ShaderType)]
struct TestGeneric<
    'a,
    T: 'a + ::encase::ShaderType + ::encase::ShaderSize,
    const N: ::core::primitive::usize,
> {
    #[size(90)]
    a: &'a mut Test,
    b: &'a mut [T; N],
    #[align(16)]
    #[size(runtime)]
    c: &'a mut ::std::vec::Vec<[::mint::Vector3<::core::primitive::f32>; 2]>,
}

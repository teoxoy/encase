/// Used to implement `ShaderType` for the given wrapper type
///
/// # Args
///
/// - `$type` the type (representing a wrapper) for which `ShaderType` will be imeplemented for
///
/// - `$generics` \[optional\] generics that will be passed into the `impl< >`
///
/// - `$using` \[optional\] can be any combination of `Ref{ X } Mut{ X } From{ X }`
/// (where `X` denotes a possible function call)
#[macro_export]
macro_rules! impl_wrapper {
    ($type:ty; using $($using:tt)*) => {
        $crate::impl_wrapper_inner!(__inner, ($type, T: ?Sized); $($using)*);
    };
    ($type:ty; ($($generics:tt)*); using $($using:tt)*) => {
        $crate::impl_wrapper_inner!(__inner, ($type, $($generics)*); $($using)*);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_wrapper_inner {
    (__inner, ($($other:tt)*); Ref{ $($get_ref:tt)* } $($using:tt)*) => {
        $crate::impl_wrapper_inner!(__ref, ($($other)*); { $($get_ref)* });
        $crate::impl_wrapper_inner!(__inner, ($($other)*); $($using)*);
    };
    (__inner, ($($other:tt)*); Mut{ $($get_mut:tt)* } $($using:tt)*) => {
        $crate::impl_wrapper_inner!(__mut, ($($other)*); { $($get_mut)* });
        $crate::impl_wrapper_inner!(__inner, ($($other)*); $($using)*);
    };
    (__inner, ($($other:tt)*); From{ $($from:tt)* } $($using:tt)*) => {
        $crate::impl_wrapper_inner!(__from, ($($other)*); { $($from)* });
        $crate::impl_wrapper_inner!(__inner, ($($other)*); $($using)*);
    };
    (__inner, ($type:ty, $($generics:tt)*); ) => {};

    (__ref, ($type:ty, $($generics:tt)*); { $($get_ref:tt)* }) => {
        impl<$($generics)*> $crate::private::ShaderType for $type
        where
            T: $crate::private::ShaderType
        {
            type ExtraMetadata = T::ExtraMetadata;
            const METADATA: $crate::private::Metadata<Self::ExtraMetadata> = T::METADATA;

            const UNIFORM_COMPAT_ASSERT: fn() = T::UNIFORM_COMPAT_ASSERT;

            fn size(&self) -> ::core::num::NonZeroU64 {
                <T as $crate::private::ShaderType>::size(&self$($get_ref)*)
            }
        }
        impl<$($generics)*> $crate::private::ShaderSize for $type
        where
            T: $crate::private::ShaderSize
        {
            const SHADER_SIZE: ::core::num::NonZeroU64 = T::SHADER_SIZE;
        }

        impl<$($generics)*> $crate::private::RuntimeSizedArray for $type
        where
            T: $crate::private::RuntimeSizedArray
        {
            fn len(&self) -> usize {
                <T as $crate::private::RuntimeSizedArray>::len(&self$($get_ref)*)
            }
        }

        impl<$($generics)*> $crate::private::CalculateSizeFor for $type
        where
            T: $crate::private::CalculateSizeFor
        {
            fn calculate_size_for(nr_of_el: u64) -> ::core::num::NonZeroU64 {
                <T as $crate::private::CalculateSizeFor>::calculate_size_for(nr_of_el)
            }
        }

        impl<$($generics)*> $crate::private::WriteInto for $type
        where
            T: $crate::private::WriteInto
        {
            fn write_into<B: $crate::private::BufferMut>(&self, writer: &mut $crate::private::Writer<B>) {
                <T as $crate::private::WriteInto>::write_into(&self$($get_ref)*, writer)
            }
        }
    };
    (__mut, ($type:ty, $($generics:tt)*); { $($get_mut:tt)* }) => {
        impl<$($generics)*> $crate::private::ReadFrom for $type
        where
            T: $crate::private::ReadFrom
        {
            fn read_from<B: $crate::private::BufferRef>(&mut self, reader: &mut $crate::private::Reader<B>) {
                <T as $crate::private::ReadFrom>::read_from(self$($get_mut)*, reader)
            }
        }
    };
    (__from, ($type:ty, $($generics:tt)*); { $($from:tt)* }) => {
        impl<$($generics)*> $crate::private::CreateFrom for $type
        where
            T: $crate::private::CreateFrom
        {
            fn create_from<B: $crate::private::BufferRef>(reader: &mut $crate::private::Reader<B>) -> Self {
                <$type>::$($from)*(<T as $crate::private::CreateFrom>::create_from(reader))
            }
        }
    };
}

impl_wrapper!(&T; using Ref{});
impl_wrapper!(&mut T; using Ref{} Mut{});
impl_wrapper!(Box<T>; using Ref{} Mut{} From{ new });
impl_wrapper!(std::borrow::Cow<'_, T>; (T: ?Sized + ToOwned<Owned = T>); using Ref{} From{ Owned });
impl_wrapper!(std::rc::Rc<T>; using Ref{} From{ new });
impl_wrapper!(std::sync::Arc<T>; using Ref{} From{ new });
impl_wrapper!(core::cell::Cell<T>; (T: ?Sized + Copy); using Ref{ .get() } Mut{ .get_mut() } From{ new });

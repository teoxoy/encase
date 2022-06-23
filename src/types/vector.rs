pub trait VectorScalar {}
impl_marker_trait_for_f32!(VectorScalar);
impl_marker_trait_for_u32!(VectorScalar);
impl_marker_trait_for_i32!(VectorScalar);

/// Enables reading from the vector (via `&[T; N]`)
pub trait AsRefVectorParts<T: VectorScalar, const N: usize> {
    fn as_ref_parts(&self) -> &[T; N];
}

/// Enables writing to the vector (via `&mut [T; N]`)
pub trait AsMutVectorParts<T: VectorScalar, const N: usize> {
    fn as_mut_parts(&mut self) -> &mut [T; N];
}

/// Enables the cration of a vector (via `[T; N]`)
pub trait FromVectorParts<T: VectorScalar, const N: usize> {
    fn from_parts(parts: [T; N]) -> Self;
}

/// Used to implement `ShaderType` for the given vector type
///
/// The given vector type should implement any combination of
/// [`AsRefVectorParts`], [`AsMutVectorParts`], [`FromVectorParts`]
/// depending on needed capability (they can also be derived via `$using`)
///
/// # Args
///
/// - `$n` nr of elements the given vector contains
///
/// - `$type` the type (representing a vector) for which `ShaderType` will be imeplemented for
///
/// - `$generics` \[optional\] generics that will be passed into the `impl< >`
///
/// - `$el_type` \[optional\] inner element type of the vector (should implement [`VectorScalar`])
///
/// - `$using` \[optional\] can be any combination of `AsRef AsMut From`
#[macro_export]
macro_rules! impl_vector {
    ($n:literal, $type:ty $( ; using $($using:tt)* )?) => {
        $crate::impl_vector_inner!(__inner, ($n, $type, T, (T)); $( $($using)* )?);
    };
    ($n:literal, $type:ty; ($($generics:tt)*) $( ; using $($using:tt)* )?) => {
        $crate::impl_vector_inner!(__inner, ($n, $type, T, ($($generics)*)); $( $($using)* )?);
    };
    ($n:literal, $type:ty, $el_ty:ty $( ; using $($using:tt)* )?) => {
        $crate::impl_vector_inner!(__inner, ($n, $type, $el_ty, ()); $( $($using)* )?);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_vector_inner {
    (__inner, ($($other:tt)*); AsRef $($using:tt)*) => {
        $crate::impl_vector_inner!(__ref, $($other)*);
        $crate::impl_vector_inner!(__inner, ($($other)*); $($using)*);
    };
    (__inner, ($($other:tt)*); AsMut $($using:tt)*) => {
        $crate::impl_vector_inner!(__mut, $($other)*);
        $crate::impl_vector_inner!(__inner, ($($other)*); $($using)*);
    };
    (__inner, ($($other:tt)*); From $($using:tt)*) => {
        $crate::impl_vector_inner!(__from, $($other)*);
        $crate::impl_vector_inner!(__inner, ($($other)*); $($using)*);
    };
    (__inner, ($n:literal, $type:ty, $el_ty:ty, ($($generics:tt)*)); ) => {
        $crate::impl_vector_inner!(__main, $n, $type, $el_ty, ($($generics)*));
    };

    (__ref, $n:literal, $type:ty, $el_ty:ty, ($($generics:tt)*)) => {
        impl<$($generics)*> $crate::private::AsRefVectorParts<$el_ty, $n> for $type
        where
            Self: ::core::convert::AsRef<[$el_ty; $n]>,
            $el_ty: $crate::private::VectorScalar,
        {
            fn as_ref_parts(&self) -> &[$el_ty; $n] {
                ::core::convert::AsRef::as_ref(self)
            }
        }
    };
    (__mut, $n:literal, $type:ty, $el_ty:ty, ($($generics:tt)*)) => {
        impl<$($generics)*> $crate::private::AsMutVectorParts<$el_ty, $n> for $type
        where
            Self: ::core::convert::AsMut<[$el_ty; $n]>,
            $el_ty: $crate::private::VectorScalar,
        {
            fn as_mut_parts(&mut self) -> &mut [$el_ty; $n] {
                ::core::convert::AsMut::as_mut(self)
            }
        }
    };
    (__from, $n:literal, $type:ty, $el_ty:ty, ($($generics:tt)*)) => {
        impl<$($generics)*> $crate::private::FromVectorParts<$el_ty, $n> for $type
        where
            Self: ::core::convert::From<[$el_ty; $n]>,
            $el_ty: $crate::private::VectorScalar,
        {
            fn from_parts(parts: [$el_ty; $n]) -> Self {
                ::core::convert::From::from(parts)
            }
        }
    };

    (__main, $n:literal, $type:ty, $el_ty:ty, ($($generics:tt)*)) => {
        const _: () = assert!(
            2 <= $n && $n <= 4,
            "Vector should have at least 2 elements and at most 4!",
        );

        impl<$($generics)*> $crate::private::ShaderType for $type
        where
            $el_ty: $crate::private::ShaderSize,
        {
            type ExtraMetadata = ();
            const METADATA: $crate::private::Metadata<Self::ExtraMetadata> = {
                let size = $crate::private::SizeValue::from(<$el_ty as $crate::private::ShaderSize>::SHADER_SIZE).mul($n);
                let alignment = $crate::private::AlignmentValue::from_next_power_of_two_size(size);

                $crate::private::Metadata {
                    alignment,
                    has_uniform_min_alignment: false,
                    min_size: size,
                    extra: ()
                }
            };
        }

        impl<$($generics)*> $crate::private::ShaderSize for $type
        where
            $el_ty: $crate::private::ShaderSize
        {}

        impl<$($generics)*> $crate::private::WriteInto for $type
        where
            Self: $crate::private::AsRefVectorParts<$el_ty, $n>,
            $el_ty: $crate::private::VectorScalar + $crate::private::WriteInto,
        {
            fn write_into<B: $crate::private::BufferMut>(&self, writer: &mut $crate::private::Writer<B>) {
                let elements = $crate::private::AsRefVectorParts::<$el_ty, $n>::as_ref_parts(self);
                for el in elements {
                    $crate::private::WriteInto::write_into(el, writer);
                }
            }
        }

        impl<$($generics)*> $crate::private::ReadFrom for $type
        where
            Self: $crate::private::AsMutVectorParts<$el_ty, $n>,
            $el_ty: $crate::private::VectorScalar + $crate::private::ReadFrom,
        {
            fn read_from<B: $crate::private::BufferRef>(&mut self, reader: &mut $crate::private::Reader<B>) {
                let elements = $crate::private::AsMutVectorParts::<$el_ty, $n>::as_mut_parts(self);
                for el in elements {
                    $crate::private::ReadFrom::read_from(el, reader);
                }
            }
        }

        impl<$($generics)*> $crate::private::CreateFrom for $type
        where
            Self: $crate::private::FromVectorParts<$el_ty, $n>,
            $el_ty: $crate::private::VectorScalar + $crate::private::CreateFrom,
        {
            fn create_from<B: $crate::private::BufferRef>(reader: &mut $crate::private::Reader<B>) -> Self {
                let elements = $crate::private::ArrayExt::from_fn(|_| {
                    $crate::private::CreateFrom::create_from(reader)
                });

                $crate::private::FromVectorParts::<$el_ty, $n>::from_parts(elements)
            }
        }
    };
}

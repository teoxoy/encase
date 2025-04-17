use crate::core::Metadata;

pub trait MatrixScalar: crate::ShaderSize {}
impl_marker_trait_for_f32!(MatrixScalar);

pub struct MatrixMetadata {
    pub col_padding: u64,
}

impl Metadata<MatrixMetadata> {
    #[inline]
    pub const fn col_padding(self) -> u64 {
        self.extra.col_padding
    }
}

/// Enables reading from the matrix (via `&[[T; R]; C]`)
pub trait AsRefMatrixParts<T: MatrixScalar, const C: usize, const R: usize> {
    fn as_ref_parts(&self) -> &[[T; R]; C];
}

/// Enables writing to the matrix (via `&mut [[T; R]; C]`)
pub trait AsMutMatrixParts<T: MatrixScalar, const C: usize, const R: usize> {
    fn as_mut_parts(&mut self) -> &mut [[T; R]; C];
}

/// Enables the creation of a matrix (via `[[T; R]; C]`)
pub trait FromMatrixParts<T: MatrixScalar, const C: usize, const R: usize> {
    fn from_parts(parts: [[T; R]; C]) -> Self;
}

/// Used to implement `ShaderType` for the given matrix type
///
/// The given matrix type should implement any combination of
/// [`AsRefMatrixParts`], [`AsMutMatrixParts`], [`FromMatrixParts`]
/// depending on needed capability (they can also be derived via `$using`)
///
/// # Args
///
/// - `$c` nr of columns the given matrix contains
///
/// - `$r` nr of rows the given matrix contains
///
/// - `$type` the type (representing a matrix) for which `ShaderType` will be implemented for
///
/// - `$generics` \[optional\] generics that will be passed into the `impl< >`
///
/// - `$el_type` \[optional\] inner element type of the matrix (should implement [`MatrixScalar`])
///
/// - `$using` \[optional\] can be any combination of `AsRef AsMut From`
#[macro_export]
macro_rules! impl_matrix {
    ($c:literal, $r:literal, $type:ty $( ; using $($using:tt)* )?) => {
        $crate::impl_matrix_inner!(__inner, ($c, $r, $type, T, (T)); $( $($using)* )?);
    };
    ($c:literal, $r:literal, $type:ty; ($($generics:tt)*) $( ; using $($using:tt)* )?) => {
        $crate::impl_matrix_inner!(__inner, ($c, $r, $type, T, ($($generics)*)); $( $($using)* )?);
    };
    ($c:literal, $r:literal, $type:ty, $el_ty:ty $( ; using $($using:tt)* )?) => {
        $crate::impl_matrix_inner!(__inner, ($c, $r, $type, $el_ty, ()); $( $($using)* )?);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_matrix_inner {
    (__inner, ($($other:tt)*); AsRef $($using:tt)*) => {
        $crate::impl_matrix_inner!(__ref, $($other)*);
        $crate::impl_matrix_inner!(__inner, ($($other)*); $($using)*);
    };
    (__inner, ($($other:tt)*); AsMut $($using:tt)*) => {
        $crate::impl_matrix_inner!(__mut, $($other)*);
        $crate::impl_matrix_inner!(__inner, ($($other)*); $($using)*);
    };
    (__inner, ($($other:tt)*); From $($using:tt)*) => {
        $crate::impl_matrix_inner!(__from, $($other)*);
        $crate::impl_matrix_inner!(__inner, ($($other)*); $($using)*);
    };
    (__inner, ($c:literal, $r:literal, $type:ty, $el_ty:ty, ($($generics:tt)*)); ) => {
        $crate::impl_matrix_inner!(__main, $c, $r, $type, $el_ty, ($($generics)*));
    };

    (__ref, $c:literal, $r:literal, $type:ty, $el_ty:ty, ($($generics:tt)*)) => {
        impl<$($generics)*> $crate::private::AsRefMatrixParts<$el_ty, $c, $r> for $type
        where
            Self: ::core::convert::AsRef<[[$el_ty; $r]; $c]>,
            $el_ty: $crate::private::MatrixScalar,
        {
            #[inline]
            fn as_ref_parts(&self) -> &[[$el_ty; $r]; $c] {
                ::core::convert::AsRef::as_ref(self)
            }
        }
    };
    (__mut, $c:literal, $r:literal, $type:ty, $el_ty:ty, ($($generics:tt)*)) => {
        impl<$($generics)*> $crate::private::AsMutMatrixParts<$el_ty, $c, $r> for $type
        where
            Self: ::core::convert::AsMut<[[$el_ty; $r]; $c]>,
            $el_ty: $crate::private::MatrixScalar,
        {
            #[inline]
            fn as_mut_parts(&mut self) -> &mut [[$el_ty; $r]; $c] {
                ::core::convert::AsMut::as_mut(self)
            }
        }
    };
    (__from, $c:literal, $r:literal, $type:ty, $el_ty:ty, ($($generics:tt)*)) => {
        impl<$($generics)*> $crate::private::FromMatrixParts<$el_ty, $c, $r> for $type
        where
            Self: ::core::convert::From<[[$el_ty; $r]; $c]>,
            $el_ty: $crate::private::MatrixScalar,
        {
            #[inline]
            fn from_parts(parts: [[$el_ty; $r]; $c]) -> Self {
                ::core::convert::From::from(parts)
            }
        }
    };

    (__main, $c:literal, $r:literal, $type:ty, $el_ty:ty, ($($generics:tt)*)) => {
        const _: () = assert!(
            2 <= $c && $c <= 4,
            "Matrix should have at least 2 columns and at most 4!",
        );
        const _: () = assert!(
            2 <= $r && $r <= 4,
            "Matrix should have at least 2 rows and at most 4!",
        );

        impl<$($generics)*> $crate::private::ShaderType for $type
        where
            $el_ty: $crate::private::ShaderSize,
        {
            type ExtraMetadata = $crate::private::MatrixMetadata;
            const METADATA: $crate::private::Metadata<Self::ExtraMetadata> = {
                let col_size = $crate::private::SizeValue::from(<$el_ty as $crate::private::ShaderSize>::SHADER_SIZE).mul($r);
                let alignment = $crate::private::AlignmentValue::from_next_power_of_two_size(col_size);
                let size = alignment.round_up_size(col_size).mul($c);
                let col_padding = alignment.padding_needed_for(col_size.get());

                $crate::private::Metadata {
                    alignment,
                    has_uniform_min_alignment: false,
                    min_size: size,
                    is_pod: <[$el_ty; $r] as $crate::private::ShaderType>::METADATA.is_pod() && col_padding == 0,
                    extra: $crate::private::MatrixMetadata {
                        col_padding,
                    },
                }
            };
        }

        impl<$($generics)*> $crate::private::ShaderSize for $type
        where
            $el_ty: $crate::private::ShaderSize
        {}

        impl<$($generics)*> $crate::private::WriteInto for $type
        where
            Self: $crate::private::AsRefMatrixParts<$el_ty, $c, $r> + $crate::private::ShaderType<ExtraMetadata = $crate::private::MatrixMetadata>,
            $el_ty: $crate::private::MatrixScalar + $crate::private::WriteInto,
        {
            #[inline]
            fn write_into<B: $crate::private::BufferMut>(&self, writer: &mut $crate::private::Writer<B>) {
                let columns = $crate::private::AsRefMatrixParts::<$el_ty, $c, $r>::as_ref_parts(self);

                $crate::if_pod_and_little_endian!(if pod_and_little_endian {
                    $crate::private::WriteInto::write_into(columns, writer);
                } else {
                    for col in columns {
                        $crate::private::WriteInto::write_into(col, writer);
                        writer.advance(<Self as $crate::private::ShaderType>::METADATA.col_padding() as ::core::primitive::usize);
                    }
                });
            }
        }

        impl<$($generics)*> $crate::private::ReadFrom for $type
        where
            Self: $crate::private::AsMutMatrixParts<$el_ty, $c, $r> + $crate::private::ShaderType<ExtraMetadata = $crate::private::MatrixMetadata>,
            $el_ty: $crate::private::MatrixScalar + $crate::private::ReadFrom,
        {
            #[inline]
            fn read_from<B: $crate::private::BufferRef>(&mut self, reader: &mut $crate::private::Reader<B>) {
                let columns = $crate::private::AsMutMatrixParts::<$el_ty, $c, $r>::as_mut_parts(self);

                $crate::if_pod_and_little_endian!(if pod_and_little_endian {
                    $crate::private::ReadFrom::read_from(columns, reader);
                } else {
                    for col in columns {
                        $crate::private::ReadFrom::read_from(col, reader);
                        reader.advance(<Self as $crate::private::ShaderType>::METADATA.col_padding() as ::core::primitive::usize);
                    }
                });
            }
        }

        impl<$($generics)*> $crate::private::CreateFrom for $type
        where
            Self: $crate::private::FromMatrixParts<$el_ty, $c, $r> + $crate::private::ShaderType<ExtraMetadata = $crate::private::MatrixMetadata>,
            $el_ty: $crate::private::MatrixScalar + $crate::private::CreateFrom,
        {
            #[inline]
            fn create_from<B: $crate::private::BufferRef>(reader: &mut $crate::private::Reader<B>) -> Self {
                let columns = $crate::if_pod_and_little_endian!(if pod_and_little_endian {
                    $crate::private::CreateFrom::create_from(reader)
                } else {
                    ::core::array::from_fn(|_| {
                        let col = $crate::private::CreateFrom::create_from(reader);
                        reader.advance(<Self as $crate::private::ShaderType>::METADATA.col_padding() as ::core::primitive::usize);
                        col
                    })
                });
                $crate::private::FromMatrixParts::<$el_ty, $c, $r>::from_parts(columns)
            }
        }
    };
}

use crate::{
    matrix::{impl_matrix, AsMutMatrixParts, AsRefMatrixParts, FromMatrixParts, MatrixScalar},
    vector::{impl_vector, AsMutVectorParts, AsRefVectorParts, VectorScalar},
};

impl_vector!(2, vek::Vec2<T>; using From);
impl_vector!(3, vek::Vec3<T>; using From);
impl_vector!(4, vek::Vec4<T>; using From);

impl_matrix!(2, 2, vek::Mat2<T>);
impl_matrix!(3, 3, vek::Mat3<T>);
impl_matrix!(4, 4, vek::Mat4<T>);

macro_rules! impl_vector_traits {
    ($n:literal, $type:ty) => {
        impl<T: VectorScalar> AsRefVectorParts<T, $n> for $type {
            fn as_ref_parts(&self) -> &[T; $n] {
                self.as_slice().try_into().unwrap()
            }
        }
        impl<T: VectorScalar> AsMutVectorParts<T, $n> for $type {
            fn as_mut_parts(&mut self) -> &mut [T; $n] {
                self.as_mut_slice().try_into().unwrap()
            }
        }
    };
}

impl_vector_traits!(2, vek::Vec2<T>);
impl_vector_traits!(3, vek::Vec3<T>);
impl_vector_traits!(4, vek::Vec4<T>);

macro_rules! impl_matrix_traits {
    ($c:literal, $r:literal, $type:ty) => {
        impl<T: MatrixScalar> AsRefMatrixParts<T, $c, $r> for $type {
            fn as_ref_parts(&self) -> &[[T; $r]; $c] {
                let array = self.as_col_slice().try_into().unwrap();
                array_ref_to_2d_array_ref!(array, T, $c, $r)
            }
        }
        impl<T: MatrixScalar> AsMutMatrixParts<T, $c, $r> for $type {
            fn as_mut_parts(&mut self) -> &mut [[T; $r]; $c] {
                let array = self.as_mut_col_slice().try_into().unwrap();
                array_mut_to_2d_array_mut!(array, T, $c, $r)
            }
        }
        impl<T: MatrixScalar> FromMatrixParts<T, $c, $r> for $type {
            fn from_parts(parts: [[T; $r]; $c]) -> Self {
                Self::from_col_arrays(parts)
            }
        }
    };
}

impl_matrix_traits!(2, 2, vek::Mat2<T>);
impl_matrix_traits!(3, 3, vek::Mat3<T>);
impl_matrix_traits!(4, 4, vek::Mat4<T>);

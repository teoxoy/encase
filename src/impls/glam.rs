use crate::{
    matrix::{impl_matrix, AsMutMatrixParts, AsRefMatrixParts, FromMatrixParts, MatrixScalar},
    vector::impl_vector,
};

impl_vector!(2, glam::Vec2, f32; using AsRef AsMut From);
impl_vector!(2, glam::UVec2, u32; using AsRef AsMut From);
impl_vector!(2, glam::IVec2, i32; using AsRef AsMut From);

impl_vector!(3, glam::Vec3, f32; using AsRef AsMut From);
impl_vector!(3, glam::UVec3, u32; using AsRef AsMut From);
impl_vector!(3, glam::IVec3, i32; using AsRef AsMut From);

impl_vector!(4, glam::Vec4, f32; using AsRef AsMut From);
impl_vector!(4, glam::UVec4, u32; using AsRef AsMut From);
impl_vector!(4, glam::IVec4, i32; using AsRef AsMut From);

impl_matrix!(2, 2, glam::Mat2, f32);
impl_matrix!(3, 3, glam::Mat3, f32);
impl_matrix!(4, 4, glam::Mat4, f32);

macro_rules! impl_matrix_traits {
    ($c:literal, $r:literal, $type:ty, $el_ty:ty) => {
        impl AsRefMatrixParts<$el_ty, $c, $r> for $type
        where
            Self: AsRef<[$el_ty; $r * $c]>,
            $el_ty: MatrixScalar,
        {
            fn as_ref_parts(&self) -> &[[$el_ty; $r]; $c] {
                array_ref_to_2d_array_ref!(self.as_ref(), $el_ty, $c, $r)
            }
        }

        impl AsMutMatrixParts<$el_ty, $c, $r> for $type
        where
            Self: AsMut<[$el_ty; $r * $c]>,
            $el_ty: MatrixScalar,
        {
            fn as_mut_parts(&mut self) -> &mut [[$el_ty; $r]; $c] {
                array_mut_to_2d_array_mut!(self.as_mut(), $el_ty, $c, $r)
            }
        }

        impl FromMatrixParts<$el_ty, $c, $r> for $type {
            fn from_parts(parts: [[$el_ty; $r]; $c]) -> Self {
                Self::from_cols_array_2d(&parts)
            }
        }
    };
}

impl_matrix_traits!(2, 2, glam::Mat2, f32);
impl_matrix_traits!(3, 3, glam::Mat3, f32);
impl_matrix_traits!(4, 4, glam::Mat4, f32);

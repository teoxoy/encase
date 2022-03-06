use crate::{
    matrix::{impl_matrix, AsMutMatrixParts, AsRefMatrixParts},
    vector::{impl_vector, AsMutVectorParts, AsRefVectorParts},
};

impl_vector!(2, ultraviolet::Vec2, f32; using From);
impl_vector!(2, ultraviolet::UVec2, u32; using From);
impl_vector!(2, ultraviolet::IVec2, i32; using From);

impl_vector!(3, ultraviolet::Vec3, f32; using From);
impl_vector!(3, ultraviolet::UVec3, u32; using From);
impl_vector!(3, ultraviolet::IVec3, i32; using From);

impl_vector!(4, ultraviolet::Vec4, f32; using From);
impl_vector!(4, ultraviolet::UVec4, u32; using From);
impl_vector!(4, ultraviolet::IVec4, i32; using From);

impl_matrix!(2, 2, ultraviolet::Mat2, f32; using From);
impl_matrix!(3, 3, ultraviolet::Mat3, f32; using From);
impl_matrix!(4, 4, ultraviolet::Mat4, f32; using From);

macro_rules! impl_vector_traits {
    ($n:literal, $type:ty, $el_ty:ty) => {
        impl AsRefVectorParts<$el_ty, $n> for $type {
            fn as_ref_parts(&self) -> &[$el_ty; $n] {
                self.as_slice().try_into().unwrap()
            }
        }
        impl AsMutVectorParts<$el_ty, $n> for $type {
            fn as_mut_parts(&mut self) -> &mut [$el_ty; $n] {
                self.as_mut_slice().try_into().unwrap()
            }
        }
    };
}

impl_vector_traits!(2, ultraviolet::Vec2, f32);
impl_vector_traits!(2, ultraviolet::UVec2, u32);
impl_vector_traits!(2, ultraviolet::IVec2, i32);

impl_vector_traits!(3, ultraviolet::Vec3, f32);
impl_vector_traits!(3, ultraviolet::UVec3, u32);
impl_vector_traits!(3, ultraviolet::IVec3, i32);

impl_vector_traits!(4, ultraviolet::Vec4, f32);
impl_vector_traits!(4, ultraviolet::UVec4, u32);
impl_vector_traits!(4, ultraviolet::IVec4, i32);

macro_rules! impl_matrix_traits {
    ($c:literal, $r:literal, $type:ty, $el_ty:ty) => {
        impl AsRefMatrixParts<$el_ty, $c, $r> for $type {
            fn as_ref_parts(&self) -> &[[$el_ty; $r]; $c] {
                array_ref_to_2d_array_ref!(self.as_array(), $el_ty, $c, $r)
            }
        }
        impl AsMutMatrixParts<$el_ty, $c, $r> for $type {
            fn as_mut_parts(&mut self) -> &mut [[$el_ty; $r]; $c] {
                let array = self.as_mut_slice().try_into().unwrap();
                array_mut_to_2d_array_mut!(array, $el_ty, $c, $r)
            }
        }
    };
}

impl_matrix_traits!(2, 2, ultraviolet::Mat2, f32);
impl_matrix_traits!(3, 3, ultraviolet::Mat3, f32);
impl_matrix_traits!(4, 4, ultraviolet::Mat4, f32);

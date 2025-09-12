use encase::{matrix::impl_matrix, vector::impl_vector};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector<T, const N: usize>([T; N]);

impl<T, const N: usize> AsRef<[T; N]> for Vector<T, N> {
    fn as_ref(&self) -> &[T; N] {
        &self.0
    }
}
impl<T, const N: usize> AsMut<[T; N]> for Vector<T, N> {
    fn as_mut(&mut self) -> &mut [T; N] {
        &mut self.0
    }
}
impl<T, const N: usize> From<[T; N]> for Vector<T, N> {
    fn from(a: [T; N]) -> Self {
        Self(a)
    }
}

pub type Vec2<T> = Vector<T, 2>;
pub type Vec3<T> = Vector<T, 3>;
pub type Vec4<T> = Vector<T, 4>;

pub type Vec2i = Vec2<i32>;
pub type Vec2u = Vec2<u32>;
pub type Vec2f = Vec2<f32>;

impl_vector!(2, Vec2i, i32; using AsRef AsMut From);
impl_vector!(2, Vec2u, u32; using AsRef AsMut From);
impl_vector!(2, Vec2f, f32; using AsRef AsMut From);

pub type Vec3i = Vec3<i32>;
pub type Vec3u = Vec3<u32>;
pub type Vec3f = Vec3<f32>;

impl_vector!(3, Vec3i, i32; using AsRef AsMut From);
impl_vector!(3, Vec3u, u32; using AsRef AsMut From);
impl_vector!(3, Vec3f, f32; using AsRef AsMut From);

pub type Vec4i = Vec4<i32>;
pub type Vec4u = Vec4<u32>;
pub type Vec4f = Vec4<f32>;

impl_vector!(4, Vec4i, i32; using AsRef AsMut From);
impl_vector!(4, Vec4u, u32; using AsRef AsMut From);
impl_vector!(4, Vec4f, f32; using AsRef AsMut From);

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix<T, const C: usize, const R: usize>([[T; R]; C]);

impl<T, const C: usize, const R: usize> AsRef<[[T; R]; C]> for Matrix<T, C, R> {
    fn as_ref(&self) -> &[[T; R]; C] {
        &self.0
    }
}
impl<T, const C: usize, const R: usize> AsMut<[[T; R]; C]> for Matrix<T, C, R> {
    fn as_mut(&mut self) -> &mut [[T; R]; C] {
        &mut self.0
    }
}
impl<T, const C: usize, const R: usize> From<[[T; R]; C]> for Matrix<T, C, R> {
    fn from(a: [[T; R]; C]) -> Self {
        Self(a)
    }
}

pub type Mat2x2<T> = Matrix<T, 2, 2>;
pub type Mat2x3<T> = Matrix<T, 2, 3>;
pub type Mat2x4<T> = Matrix<T, 2, 4>;

pub type Mat3x2<T> = Matrix<T, 3, 2>;
pub type Mat3x3<T> = Matrix<T, 3, 3>;
pub type Mat3x4<T> = Matrix<T, 3, 4>;

pub type Mat4x2<T> = Matrix<T, 4, 2>;
pub type Mat4x3<T> = Matrix<T, 4, 3>;
pub type Mat4x4<T> = Matrix<T, 4, 4>;

pub type Mat2x2f = Mat2x2<f32>;
pub type Mat2x3f = Mat2x3<f32>;
pub type Mat2x4f = Mat2x4<f32>;

impl_matrix!(2, 2, Mat2x2f, f32; using AsRef AsMut From);
impl_matrix!(2, 3, Mat2x3f, f32; using AsRef AsMut From);
impl_matrix!(2, 4, Mat2x4f, f32; using AsRef AsMut From);

pub type Mat3x2f = Mat3x2<f32>;
pub type Mat3x3f = Mat3x3<f32>;
pub type Mat3x4f = Mat3x4<f32>;

impl_matrix!(3, 2, Mat3x2f, f32; using AsRef AsMut From);
impl_matrix!(3, 3, Mat3x3f, f32; using AsRef AsMut From);
impl_matrix!(3, 4, Mat3x4f, f32; using AsRef AsMut From);

pub type Mat4x2f = Mat4x2<f32>;
pub type Mat4x3f = Mat4x3<f32>;
pub type Mat4x4f = Mat4x4<f32>;

impl_matrix!(4, 2, Mat4x2f, f32; using AsRef AsMut From);
impl_matrix!(4, 3, Mat4x3f, f32; using AsRef AsMut From);
impl_matrix!(4, 4, Mat4x4f, f32; using AsRef AsMut From);

pub trait Constants: Copy {
    const ZERO: Self;
    const ONE: Self;
}
impl Constants for i32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl Constants for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
}
impl Constants for f32 {
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;
}
impl<T: Constants, const N: usize> Vector<T, N> {
    pub const ZERO: Self = Self([T::ZERO; N]);
    pub const ONE: Self = Self([T::ONE; N]);
}
impl<T: Constants, const C: usize, const R: usize> Matrix<T, C, R> {
    pub const ZERO: Self = Self([[T::ZERO; R]; C]);
    pub const ONE: Self = Self([[T::ONE; R]; C]);
}
impl<T: Constants, const S: usize> Matrix<T, S, S> {
    pub const IDENTITY: Self = {
        let mut res = Self::ZERO;
        let mut i = 0;
        while i < S {
            res.0[i][i] = T::ONE;
            i += 1;
        }
        res
    };
}

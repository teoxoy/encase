use crate::{
    matrix::{impl_matrix, AsMutMatrixParts, AsRefMatrixParts, FromMatrixParts, MatrixScalar},
    vector::{impl_vector, AsMutVectorParts, AsRefVectorParts, FromVectorParts, VectorScalar},
};

impl_vector!(2, nalgebra::VectorView2<'_, T>);
impl_vector!(2, nalgebra::VectorViewMut2<'_, T>);
impl_vector!(2, nalgebra::Vector2<T>);

impl_vector!(3, nalgebra::VectorView3<'_, T>);
impl_vector!(3, nalgebra::VectorViewMut3<'_, T>);
impl_vector!(3, nalgebra::Vector3<T>);

impl_vector!(4, nalgebra::VectorView4<'_, T>);
impl_vector!(4, nalgebra::VectorViewMut4<'_, T>);
impl_vector!(4, nalgebra::Vector4<T>);

impl_matrix!(2, 2, nalgebra::MatrixView2<'_, T>);
impl_matrix!(2, 2, nalgebra::MatrixViewMut2<'_, T>);
impl_matrix!(2, 2, nalgebra::Matrix2<T>);

impl_matrix!(3, 2, nalgebra::MatrixView2x3<'_, T>);
impl_matrix!(4, 2, nalgebra::MatrixView2x4<'_, T>);
impl_matrix!(2, 3, nalgebra::MatrixView3x2<'_, T>);
impl_matrix!(3, 2, nalgebra::MatrixViewMut2x3<'_, T>);
impl_matrix!(4, 2, nalgebra::MatrixViewMut2x4<'_, T>);
impl_matrix!(2, 3, nalgebra::MatrixViewMut3x2<'_, T>);
impl_matrix!(3, 2, nalgebra::Matrix2x3<T>);
impl_matrix!(4, 2, nalgebra::Matrix2x4<T>);
impl_matrix!(2, 3, nalgebra::Matrix3x2<T>);

impl_matrix!(3, 3, nalgebra::MatrixView3<'_, T>);
impl_matrix!(3, 3, nalgebra::MatrixViewMut3<'_, T>);
impl_matrix!(3, 3, nalgebra::Matrix3<T>);

impl_matrix!(4, 3, nalgebra::MatrixView3x4<'_, T>);
impl_matrix!(2, 4, nalgebra::MatrixView4x2<'_, T>);
impl_matrix!(3, 4, nalgebra::MatrixView4x3<'_, T>);
impl_matrix!(4, 3, nalgebra::MatrixViewMut3x4<'_, T>);
impl_matrix!(2, 4, nalgebra::MatrixViewMut4x2<'_, T>);
impl_matrix!(3, 4, nalgebra::MatrixViewMut4x3<'_, T>);
impl_matrix!(4, 3, nalgebra::Matrix3x4<T>);
impl_matrix!(2, 4, nalgebra::Matrix4x2<T>);
impl_matrix!(3, 4, nalgebra::Matrix4x3<T>);

impl_matrix!(4, 4, nalgebra::MatrixView4<'_, T>);
impl_matrix!(4, 4, nalgebra::MatrixViewMut4<'_, T>);
impl_matrix!(4, 4, nalgebra::Matrix4<T>);

impl<T: VectorScalar, S, const N: usize> AsRefVectorParts<T, N>
    for nalgebra::Matrix<T, nalgebra::Const<N>, nalgebra::Const<1>, S>
where
    Self: AsRef<[T; N]>,
{
    fn as_ref_parts(&self) -> &[T; N] {
        self.as_ref()
    }
}

impl<T: VectorScalar, S, const N: usize> AsMutVectorParts<T, N>
    for nalgebra::Matrix<T, nalgebra::Const<N>, nalgebra::Const<1>, S>
where
    Self: AsMut<[T; N]>,
{
    fn as_mut_parts(&mut self) -> &mut [T; N] {
        self.as_mut()
    }
}

impl<T: VectorScalar, const N: usize> FromVectorParts<T, N> for nalgebra::SMatrix<T, N, 1> {
    fn from_parts(parts: [T; N]) -> Self {
        Self::from_array_storage(nalgebra::ArrayStorage([parts]))
    }
}

impl<T: MatrixScalar, S, const C: usize, const R: usize> AsRefMatrixParts<T, C, R>
    for nalgebra::Matrix<T, nalgebra::Const<R>, nalgebra::Const<C>, S>
where
    Self: AsRef<[[T; R]; C]>,
{
    fn as_ref_parts(&self) -> &[[T; R]; C] {
        self.as_ref()
    }
}

impl<T: MatrixScalar, S, const C: usize, const R: usize> AsMutMatrixParts<T, C, R>
    for nalgebra::Matrix<T, nalgebra::Const<R>, nalgebra::Const<C>, S>
where
    Self: AsMut<[[T; R]; C]>,
{
    fn as_mut_parts(&mut self) -> &mut [[T; R]; C] {
        self.as_mut()
    }
}

impl<T: MatrixScalar, const C: usize, const R: usize> FromMatrixParts<T, C, R>
    for nalgebra::SMatrix<T, R, C>
{
    fn from_parts(parts: [[T; R]; C]) -> Self {
        Self::from_array_storage(nalgebra::ArrayStorage(parts))
    }
}

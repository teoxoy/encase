use crate::{matrix::impl_matrix, vector::impl_vector};

impl_vector!(2, mint::Vector2<T>; using AsRef AsMut From);
impl_vector!(3, mint::Vector3<T>; using AsRef AsMut From);
impl_vector!(4, mint::Vector4<T>; using AsRef AsMut From);

impl_vector!(2, mint::Point2<T>; using AsRef AsMut From);
impl_vector!(3, mint::Point3<T>; using AsRef AsMut From);

impl_matrix!(2, 2, mint::ColumnMatrix2<T>; using AsRef AsMut From);

impl_matrix!(3, 2, mint::ColumnMatrix2x3<T>; using AsRef AsMut From);
impl_matrix!(4, 2, mint::ColumnMatrix2x4<T>; using AsRef AsMut From);
impl_matrix!(2, 3, mint::ColumnMatrix3x2<T>; using AsRef AsMut From);

impl_matrix!(3, 3, mint::ColumnMatrix3<T>; using AsRef AsMut From);

impl_matrix!(4, 3, mint::ColumnMatrix3x4<T>; using AsRef AsMut From);
impl_matrix!(2, 4, mint::ColumnMatrix4x2<T>; using AsRef AsMut From);
impl_matrix!(3, 4, mint::ColumnMatrix4x3<T>; using AsRef AsMut From);

impl_matrix!(4, 4, mint::ColumnMatrix4<T>; using AsRef AsMut From);

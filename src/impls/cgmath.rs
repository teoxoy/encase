use crate::{matrix::impl_matrix, vector::impl_vector};

impl_vector!(2, cgmath::Vector2<T>; using AsRef AsMut From);
impl_vector!(3, cgmath::Vector3<T>; using AsRef AsMut From);
impl_vector!(4, cgmath::Vector4<T>; using AsRef AsMut From);

impl_vector!(2, cgmath::Point2<T>; using AsRef AsMut From);
impl_vector!(3, cgmath::Point3<T>; using AsRef AsMut From);

impl_matrix!(2, 2, cgmath::Matrix2<T>; using AsRef AsMut From);
impl_matrix!(3, 3, cgmath::Matrix3<T>; using AsRef AsMut From);
impl_matrix!(4, 4, cgmath::Matrix4<T>; using AsRef AsMut From);

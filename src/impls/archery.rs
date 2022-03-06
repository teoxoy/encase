use crate::impl_wrapper;

impl_wrapper!(archery::SharedPointer<T, P>; (T, P: archery::SharedPointerKind); using Ref{} From{ new });

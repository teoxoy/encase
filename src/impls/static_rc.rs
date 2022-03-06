use crate::impl_wrapper;

impl_wrapper!(static_rc::StaticRc<T, NUM, DEN>; (T: ?Sized, const NUM: usize, const DEN: usize); using Ref{});
impl_wrapper!(static_rc::StaticRc<T, N, N>; (T: ?Sized, const N: usize); using Mut{} From{ new });

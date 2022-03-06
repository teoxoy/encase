use crate::rts_array::impl_rts_array;

// hardcap
impl_rts_array!(arrayvec::ArrayVec<T, N>; (T, const N: usize); using len truncate);

use crate::rts_array::impl_rts_array;

// hardcap
impl_rts_array!(tinyvec::ArrayVec<A>; (T, A: tinyvec::Array<Item = T>); using len truncate);

// softcap
impl_rts_array!(tinyvec::TinyVec<A>; (T, A: tinyvec::Array<Item = T>); using len truncate);

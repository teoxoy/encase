use crate::rts_array::impl_rts_array;

// softcap
impl_rts_array!(smallvec::SmallVec<A>; (T, A: smallvec::Array<Item = T>); using len truncate);

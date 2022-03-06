use crate::rts_array::impl_rts_array;

impl_rts_array!(imbl::Vector<T>; (T: Clone); using len);

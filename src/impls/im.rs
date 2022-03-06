use crate::rts_array::impl_rts_array;

impl_rts_array!(im::Vector<T>; (T: Clone); using len);

use crate::rts_array::impl_rts_array;

impl_rts_array!(im_rc::Vector<T>; (T: Clone); using len);

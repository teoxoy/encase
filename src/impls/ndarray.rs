use crate::rts_array::impl_rts_array;

impl_rts_array!(ndarray::ArrayBase<S, D>; (T, S: ndarray::RawData<Elem = T>, D: ndarray::Dimension); using len);

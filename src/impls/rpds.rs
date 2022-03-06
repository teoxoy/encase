use crate::rts_array::{impl_rts_array, Length};

impl_rts_array!(rpds::List<T, P>; (T, P: archery::SharedPointerKind); using len);
impl_rts_array!(rpds::Vector<T, P>; (T, P: archery::SharedPointerKind); using len);
impl_rts_array!(rpds::Stack<T, P>; (T, P: archery::SharedPointerKind));
impl_rts_array!(rpds::Queue<T, P>; (T, P: archery::SharedPointerKind); using len);

impl<T, P: archery::SharedPointerKind> Length for rpds::Stack<T, P> {
    fn length(&self) -> usize {
        self.size()
    }
}

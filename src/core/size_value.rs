use core::num::NonZeroU64;

/// Helper type for size calculations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SizeValue(pub NonZeroU64);

impl SizeValue {
    pub const fn new(val: u64) -> Self {
        match val {
            0 => panic!("Size can't be 0!"),
            val => {
                // SAFETY: This is safe since we checked if the value is 0
                Self(unsafe { NonZeroU64::new_unchecked(val) })
            }
        }
    }

    pub const fn from(val: NonZeroU64) -> Self {
        Self(val)
    }

    pub const fn get(&self) -> u64 {
        self.0.get()
    }

    pub const fn mul(self, rhs: u64) -> Self {
        match self.get().checked_mul(rhs) {
            None => panic!("Overflow occured while multiplying size values!"),
            Some(val) => {
                // SAFETY: This is safe since we checked for overflow
                Self(unsafe { NonZeroU64::new_unchecked(val) })
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::SizeValue;

    #[test]
    fn new() {
        assert_eq!(4, SizeValue::new(4).get());
    }

    #[test]
    #[should_panic]
    fn new_panic() {
        SizeValue::new(0);
    }

    #[test]
    fn mul() {
        assert_eq!(SizeValue::new(64), SizeValue::new(8).mul(8));
    }

    #[test]
    #[should_panic]
    fn mul_panic() {
        SizeValue::new(8).mul(u64::MAX);
    }

    #[test]
    fn derived_traits() {
        let size = SizeValue::new(8);
        #[allow(clippy::clone_on_copy)]
        let size_clone = size.clone();

        assert!(size == size_clone);

        assert_eq!(format!("{:?}", size), "SizeValue(8)");
    }
}

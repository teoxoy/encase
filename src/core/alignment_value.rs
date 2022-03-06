use super::SizeValue;
use core::num::NonZeroU64;

/// Helper type for alignment calculations
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AlignmentValue(NonZeroU64);

impl AlignmentValue {
    pub const fn new(val: u64) -> Self {
        if !val.is_power_of_two() {
            panic!("Alignment must be a power of 2!");
        }
        // SAFETY: This is safe since 0 is not a power of 2
        Self(unsafe { NonZeroU64::new_unchecked(val) })
    }

    /// Returns an alignment that is the smallest power of two greater than the passed in `size`
    pub const fn from_next_power_of_two_size(size: SizeValue) -> Self {
        match size.get().checked_next_power_of_two() {
            None => panic!("Overflow occured while getting the next power of 2!"),
            Some(val) => {
                // SAFETY: This is safe since we got the next_power_of_two
                Self(unsafe { NonZeroU64::new_unchecked(val) })
            }
        }
    }

    pub const fn get(&self) -> u64 {
        self.0.get()
    }

    /// Returns the max alignment from an array of alignments
    pub const fn max<const N: usize>(input: [AlignmentValue; N]) -> AlignmentValue {
        let mut max = input[0];
        let mut i = 1;

        while i < N {
            if input[i].get() > max.get() {
                max = input[i];
            }

            i += 1;
        }

        max
    }

    /// Returns true if `n` is a multiple of this alignment
    pub const fn is_aligned(&self, n: u64) -> bool {
        n % self.get() == 0
    }

    /// Returns the amount of padding needed so that `n + padding` will be a multiple of this alignment
    pub const fn padding_needed_for(&self, n: u64) -> u64 {
        let r = n % self.get();
        if r > 0 {
            self.get() - r
        } else {
            0
        }
    }

    /// Will round up the given `n` so that the returned value will be a multiple of this alignment
    pub const fn round_up(&self, n: u64) -> u64 {
        n + self.padding_needed_for(n)
    }

    /// Will round up the given `n` so that the returned value will be a multiple of this alignment
    pub const fn round_up_size(&self, n: SizeValue) -> SizeValue {
        SizeValue::new(self.round_up(n.get()))
    }
}

#[cfg(test)]
mod test {
    use super::AlignmentValue;

    #[test]
    fn new() {
        assert_eq!(4, AlignmentValue::new(4).get());
    }

    #[test]
    #[should_panic]
    fn new_panic() {
        AlignmentValue::new(3);
    }

    #[test]
    fn from_next_power_of_two_size() {
        assert_eq!(
            AlignmentValue::new(8),
            AlignmentValue::from_next_power_of_two_size(super::SizeValue::new(7))
        );
    }

    #[test]
    #[should_panic]
    fn from_next_power_of_two_size_panic() {
        AlignmentValue::from_next_power_of_two_size(super::SizeValue::new(u64::MAX));
    }

    #[test]
    fn max() {
        assert_eq!(
            AlignmentValue::new(32),
            AlignmentValue::max([
                AlignmentValue::new(2),
                AlignmentValue::new(8),
                AlignmentValue::new(32)
            ])
        );
    }

    #[test]
    fn is_aligned() {
        assert!(AlignmentValue::new(8).is_aligned(32));
        assert!(!AlignmentValue::new(8).is_aligned(9));
    }

    #[test]
    fn padding_needed_for() {
        assert_eq!(1, AlignmentValue::new(8).padding_needed_for(7));
        assert_eq!(16 - 9, AlignmentValue::new(8).padding_needed_for(9));
    }

    #[test]
    fn round_up() {
        assert_eq!(24, AlignmentValue::new(8).round_up(20));
        assert_eq!(
            super::SizeValue::new(16),
            AlignmentValue::new(16).round_up_size(super::SizeValue::new(7))
        );
    }

    #[test]
    fn derived_traits() {
        let alignment = AlignmentValue::new(8);
        #[allow(clippy::clone_on_copy)]
        let alignment_clone = alignment.clone();

        assert!(alignment == alignment_clone);

        assert_eq!(format!("{:?}", alignment), "AlignmentValue(8)");
    }
}

use crate::core::Metadata;

pub struct StructMetadata<const N: usize> {
    pub offsets: [u64; N],
    pub paddings: [u64; N],
}

impl<const N: usize> Metadata<StructMetadata<N>> {
    pub const fn offset(self, i: usize) -> u64 {
        self.extra.offsets[i]
    }

    pub const fn last_offset(self) -> u64 {
        self.extra.offsets[N - 1]
    }

    pub const fn padding(self, i: usize) -> u64 {
        self.extra.paddings[i]
    }
}

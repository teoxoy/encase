// Const string implementation for SHADER_TYPE and SHADER_STRUCT_DECLARATION
// Used instead of crates like const_str because of E0401 when trying to use them in traits
// See also https://old.reddit.com/r/rust/comments/sv119a/concat_static_str_at_compile_time/

// Must be constant to avoid running into E0401. Should only affect compilation.
const BUFFER_SIZE: usize = 8192;

pub struct ConstStr {
    data: [u8; BUFFER_SIZE],
    len: usize,
}

impl ConstStr {
    pub const fn new() -> ConstStr {
        ConstStr {
            data: [0u8; BUFFER_SIZE],
            len: 0,
        }
    }

    pub const fn str(mut self, s: &str) -> Self {
        let b = s.as_bytes();
        let mut index = 0;
        while index < b.len() {
            self.data[self.len] = b[index];
            self.len += 1;
            index += 1;
        }

        self
    }

    pub const fn u64(mut self, x: u64) -> Self {
        let mut x2 = x;
        let mut l = 0;
        loop {
            l += 1;
            x2 /= 10;
            if x2 == 0 {
                break;
            }
        }
        let mut x3 = x;
        let mut index = 0;
        loop {
            self.data[self.len + l - 1 - index] = (x3 % 10) as u8 + b'0';
            index += 1;
            x3 /= 10;
            if x3 == 0 {
                break;
            }
        }
        self.len += l;

        self
    }

    pub const fn as_str(&self) -> &str {
        // SAFETY: safe because this is only used in const, and should be correct by construction
        unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.data.as_ptr(), self.len))
        }
    }
}

mod test {
    use super::ConstStr;

    trait Name {
        const NAME: &'static str;
    }

    trait Prefix: Name {}

    trait Root: Name {}

    struct Kilo;

    impl Prefix for Kilo {}

    struct Meter;

    impl Root for Meter {}

    impl Name for Kilo {
        const NAME: &'static str = "kilo";
    }

    impl Name for Meter {
        const NAME: &'static str = "meter";
    }

    impl<P: Prefix, R: Root> Name for (P, R) {
        const NAME: &'static str = ConstStr::new()
            .str(P::NAME)
            .str(R::NAME)
            .u64(1234567)
            .as_str();
    }

    #[test]
    fn test_trait() {
        assert_eq!(<(Kilo, Meter)>::NAME, "kilometer1234567");
    }
}

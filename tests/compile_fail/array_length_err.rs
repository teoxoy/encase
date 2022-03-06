use encase::{ArrayLength, WgslType};

fn main() {}

#[derive(WgslType)]
struct Test {
    a: ArrayLength,
    b: ArrayLength,
}

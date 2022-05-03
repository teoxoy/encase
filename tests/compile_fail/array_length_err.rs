use encase::{ArrayLength, ShaderType};

fn main() {}

#[derive(ShaderType)]
struct Test {
    a: ArrayLength,
    b: ArrayLength,
}

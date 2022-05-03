use encase::ShaderType;

fn main() {}

#[derive(ShaderType)]
struct Test {
    #[align]
    a: u32,
    #[align()]
    b: u32,
    #[align(invalid)]
    c: u32,
    #[align(3)]
    d: u32,
}

use encase::ShaderType;

fn main() {}

#[derive(ShaderType)]
struct Test {
    #[shader(align)]
    a: u32,
    #[shader(align())]
    b: u32,
    #[shader(align(invalid))]
    c: u32,
    #[shader(align(3))]
    d: u32,
}

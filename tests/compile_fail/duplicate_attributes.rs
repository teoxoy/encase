use encase::ShaderType;

fn main() {}

#[derive(ShaderType)]
struct Test {
    #[shader(align(8), align(16))]
    a: u32,
    #[shader(size(runtime), size(16))]
    b: u32,
}

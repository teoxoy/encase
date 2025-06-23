use encase::ShaderType;

fn main() {}

#[derive(ShaderType)]
struct Test {
    #[shader_align]
    a: u32,
    #[shader_align()]
    b: u32,
    #[shader_align(invalid)]
    c: u32,
    #[shader_align(3)]
    d: u32,
}

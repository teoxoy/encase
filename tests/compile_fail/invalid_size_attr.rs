use encase::ShaderType;

fn main() {}

#[derive(ShaderType)]
struct Test {
    #[shader(size)]
    a: u32,
    #[shader(size())]
    b: u32,
    #[shader(size(invalid))]
    c: u32,
    #[shader(size(runtime))]
    d: u32,
    #[shader(size(-1))]
    e: u32,
}

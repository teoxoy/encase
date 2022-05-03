use encase::ShaderType;

fn main() {}

#[derive(ShaderType)]
struct Test {
    #[size]
    a: u32,
    #[size()]
    b: u32,
    #[size(invalid)]
    c: u32,
    #[size(-1)]
    d: u32,
}

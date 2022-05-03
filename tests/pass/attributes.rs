use encase::ShaderType;

fn main() {}

#[derive(ShaderType)]
struct TestAttributes {
    #[align(16)]
    a: u32,
    #[size(8)]
    b: u32,
}

#[derive(ShaderType)]
struct TestRtArray {
    #[size(8)]
    a: u32,
    #[align(16)]
    #[size(runtime)]
    b: Vec<u32>,
}

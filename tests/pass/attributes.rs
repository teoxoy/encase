use encase::WgslType;

fn main() {}

#[derive(WgslType)]
#[assert_uniform_compat]
struct TestAttributes {
    a: u32,
    #[align(16)]
    #[size(8)]
    b: u32,
}

#[derive(WgslType)]
struct TestRtArray {
    #[size(8)]
    a: u32,
    #[align(16)]
    #[size(runtime)]
    b: Vec<u32>,
}

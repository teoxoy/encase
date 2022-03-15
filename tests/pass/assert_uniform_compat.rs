use encase::WgslType;

fn main() {}

#[derive(WgslType)]
struct S {
    x: f32,
}

#[derive(WgslType)]
struct WrappedF32 {
    #[size(16)]
    elem: f32,
}

#[derive(WgslType)]
#[assert_uniform_compat]
struct TestStruct {
    a: u32,
    #[align(16)]
    b: S,
}

#[derive(WgslType)]
#[assert_uniform_compat]
struct TestArray {
    a: u32,
    #[align(16)]
    b: [WrappedF32; 1],
}

#[derive(WgslType)]
#[assert_uniform_compat]
struct TestStructFirst {
    a: S,
    #[align(16)]
    b: f32,
}

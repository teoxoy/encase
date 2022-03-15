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
    b: S,
}

#[derive(WgslType)]
#[assert_uniform_compat]
struct TestArray {
    a: u32,
    b: [WrappedF32; 1],
}

#[derive(WgslType)]
#[assert_uniform_compat]
struct TestStructFirst {
    a: S,
    b: f32,
}

#[derive(WgslType)]
#[assert_uniform_compat]
struct TestArrayStride {
    a: [u32; 8],
}

#[derive(WgslType)]
#[assert_uniform_compat]
struct TestRTSArray {
    #[size(runtime)]
    a: Vec<f32>,
}

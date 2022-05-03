use encase::ShaderType;

#[derive(ShaderType)]
struct S {
    x: f32,
}

#[derive(ShaderType)]
struct WrappedF32 {
    #[size(16)]
    elem: f32,
}

#[derive(ShaderType)]
struct TestStruct {
    a: u32,
    #[align(16)]
    b: S,
}

#[derive(ShaderType)]
struct TestArray {
    a: u32,
    #[align(16)]
    b: [WrappedF32; 1],
}

#[derive(ShaderType)]
struct TestStructFirst {
    a: S,
    #[align(16)]
    b: f32,
}

#[test]
fn assert_uniform_compat_success() {
    TestStruct::assert_uniform_compat();
    TestArray::assert_uniform_compat();
    TestStructFirst::assert_uniform_compat();
}

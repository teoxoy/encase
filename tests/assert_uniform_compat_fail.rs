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

#[test]
#[should_panic]
fn test_struct() {
    #[derive(ShaderType)]
    struct TestStruct {
        a: u32,
        b: S,
    }

    TestStruct::assert_uniform_compat()
}

#[test]
#[should_panic]
fn test_array() {
    #[derive(ShaderType)]
    struct TestArray {
        a: u32,
        b: [WrappedF32; 1],
    }

    TestArray::assert_uniform_compat()
}

#[test]
#[should_panic]
fn test_struct_first() {
    #[derive(ShaderType)]
    struct TestStructFirst {
        a: S,
        b: f32,
    }

    TestStructFirst::assert_uniform_compat()
}

#[test]
#[should_panic]
fn test_array_stride() {
    #[derive(ShaderType)]
    struct TestArrayStride {
        a: [u32; 8],
    }

    TestArrayStride::assert_uniform_compat()
}

#[test]
#[should_panic]
fn test_rts_array() {
    #[derive(ShaderType)]
    struct TestRTSArray {
        #[size(runtime)]
        a: Vec<f32>,
    }

    TestRTSArray::assert_uniform_compat()
}

use encase::ShaderType;

#[derive(ShaderType)]
struct WrappedF32 {
    #[size(16)]
    value: f32,
}

#[test]
fn field_padding() {
    assert_eq!(WrappedF32::METADATA.padding(0), 12);
}

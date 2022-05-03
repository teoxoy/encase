use encase::{ShaderType, UniformBuffer};

#[derive(Debug, ShaderType, PartialEq, Eq)]
struct TestUniform {
    a: u32,
    b: u32,
}

#[test]
fn uniform() {
    let mut val = TestUniform { a: 4, b: 23 };

    let mut buffer = UniformBuffer::new(Vec::new());

    buffer.write(&val).unwrap();
    buffer.read(&mut val).unwrap();
    assert_eq!(val, buffer.create().unwrap());
}

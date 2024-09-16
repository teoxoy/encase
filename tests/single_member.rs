use encase::ShaderType;

#[derive(ShaderType)]
struct Test {
    a: u32,
}

#[test]
fn single_member_uniform() {
    let mut buffer = encase::DynamicUniformBuffer::new(Vec::<u8>::new());

    assert_eq!(buffer.write_struct_member(&1_u32).unwrap(), 0);
    assert_eq!(
        buffer.write_struct_member(&glam::UVec2::new(2, 3)).unwrap(),
        8
    );
    assert_eq!(buffer.write_struct_member(&4_u32).unwrap(), 16);
    assert_eq!(
        buffer
            .write_struct_member(&glam::UVec3::new(5, 6, 7))
            .unwrap(),
        32
    );
    assert_eq!(buffer.write_struct_member(&[8, 9]).unwrap(), 48);
    assert_eq!(buffer.write_struct_member(&Test { a: 10 }).unwrap(), 64);

    let cast: &[u32] = bytemuck::cast_slice(buffer.as_ref());

    assert_eq!(cast, &[1, 0, 2, 3, 4, 0, 0, 0, 5, 6, 7, 0, 8, 9, 0, 0, 10]);
}

#[test]
fn single_member_storage() {
    let mut buffer = encase::DynamicStorageBuffer::new(Vec::<u8>::new());

    assert_eq!(buffer.write_struct_member(&1_u32).unwrap(), 0);
    assert_eq!(
        buffer.write_struct_member(&glam::UVec2::new(2, 3)).unwrap(),
        8
    );
    assert_eq!(buffer.write_struct_member(&4_u32).unwrap(), 16);
    assert_eq!(
        buffer
            .write_struct_member(&glam::UVec3::new(5, 6, 7))
            .unwrap(),
        32
    );
    assert_eq!(buffer.write_struct_member(&[8, 9]).unwrap(), 44);
    assert_eq!(buffer.write_struct_member(&Test { a: 10 }).unwrap(), 52);

    let cast: &[u32] = bytemuck::cast_slice(buffer.as_ref());

    assert_eq!(cast, &[1, 0, 2, 3, 4, 0, 0, 0, 5, 6, 7, 8, 9, 10]);
}

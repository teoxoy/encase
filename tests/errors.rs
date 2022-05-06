use encase::{internal::Error, ShaderType, StorageBuffer};

#[test]
fn buffer_too_small() {
    #[derive(ShaderType)]
    struct Test {
        a: u32,
    }

    let mut v = Test { a: 4 };
    let mut buffer = StorageBuffer::new([0u8]);

    assert!(matches!(
        buffer.write(&v),
        Err(Error::BufferTooSmall {
            expected: 4,
            found: 1
        })
    ));

    assert!(matches!(
        buffer.read(&mut v),
        Err(Error::BufferTooSmall {
            expected: 4,
            found: 1
        })
    ));

    assert!(matches!(
        buffer.create::<Test>(),
        Err(Error::BufferTooSmall {
            expected: 4,
            found: 1
        })
    ));
}

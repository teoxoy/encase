<p align="center"><img src="./logo.svg" width="320px" alt/></p>

Provides a mechanism to lay out data into GPU buffers ensuring WGSL's memory layout requirements are met.

## Features

- supports all WGSL [host-shareable types] + wrapper types (`&T`, `&mut T`, `Box<T>`, ...)
- supports data types from a multitude of crates as [features]
- covers a wide area of use cases (see [examples](#examples))

## Motivation

Having to manually lay out data into GPU buffers can become very tedious and error prone. How do you make sure the data in the buffer is laid out correctly? Enforce it so that future changes don't break this delicate balance?

`encase` gives you the ability to make sure at compile time that your types will be laid out correctly.

## Design

The core trait is [`ShaderType`] which mainly contains metadata about the given type.

The [`WriteInto`], [`ReadFrom`] and [`CreateFrom`] traits represent the ability of a type to be written into the buffer, read from the buffer and created from the buffer respectively.

Most data types can implement the above traits via their respective macros:

  - [`impl_vector!`] for vectors
  - [`impl_matrix!`] for matrices
  - [`impl_rts_array!`] for runtime-sized arrays
  - [`impl_wrapper!`] for wrappers
  - [`ShaderType`][derive@ShaderType] for structs

The [`UniformBuffer`], [`StorageBuffer`], [`DynamicUniformBuffer`] and [`DynamicStorageBuffer`] structs are wrappers around an underlying raw buffer (a type implementing [`BufferRef`] and/or [`BufferMut`] depending on required capability). They facilitate the read/write/create operations.

## Examples

Write affine transform to uniform buffer

```rust
use encase::{ShaderType, UniformBuffer};

#[derive(ShaderType)]
struct AffineTransform2D {
    matrix: glam::Mat2,
    translate: glam::Vec2
}

let transform = AffineTransform2D {
    matrix: glam::Mat2::IDENTITY,
    translate: glam::Vec2::ZERO,
};

let mut buffer = UniformBuffer::new(Vec::new());
buffer.write(&transform).unwrap();
let byte_buffer = buffer.into_inner();

// write byte_buffer to GPU

assert_eq!(&byte_buffer, &[0, 0, 128, 63, 0, 0, 0, 0,
0, 0, 0, 0, 0, 0, 128, 63, 0, 0, 0, 0, 0, 0, 0, 0]);
```

Create vector instance by reading from dynamic uniform buffer at specific offset

```rust
use encase::DynamicUniformBuffer;

// read byte_buffer from GPU
let byte_buffer = [1u8; 256 + 8];

let mut buffer = DynamicUniformBuffer::new(&byte_buffer);
buffer.set_offset(256);
let vector: mint::Vector2<i32> = buffer.create().unwrap();

assert_eq!(vector, mint::Vector2 { x: 16843009, y: 16843009 });
```

Write and read back data from storage buffer

```rust
use encase::{ShaderType, ArrayLength, StorageBuffer};

#[derive(ShaderType)]
struct Positions {
    length: ArrayLength,
    #[size(runtime)]
    positions: Vec<mint::Point2<f32>>
}

let mut positions = Positions {
    length: ArrayLength,
    positions: Vec::from([
        mint::Point2 { x: 4.5, y: 3.4 },
        mint::Point2 { x: 1.5, y: 7.4 },
        mint::Point2 { x: 4.3, y: 1.9 },
    ])
};

let mut byte_buffer = Vec::new();

let mut buffer = StorageBuffer::new(&mut byte_buffer);
buffer.write(&positions).unwrap();

// write byte_buffer to GPU

// change length on GPU side
byte_buffer[0] = 2;

// read byte_buffer from GPU

let mut buffer = StorageBuffer::new(&mut byte_buffer);
buffer.read(&mut positions).unwrap();

assert_eq!(positions.positions.len(), 2);

```

Write different data types to dynamic storage buffer

```rust
use encase::{ShaderType, DynamicStorageBuffer};

let mut byte_buffer = Vec::new();

let mut buffer = DynamicStorageBuffer::new_with_alignment(&mut byte_buffer, 64);
let offsets = [
    buffer.write(&[5.; 10]).unwrap(),
    buffer.write(&vec![3u32; 20]).unwrap(),
    buffer.write(&glam::Vec3::ONE).unwrap(),
];

// write byte_buffer to GPU

assert_eq!(offsets, [0, 64, 192]);

```

[host-shareable types]: https://gpuweb.github.io/gpuweb/wgsl/#host-shareable-types
[features]: https://docs.rs/crate/encase/latest/features
[`ShaderType`]: https://docs.rs/encase/latest/encase/trait.ShaderType.html

[`WriteInto`]: https://docs.rs/encase/latest/encase/internal/trait.WriteInto.html
[`ReadFrom`]: https://docs.rs/encase/latest/encase/internal/trait.ReadFrom.html
[`CreateFrom`]: https://docs.rs/encase/latest/encase/internal/trait.CreateFrom.html

[`impl_vector!`]: https://docs.rs/encase/latest/encase/macro.impl_vector.html
[`impl_matrix!`]: https://docs.rs/encase/latest/encase/macro.impl_matrix.html
[`impl_rts_array!`]: https://docs.rs/encase/latest/encase/macro.impl_rts_array.html
[`impl_wrapper!`]: https://docs.rs/encase/latest/encase/macro.impl_wrapper.html
[derive@ShaderType]: https://docs.rs/encase/latest/encase/derive.ShaderType.html

[`UniformBuffer`]: https://docs.rs/encase/latest/encase/struct.UniformBuffer.html
[`StorageBuffer`]: https://docs.rs/encase/latest/encase/struct.StorageBuffer.html
[`DynamicUniformBuffer`]: https://docs.rs/encase/latest/encase/struct.DynamicUniformBuffer.html
[`DynamicStorageBuffer`]: https://docs.rs/encase/latest/encase/struct.DynamicStorageBuffer.html

[`BufferRef`]: https://docs.rs/encase/latest/encase/internal/trait.BufferRef.html
[`BufferMut`]: https://docs.rs/encase/latest/encase/internal/trait.BufferMut.html
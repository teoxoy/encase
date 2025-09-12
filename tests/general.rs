use encase::{ArrayLength, CalculateSizeFor, ShaderType, StorageBuffer};
use test_impl::*;

macro_rules! gen {
    ($rng:ident, $ty:ty) => {{
        let mut buf = [0; 4];
        use rand::RngCore;
        $rng.fill_bytes(&mut buf);
        <$ty>::from_ne_bytes(buf)
    }};
}

macro_rules! gen_arr {
    ($rng:ident, $ty:ty, $n:literal) => {{
        [(); $n].map(|_| gen!($rng, $ty))
    }};
}

macro_rules! gen_2d_arr {
    ($rng:ident, $ty:ty, $n:literal, $m:literal) => {{
        [[(); $m]; $n].map(|arr| arr.map(|_| gen!($rng, $ty)))
    }};
}

macro_rules! gen_inner {
    ($n:literal, $($tail:tt)*) => {{
        [(); $n].map(|_| $($tail)*)
    }};
}

#[derive(ShaderType)]
struct A {
    f: f32,
    u: u32,
    i: i32,
    nu: Option<core::num::NonZeroU32>,
    ni: Option<core::num::NonZeroI32>,
    wu: core::num::Wrapping<u32>,
    wi: core::num::Wrapping<i32>,
    au: core::sync::atomic::AtomicU32,
    ai: core::sync::atomic::AtomicI32,
    v2: Vec2f,
    v3: Vec3u,
    v4: Vec4i,
    mat2: Mat2x2f,
    mat2x3: Mat2x3f,
    mat2x4: Mat2x4f,
    mat3x2: Mat3x2f,
    mat3: Mat3x3f,
    mat3x4: Mat3x4f,
    mat4x2: Mat4x2f,
    mat4x3: Mat4x3f,
    mat4: Mat4x4f,
    arrf: [f32; 32],
    arru: [u32; 32],
    arri: [i32; 32],
    arrvf: [Vec2f; 16],
    arrvu: [Vec3u; 16],
    arrvi: [Vec4i; 16],
    arrm2: [Mat2x2f; 8],
    arrm3: [Mat3x3f; 8],
    arrm4: [Mat4x4f; 8],
    rt_arr_len: ArrayLength,
    #[shader(size(runtime))]
    rt_arr: Vec<Mat2x3f>,
}

fn gen_a(rng: &mut rand::rngs::StdRng) -> A {
    A {
        f: gen!(rng, f32),
        u: gen!(rng, u32),
        i: gen!(rng, i32),
        nu: core::num::NonZeroU32::new(gen!(rng, u32)),
        ni: core::num::NonZeroI32::new(gen!(rng, i32)),
        wu: core::num::Wrapping(gen!(rng, u32)),
        wi: core::num::Wrapping(gen!(rng, i32)),
        au: core::sync::atomic::AtomicU32::new(gen!(rng, u32)),
        ai: core::sync::atomic::AtomicI32::new(gen!(rng, i32)),
        v2: Vec2f::from(gen_arr!(rng, f32, 2)),
        v3: Vec3u::from(gen_arr!(rng, u32, 3)),
        v4: Vec4i::from(gen_arr!(rng, i32, 4)),
        mat2: Mat2x2f::from(gen_2d_arr!(rng, f32, 2, 2)),
        mat2x3: Mat2x3f::from(gen_2d_arr!(rng, f32, 2, 3)),
        mat2x4: Mat2x4f::from(gen_2d_arr!(rng, f32, 2, 4)),
        mat3x2: Mat3x2f::from(gen_2d_arr!(rng, f32, 3, 2)),
        mat3: Mat3x3f::from(gen_2d_arr!(rng, f32, 3, 3)),
        mat3x4: Mat3x4f::from(gen_2d_arr!(rng, f32, 3, 4)),
        mat4x2: Mat4x2f::from(gen_2d_arr!(rng, f32, 4, 2)),
        mat4x3: Mat4x3f::from(gen_2d_arr!(rng, f32, 4, 3)),
        mat4: Mat4x4f::from(gen_2d_arr!(rng, f32, 4, 4)),
        arrf: gen_arr!(rng, f32, 32),
        arru: gen_arr!(rng, u32, 32),
        arri: gen_arr!(rng, i32, 32),
        arrvf: gen_inner!(16, Vec2f::from(gen_arr!(rng, f32, 2))),
        arrvu: gen_inner!(16, Vec3u::from(gen_arr!(rng, u32, 3))),
        arrvi: gen_inner!(16, Vec4i::from(gen_arr!(rng, i32, 4))),
        arrm2: gen_inner!(8, Mat2x2f::from(gen_2d_arr!(rng, f32, 2, 2))),
        arrm3: gen_inner!(8, Mat3x3f::from(gen_2d_arr!(rng, f32, 3, 3))),
        arrm4: gen_inner!(8, Mat4x4f::from(gen_2d_arr!(rng, f32, 4, 4))),
        rt_arr_len: ArrayLength,
        rt_arr: vec![Mat2x3f::from(gen_2d_arr!(rng, f32, 2, 3)); 64],
    }
}

#[test]
fn size() {
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(1234);
    let a = gen_a(&mut rng);

    assert_eq!(a.size().get(), 4560);
}

#[test]
fn calculate_size_for() {
    assert_eq!(<&A>::calculate_size_for(12).get(), 2896);
}

#[test]
fn all_types() {
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(1234);
    let a = gen_a(&mut rng);

    let mut raw_buffer = Vec::new();
    let mut buffer = StorageBuffer::new(&mut raw_buffer);
    buffer.write(&a).unwrap();

    let mut a_clone: A = buffer.create().unwrap();
    let mut raw_buffer_2 = Vec::new();
    let mut buffer_2 = StorageBuffer::new(&mut raw_buffer_2);
    buffer_2.write(&a_clone).unwrap();

    assert_eq!(buffer.as_ref(), buffer_2.as_ref());

    a_clone.rt_arr.truncate(10);
    // a_clone.rt_arr.reserve_exact(0);
    buffer_2.read(&mut a_clone).unwrap();
    buffer_2.write(&a_clone).unwrap();

    assert_eq!(raw_buffer, raw_buffer_2);
}

#[test]
fn test_opt_writing() {
    let one = 1_u32;
    let two = 2_u32;
    let data = [&one, &two];
    let data2 = [one, two];
    let mut in_byte_buffer: Vec<u8> = Vec::new();
    let mut in_byte_buffer2: Vec<u8> = Vec::new();
    let mut in_buffer = StorageBuffer::new(&mut in_byte_buffer);
    let mut in_buffer2 = StorageBuffer::new(&mut in_byte_buffer2);
    in_buffer.write(&data).unwrap();
    in_buffer2.write(&data2).unwrap();
    assert_eq!(in_byte_buffer, in_byte_buffer2);
}

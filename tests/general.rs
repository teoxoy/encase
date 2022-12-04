use encase::{ArrayLength, CalculateSizeFor, ShaderType, StorageBuffer};

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
    v2: mint::Vector2<f32>,
    v3: mint::Vector3<u32>,
    v4: mint::Vector4<i32>,
    p2: mint::Point2<f32>,
    p3: mint::Point3<f32>,
    mat2: mint::ColumnMatrix2<f32>,
    mat2x3: mint::ColumnMatrix2x3<f32>,
    mat2x4: mint::ColumnMatrix2x4<f32>,
    mat3x2: mint::ColumnMatrix3x2<f32>,
    mat3: mint::ColumnMatrix3<f32>,
    mat3x4: mint::ColumnMatrix3x4<f32>,
    mat4x2: mint::ColumnMatrix4x2<f32>,
    mat4x3: mint::ColumnMatrix4x3<f32>,
    mat4: mint::ColumnMatrix4<f32>,
    arrf: [f32; 32],
    arru: [u32; 32],
    arri: [i32; 32],
    arrvf: [mint::Vector2<f32>; 16],
    arrvu: [mint::Vector3<u32>; 16],
    arrvi: [mint::Vector4<i32>; 16],
    arrm2: [mint::ColumnMatrix2<f32>; 8],
    arrm3: [mint::ColumnMatrix3<f32>; 8],
    arrm4: [mint::ColumnMatrix4<f32>; 8],
    rt_arr_len: ArrayLength,
    #[size(runtime)]
    rt_arr: Vec<mint::ColumnMatrix2x3<f32>>,
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
        v2: mint::Vector2::from(gen_arr!(rng, f32, 2)),
        v3: mint::Vector3::from(gen_arr!(rng, u32, 3)),
        v4: mint::Vector4::from(gen_arr!(rng, i32, 4)),
        p2: mint::Point2::from(gen_arr!(rng, f32, 2)),
        p3: mint::Point3::from(gen_arr!(rng, f32, 3)),
        mat2: mint::ColumnMatrix2::from(gen_2d_arr!(rng, f32, 2, 2)),
        mat2x3: mint::ColumnMatrix2x3::from(gen_2d_arr!(rng, f32, 3, 2)),
        mat2x4: mint::ColumnMatrix2x4::from(gen_2d_arr!(rng, f32, 4, 2)),
        mat3x2: mint::ColumnMatrix3x2::from(gen_2d_arr!(rng, f32, 2, 3)),
        mat3: mint::ColumnMatrix3::from(gen_2d_arr!(rng, f32, 3, 3)),
        mat3x4: mint::ColumnMatrix3x4::from(gen_2d_arr!(rng, f32, 4, 3)),
        mat4x2: mint::ColumnMatrix4x2::from(gen_2d_arr!(rng, f32, 2, 4)),
        mat4x3: mint::ColumnMatrix4x3::from(gen_2d_arr!(rng, f32, 3, 4)),
        mat4: mint::ColumnMatrix4::from(gen_2d_arr!(rng, f32, 4, 4)),
        arrf: gen_arr!(rng, f32, 32),
        arru: gen_arr!(rng, u32, 32),
        arri: gen_arr!(rng, i32, 32),
        arrvf: gen_inner!(16, mint::Vector2::from(gen_arr!(rng, f32, 2))),
        arrvu: gen_inner!(16, mint::Vector3::from(gen_arr!(rng, u32, 3))),
        arrvi: gen_inner!(16, mint::Vector4::from(gen_arr!(rng, i32, 4))),
        arrm2: gen_inner!(8, mint::ColumnMatrix2::from(gen_2d_arr!(rng, f32, 2, 2))),
        arrm3: gen_inner!(8, mint::ColumnMatrix3::from(gen_2d_arr!(rng, f32, 3, 3))),
        arrm4: gen_inner!(8, mint::ColumnMatrix4::from(gen_2d_arr!(rng, f32, 4, 4))),
        rt_arr_len: ArrayLength,
        rt_arr: vec![mint::ColumnMatrix2x3::from(gen_2d_arr!(rng, f32, 3, 2)); 64],
    }
}

#[test]
fn size() {
    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(1234);
    let a = gen_a(&mut rng);

    assert_eq!(a.size().get(), 4080);
}

#[test]
fn calculate_size_for() {
    assert_eq!(<&A>::calculate_size_for(12).get(), 2832);
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

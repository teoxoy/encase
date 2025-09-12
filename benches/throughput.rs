use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use encase::{ShaderType, StorageBuffer};
use test_impl::*;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

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

#[derive(Debug, ShaderType, PartialEq, Clone, Copy)]
struct A {
    f: f32,
    u: u32,
    i: i32,
    nu: Option<core::num::NonZeroU32>,
    ni: Option<core::num::NonZeroI32>,
    wu: core::num::Wrapping<u32>,
    wi: core::num::Wrapping<i32>,
    // au: core::sync::atomic::AtomicU32,
    // ai: core::sync::atomic::AtomicI32,
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
    #[shader(size(1600))]
    _pad: u32,
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
        _pad: gen!(rng, u32),
    }
}

const _: () = const_panic::concat_assert!(
    A::METADATA.min_size().get() == 4096,
    A::METADATA.min_size().get()
);

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("Throughput");

    use rand::SeedableRng;
    let mut rng = rand::rngs::StdRng::seed_from_u64(1234);

    let a = gen_a(&mut rng);

    const KB: usize = 1024;
    const MB: usize = KB * KB;
    // const GB: usize = MB * KB;

    let sizes = [
        // ("16B", 16),
        // ("128B", 128),
        // ("1KiB", KB),
        ("16KiB", 16 * KB),
        ("128KiB", 128 * KB),
        ("1MiB", MB),
        ("16MiB", 16 * MB),
        ("512MiB", 512 * MB),
    ];
    for (name, size) in sizes {
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_function(format!("{name}_write"), |b| {
            b.iter_batched_ref(
                || create_vecs(a, size),
                |(src, dst)| dst.write(src).unwrap(),
                criterion::BatchSize::LargeInput,
            );
        });
        group.bench_function(format!("{name}_read"), |b| {
            b.iter_batched_ref(
                || create_vecs(a, size),
                |(src, dst)| dst.read(src).unwrap(),
                criterion::BatchSize::LargeInput,
            );
        });
        group.bench_function(format!("{name}_create"), |b| {
            b.iter_batched_ref(
                || create_vecs(a, size),
                |(_src, dst)| dst.create::<Vec<A>>().unwrap(),
                criterion::BatchSize::LargeInput,
            );
        });
        group.bench_function(format!("{name}_manual"), |b| {
            b.iter_batched_ref(
                || create_aligned_vecs(size),
                |(dst, src)| manual_memcpy(dst, src),
                criterion::BatchSize::LargeInput,
            );
        });
        group.bench_function(format!("{name}_stdlib"), |b| {
            b.iter_batched_ref(
                || create_aligned_vecs(size),
                |(dst, src)| dst.copy_from_slice(src),
                criterion::BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

fn manual_memcpy(src: &mut [u8], dst: &[u8]) {
    assert_eq!(src.len(), dst.len());
    #[allow(clippy::manual_memcpy)]
    for i in 0..src.len() {
        src[i] = dst[i];
    }
}

fn create_aligned_vecs(size: usize) -> (Vec<u8>, Vec<u8>) {
    let src = vec![1u8; size];
    let dst = vec![0u8; size];
    assert_eq!(src.as_ptr() as usize % 8, 0);
    assert_eq!(dst.as_ptr() as usize % 8, 0);
    (src, dst)
}

fn create_vecs(a: A, size: usize) -> (Vec<A>, StorageBuffer<Vec<u8>>) {
    let src = vec![a; size / A::min_size().get() as usize];
    let dst = StorageBuffer::new(vec![0u8; size]);
    (src, dst)
}

#[cfg(target_family = "unix")]
criterion_group! {
    name = benches;
    config = Criterion::default()
        .with_profiler(pprof::criterion::PProfProfiler::new(100, pprof::criterion::Output::Flamegraph(None)));
    targets = bench
}
#[cfg(not(target_family = "unix"))]
criterion_group! {
    name = benches;
    config = Criterion::default();
    targets = bench
}
criterion_main!(benches);

use encase::{ArrayLength, ShaderType, StorageBuffer};
use futures::executor::block_on;
use mint::{Vector2, Vector3};
use wgpu::{include_wgsl, util::DeviceExt};

#[derive(Debug, ShaderType, PartialEq)]
struct A {
    u: u32,
    v: u32,
    w: Vector2<u32>,
    #[size(16)]
    #[align(8)]
    x: u32,
    xx: u32,
}

#[derive(Debug, ShaderType, PartialEq)]
struct B {
    a: Vector2<u32>,
    b: Vector3<u32>,
    c: u32,
    d: u32,
    #[align(16)]
    e: A,
    f: Vector3<u32>,
    g: [A; 3],
    h: i32,
    #[align(32)]
    #[size(runtime)]
    i: Vec<A>,
}

#[test]
fn test_wgpu() {
    let b = B {
        a: Vector2 { x: 45, y: 564 },
        b: Vector3 {
            x: 465,
            y: 56664,
            z: 5646,
        },
        c: 4,
        d: 3,
        e: A {
            u: 5,
            v: 566,
            w: Vector2 { x: 4345, y: 43564 },
            x: 5444,
            xx: 305444,
        },
        f: Vector3 {
            x: 455465,
            y: 55665464,
            z: 5564546,
        },
        g: [
            A {
                u: 105,
                v: 10566,
                w: Vector2 {
                    x: 14345,
                    y: 143564,
                },
                x: 105444,
                xx: 305444,
            },
            A {
                u: 205,
                v: 20566,
                w: Vector2 {
                    x: 24345,
                    y: 243564,
                },
                x: 205444,
                xx: 305444,
            },
            A {
                u: 305,
                v: 30566,
                w: Vector2 {
                    x: 34345,
                    y: 343564,
                },
                x: 305444,
                xx: 305444,
            },
        ],
        h: 5,
        i: Vec::from([A {
            u: 205,
            v: 20566,
            w: Vector2 {
                x: 24345,
                y: 243564,
            },
            x: 205444,
            xx: 305444,
        }]),
    };

    let mut in_byte_buffer = Vec::new();
    let mut in_buffer = StorageBuffer::new(&mut in_byte_buffer);

    in_buffer.write(&b).unwrap();
    assert_eq!(in_byte_buffer.len(), b.size().get() as _);

    let shader = include_wgsl!("./shaders/general.wgsl");
    let out_byte_buffer = in_out::<B, B>(&shader, &in_byte_buffer, false);

    assert_eq!(in_byte_buffer, out_byte_buffer);

    let out_buffer = StorageBuffer::new(out_byte_buffer);
    let out_val: B = out_buffer.create().unwrap();
    assert_eq!(b, out_val);
}

#[test]
fn array_length() {
    #[derive(Debug, ShaderType, PartialEq)]
    struct A {
        array_length: ArrayLength,
        array_length_call_ret_val: u32,
        a: Vector3<u32>,
        #[align(16)]
        #[size(runtime)]
        arr: Vec<u32>,
    }

    let in_value = A {
        array_length: ArrayLength,
        array_length_call_ret_val: 4,
        a: Vector3 { x: 5, y: 4, z: 6 },
        arr: vec![45],
    };

    let mut in_byte_buffer = Vec::new();
    let mut in_buffer = StorageBuffer::new(&mut in_byte_buffer);

    in_buffer.write(&in_value).unwrap();
    assert_eq!(in_byte_buffer.len(), in_value.size().get() as _);

    let shader = include_wgsl!("./shaders/array_length.wgsl");
    let out_byte_buffer = in_out::<A, A>(&shader, &in_byte_buffer, false);

    assert_eq!(in_byte_buffer, out_byte_buffer);

    let out_buffer = StorageBuffer::new(out_byte_buffer);
    let out_val: A = out_buffer.create().unwrap();

    assert_eq!(in_value, out_val);
}

fn in_out<IN: encase::ShaderType, OUT: encase::ShaderType>(
    shader: &wgpu::ShaderModuleDescriptor,
    data: &[u8],
    is_uniform: bool,
) -> Vec<u8> {
    let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);
    let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::default(),
        ..Default::default()
    }))
    .unwrap();

    println!("Adapter info: {:#?}", adapter.get_info());

    let (device, queue) =
        block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None)).unwrap();

    let input_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Input Buffer"),
        contents: data,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::UNIFORM,
    });

    let output_gpu_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Output Buffer"),
        size: data.len() as _,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: if is_uniform {
                        wgpu::BufferBindingType::Uniform
                    } else {
                        wgpu::BufferBindingType::Storage { read_only: true }
                    },
                    has_dynamic_offset: false,
                    min_binding_size: Some(IN::min_size()),
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: false },
                    has_dynamic_offset: false,
                    min_binding_size: Some(OUT::min_size()),
                },
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let shader = device.create_shader_module(shader);

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: "main",
    });

    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_gpu_buffer.as_entire_binding(),
            },
        ],
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch(1, 1, 1);
    }

    queue.submit(core::iter::once(encoder.finish()));

    let output_slice = output_gpu_buffer.slice(..);
    let output_future = output_slice.map_async(wgpu::MapMode::Read);

    device.poll(wgpu::Maintain::Wait);
    block_on(output_future).unwrap();

    let output = output_slice.get_mapped_range().to_vec();
    output_gpu_buffer.unmap();
    output
}

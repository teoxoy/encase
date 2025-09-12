#![cfg(not(miri))] // Can't run wgpu through miri

use encase::{ArrayLength, ShaderType, StorageBuffer};
use futures::executor::block_on;
use test_impl::*;
use wgpu::{include_wgsl, util::DeviceExt};

#[derive(Debug, ShaderType, PartialEq)]
struct A {
    u: u32,
    v: u32,
    w: Vec2u,
    #[shader(size(16), align(8))]
    x: u32,
    xx: u32,
}

#[derive(Debug, ShaderType, PartialEq)]
struct B {
    a: Vec2u,
    b: Vec3u,
    c: u32,
    d: u32,
    #[shader(align(16))]
    e: A,
    f: Vec3u,
    g: [A; 3],
    h: i32,
    #[shader(align(32), size(runtime))]
    i: Vec<A>,
}

#[test]
fn test_wgpu() {
    let b = B {
        a: Vec2u::from([45, 564]),
        b: Vec3u::from([465, 56664, 5646]),
        c: 4,
        d: 3,
        e: A {
            u: 5,
            v: 566,
            w: Vec2u::from([4345, 43564]),
            x: 5444,
            xx: 305444,
        },
        f: Vec3u::from([455465, 55665464, 5564546]),
        g: [
            A {
                u: 105,
                v: 10566,
                w: Vec2u::from([14345, 143564]),
                x: 105444,
                xx: 305444,
            },
            A {
                u: 205,
                v: 20566,
                w: Vec2u::from([24345, 243564]),
                x: 205444,
                xx: 305444,
            },
            A {
                u: 305,
                v: 30566,
                w: Vec2u::from([34345, 343564]),
                x: 305444,
                xx: 305444,
            },
        ],
        h: 5,
        i: Vec::from([A {
            u: 205,
            v: 20566,
            w: Vec2u::from([24345, 243564]),
            x: 205444,
            xx: 305444,
        }]),
    };

    let mut in_byte_buffer = Vec::new();
    let mut in_buffer = StorageBuffer::new(&mut in_byte_buffer);

    in_buffer.write(&b).unwrap();
    assert_eq!(in_byte_buffer.len(), b.size().get() as _);

    let shader = include_wgsl!("./shaders/general.wgsl");
    let out_byte_buffer = in_out::<B, B>(shader, &in_byte_buffer, false);

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
        a: Vec3u,
        #[shader(align(16), size(runtime))]
        arr: Vec<u32>,
    }

    let in_value = A {
        array_length: ArrayLength,
        array_length_call_ret_val: 4,
        a: Vec3u::from([5, 4, 6]),
        arr: vec![45],
    };

    let mut in_byte_buffer = Vec::new();
    let mut in_buffer = StorageBuffer::new(&mut in_byte_buffer);

    in_buffer.write(&in_value).unwrap();
    assert_eq!(in_byte_buffer.len(), in_value.size().get() as _);

    let shader = include_wgsl!("./shaders/array_length.wgsl");
    let out_byte_buffer = in_out::<A, A>(shader, &in_byte_buffer, false);

    assert_eq!(in_byte_buffer, out_byte_buffer);

    let out_buffer = StorageBuffer::new(out_byte_buffer);
    let out_val: A = out_buffer.create().unwrap();

    assert_eq!(in_value, out_val);
}

fn in_out<IN: encase::ShaderType, OUT: encase::ShaderType>(
    shader: wgpu::ShaderModuleDescriptor,
    data: &[u8],
    is_uniform: bool,
) -> Vec<u8> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });
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
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let mapping_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Mapping Buffer"),
        size: data.len() as _,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
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
        entry_point: Some("main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
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
        cpass.dispatch_workgroups(1, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&output_gpu_buffer, 0, &mapping_buffer, 0, data.len() as _);

    queue.submit(core::iter::once(encoder.finish()));

    let output_slice = mapping_buffer.slice(..);
    output_slice.map_async(wgpu::MapMode::Read, |_| {});

    device.poll(wgpu::Maintain::Wait);

    let output = output_slice.get_mapped_range().to_vec();
    mapping_buffer.unmap();
    output
}

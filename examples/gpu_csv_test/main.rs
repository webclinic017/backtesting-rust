use wgpu;
use std::{borrow::Cow, str::FromStr};
use std::borrow::Borrow;
use bytemuck::{Pod, Zeroable, cast_slice};
use wgpu::util::DeviceExt;
use pollster;
use std::time::Instant;
use rand::prelude::*;

struct ComputeModel {
    device: wgpu::Device,
    queue: wgpu::Queue,
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,

    storage_buffer: Option<wgpu::Buffer>,
    output_buffer: Option<wgpu::Buffer>,
    output_buffer_length: Option<usize>,

    compute_pipeline_layout: Option<wgpu::PipelineLayout>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,

    compute_pipeline: Option<wgpu::ComputePipeline>,
    bind_group: Option<wgpu::BindGroup>,
}
impl ComputeModel {
    pub async fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::Backends::all());

        // `request_adapter` instantiates the general connection to the GPU
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .await
            .unwrap();

        let bind_group_layout = Some(device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage {
                        read_only: false,
                    },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: false,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        }));

        let compute_pipeline_layout = Some(device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout.as_ref().unwrap()],
            push_constant_ranges: &[],
        }));

        Self {
            device,
            queue,
            instance,
            adapter,

            storage_buffer: None,
            output_buffer: None,
            output_buffer_length: None,

            compute_pipeline_layout,
            bind_group_layout,

            bind_group: None,
            compute_pipeline: None,
        }
    }

    pub fn initialize_buffers<T>(&mut self, numbers: &[T], output_length: usize) where T: Pod + Zeroable {
        self.storage_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer"),
            contents: bytemuck::cast_slice(numbers),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        }));

        let ouput_buffer_prealloc: Vec<f32> = vec![f32::NAN; output_length];
        self.output_buffer_length = Some(output_length);

        self.output_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Output Buffer"),
            contents: bytemuck::cast_slice(ouput_buffer_prealloc.as_slice()),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        }));
    }

    pub fn initialize_pipeline(&mut self) -> Result<(), String> {
        // Instantiates the pipeline.
        let cs_module = self.device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        self.compute_pipeline = Some(self.device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&self.compute_pipeline_layout.as_ref().unwrap()),
            module: &cs_module,
            entry_point: "main",
        }));

        self.bind_group = Some(self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &self.bind_group_layout.as_ref().unwrap(),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: self.storage_buffer.as_ref().unwrap().as_entire_binding(),
            },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.output_buffer.as_ref().unwrap().as_entire_binding(),
                }
            ],
        }));

        Ok(())
    }
    // async fn execute_gpu(&self, numbers: &[f32], output_length: u8) -> Option<Vec<f32>> {
    //     let output_size = output_length as usize * std::mem::size_of::<f32>();
    //     let staging_buffer_size = output_size as wgpu::BufferAddress;
    //
    //     let staging_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
    //         label: None,
    //         size: staging_buffer_size,
    //         usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
    //         mapped_at_creation: false,
    //     });
    //
    //     let mut encoder =
    //         self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    //     {
    //         let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
    //         cpass.set_pipeline(&self.compute_pipeline.as_ref().unwrap());
    //         cpass.set_bind_group(0, &self.bind_group.as_ref().unwrap(), &[]);
    //         cpass.insert_debug_marker("compute");
    //         cpass.dispatch(output_length as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
    //     }
    //     // Will copy data from storage buffer on GPU to staging buffer on CPU.
    //     encoder.copy_buffer_to_buffer(&self.output_buffer.as_ref().unwrap(), 0, &staging_buffer, 0, staging_buffer_size);
    //
    //     let timer = Instant::now();
    //     // Submits command encoder for processing
    //     self.queue.submit(Some(encoder.finish()));
    //
    //     let buffer_slice = staging_buffer.slice(..);
    //     let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);
    //
    //     // Poll the device in a blocking manner so that our future resolves.
    //     // In an actual application, `device.poll(...)` should
    //     // be called in an event loop or on another thread.
    //     self.device.poll(wgpu::Maintain::Wait);
    //
    //     if let Ok(()) = buffer_future.await {
    //         let data = buffer_slice.get_mapped_range();
    //         let result = bytemuck::cast_slice(&data).to_vec();
    //
    //         drop(data);
    //         staging_buffer.unmap(); // Unmaps buffer from memory
    //         println!("{:2}", timer.elapsed().as_secs_f32());
    //         println!("{:?}", result);
    //         Some(result)
    //     } else {
    //         panic!("failed to run compute on gpu!")
    //     }
    // }
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct DataStruct {
    _f1: f32,
    _f2: f32,
}

fn main() -> Result<(), String>{
    let mut rng = thread_rng();
    // let numbers: Vec<f32> = (0..1_000).map(|_| rng.gen::<f32>()*100.0).collect();
    let numbers: Vec<DataStruct> = (0..1_000).map(|_| DataStruct {_f1: rng.gen::<f32>()*100.0, _f2: rng.gen::<f32>()*100.0}).collect();
    let output_length = 5;

    let mut cs_model = pollster::block_on(ComputeModel::new());
    cs_model.initialize_buffers(&numbers, output_length);
    cs_model.initialize_pipeline()?;

    pollster::block_on(run(&cs_model));

    Ok(())
}

async fn run(cs_model: &ComputeModel) -> Option<Vec<f32>> {
    // let output_size = output_length as usize * std::mem::size_of::<f32>();
    let staging_buffer_size = (cs_model.output_buffer_length.unwrap() * std::mem::size_of::<f32>()) as wgpu::BufferAddress;

    let staging_buffer = cs_model.device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: staging_buffer_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let mut encoder =
        cs_model.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: None });
        cpass.set_pipeline(&cs_model.compute_pipeline.as_ref().unwrap());
        cpass.set_bind_group(0, &cs_model.bind_group.as_ref().unwrap(), &[]);
        cpass.insert_debug_marker("compute");
        cpass.dispatch(cs_model.output_buffer_length.unwrap() as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
    }
    // Will copy data from storage buffer on GPU to staging buffer on CPU.
    encoder.copy_buffer_to_buffer(&cs_model.output_buffer.as_ref().unwrap(), 0, &staging_buffer, 0, staging_buffer_size);

    let timer = Instant::now();
    // Submits command encoder for processing
    cs_model.queue.submit(Some(encoder.finish()));

    let buffer_slice = staging_buffer.slice(..);
    let buffer_future = buffer_slice.map_async(wgpu::MapMode::Read);

    // Poll the device in a blocking manner so that our future resolves.
    // In an actual application, `device.poll(...)` should
    // be called in an event loop or on another thread.
    cs_model.device.poll(wgpu::Maintain::Wait);

    if let Ok(()) = buffer_future.await {
        let data = buffer_slice.get_mapped_range();
        let result = bytemuck::cast_slice(&data).to_vec();

        drop(data);
        staging_buffer.unmap(); // Unmaps buffer from memory
        println!("{:2}", timer.elapsed().as_secs_f32());
        println!("{:?}", result);
        Some(result)
    } else {
        panic!("failed to run compute on gpu!")
    }
}

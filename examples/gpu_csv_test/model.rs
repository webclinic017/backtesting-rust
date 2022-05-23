use wgpu;
use std::{borrow::Cow, str::FromStr};
use std::borrow::Borrow;
use bytemuck::{Pod, Zeroable, cast_slice};
use wgpu::util::DeviceExt;
use std::time::Instant;

pub struct ComputeModel {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,

    pub storage_buffer: Option<wgpu::Buffer>,
    pub output_buffer: Option<wgpu::Buffer>,
    pub output_buffer_length: Option<usize>,

    pub compute_pipeline_layout: Option<wgpu::PipelineLayout>,
    pub bind_group_layout: Option<wgpu::BindGroupLayout>,

    pub compute_pipeline: Option<wgpu::ComputePipeline>,
    pub bind_group: Option<wgpu::BindGroup>,
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

    pub fn initialize_buffers<T, U>(&mut self, data_in: &[T], data_out_prealloc: &[U])
        where T: Pod + Zeroable, U: Pod + Zeroable {
        self.storage_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Storage Buffer"),
            contents: bytemuck::cast_slice(data_in),
            usage: wgpu::BufferUsages::STORAGE
                | wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::COPY_SRC,
        }));

        // let ouput_buffer_prealloc: Vec<f32> = vec![f32::NAN; output_length];
        // self.output_buffer_length = Some(output_length);
        self.output_buffer_length = Some(data_out_prealloc.len());

        self.output_buffer = Some(self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Output Buffer"),
            contents: bytemuck::cast_slice(data_out_prealloc),
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
}
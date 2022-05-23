mod model;

use wgpu;
use bytemuck::{Pod, Zeroable};
use model::ComputeModel;
use pollster;
use rand::prelude::*;
use std::time::Instant;


#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct DataIn {
    _f1: f32,
    _f2: f32,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct DataOut {
    _f1: f32,
}

fn main() -> Result<(), String>{
    let mut rng = thread_rng();
    let data_in: Vec<DataIn> = (0..1_000_000).map(|_| DataIn {_f1: rng.gen::<f32>()*100.0, _f2: rng.gen::<f32>()*100.0}).collect();
    let data_out_prealloc: Vec<DataOut> = vec![DataOut{ _f1: f32::NAN }; 75];

    let mut cs_model = pollster::block_on(ComputeModel::new());
    cs_model.initialize_buffers(&data_in, &data_out_prealloc);
    cs_model.initialize_pipeline()?;

    let result = pollster::block_on(run(&cs_model)).unwrap();
    let r: Vec<f32> = result.into_iter()
        .filter(|&x| !x.is_nan())
        .collect();
    let l = r.len();
    for i in 1..l {
        assert_eq!(r[i], (r[i-1] + 1.0), "{i}");
    }
    println!("{:?}", l);

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

        let y = cs_model.output_buffer_length.unwrap() as f32 / 65_535.0;
        cpass.dispatch(65_535, y.ceil() as u32, 1); // Number of cells to run, the (x,y,z) size of item being processed
        // cpass.dispatch(cs_model.output_buffer_length.unwrap() as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
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
        Some(result)
    } else {
        panic!("failed to run compute on gpu!")
    }
}

mod model;

use wgpu;
use model::{ComputeModel, DataIn, DataOut};
use pollster;
use rand::prelude::*;
use std::time::Instant;

fn main() -> Result<(), String>{
    let data_in: Vec<DataIn> = (0..4).map(|i| DataIn::new(i as f32, 0.0)).collect();
    let data_out_prealloc: Vec<DataOut> = (0..20).map(|i| DataOut::new(i, i)).collect();

    let mut cs_model = pollster::block_on(ComputeModel::new());
    cs_model.initialize_buffers(&data_in, &data_out_prealloc);
    cs_model.initialize_pipeline()?;

    let result = pollster::block_on(cs_model.run()).unwrap();
    let r: Vec<DataOut> = result.into_iter()
        // .filter(|&x| !x.mean().is_nan())
        .collect();
    println!("{:?}", r);

    Ok(())
}
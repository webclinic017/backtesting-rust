mod model;
use log4rs;
use log::info;
use wgpu;
use model::{ComputeModel, DataIn, DataOut};
use pollster;
use rand::prelude::*;
use std::time::Instant;
use backtesting::utils::read_csv;
use serde_derive::Deserialize;
use itertools::Itertools;

#[derive(Deserialize, Clone, Debug)]
struct Row {
    datetime_str: String,
    value: f32,
}

fn main() -> Result<(), String>{
    log4rs::init_file("config/debug_log4rs.yaml", Default::default()).unwrap();

    // let data_out_prealloc: Vec<DataOut> = (0..20).map(|i| DataOut::new(i, i)).collect();
    let data_out_prealloc: Vec<DataOut> = (120..480).cartesian_product(3..360)
        .map(|(st,i)| DataOut::new(st, i))
        .collect();
    println!("Running {} iterations", data_out_prealloc.len());

    let file_name = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\examples\\gpu_csv_test\\data.csv";
    let data: Vec<Row> = read_csv(file_name).unwrap();
    let data_in: Vec<DataIn> = data.into_iter().map(|r| DataIn::new(r.value)).collect();

    let mut cs_model = pollster::block_on(ComputeModel::new());
    cs_model.initialize_buffers(&data_in, &data_out_prealloc);
    cs_model.initialize_pipeline()?;

    let result = pollster::block_on(cs_model.run()).unwrap();
    let r: Vec<DataOut> = result.into_iter()
        // .filter(|&x| !x.mean().is_nan())
        .collect();
    // println!("{:?}", r);

    Ok(())
}
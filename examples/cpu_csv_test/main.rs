use std::borrow::Borrow;
use std::error::Error;
use log4rs;
use log::info;
use std::time::Instant;
use backtesting::utils::{read_csv, write_csv};
use backtesting::strategy::FieldsToStrings;
use serde_derive::Deserialize;
use itertools::Itertools;
use backtesting::vector_utils::vec_mean;
use rayon::prelude::*;
use crate::date_time::DateTime;

mod date_time;

const MINS_IN_DAY: u32 = 60*24;

#[derive(Deserialize, Debug)]
struct Row {
    datetime: DateTime,
    value: f32,
}

struct InputStruct {
    st: u32,
    interval: u32,
}

fn main() -> Result<(), String> {
    log4rs::init_file("config/debug_log4rs.yaml", Default::default()).unwrap();

    let inputs: Vec<InputStruct> = ((5 * 60)..(16 * 60)).cartesian_product(3..(9 * 60))
        .filter(|(st, i)| (st + i) <= (17 * 60))
        .map(|(st, interval)| InputStruct {st, interval})
        .collect();

    let file_name = "./examples/cpu_csv_test/data.csv";
    let data: Vec<Row> = read_csv(file_name).unwrap();

    println!("{:?}", data.get(1).unwrap());

    let n_days = {
        let mut n = 0;
        let mut last: u32 = 0;
        for r in data.iter() {
            let x = r.datetime.year << 9 | r.datetime.month << 5 | r.datetime.day;
            if x != last {
                n += 1;
                last = x;
            }
        }
        n
    };

    let now = Instant::now();
    println!("Running now");
    let results: Vec<OutputStruct> = inputs.par_iter()
        .map(|x| run_strat(x, &data, n_days))
        .collect();

    println!("{:?}", results.len());
    println!("{:?}", now.elapsed().as_secs_f32());

    static FIELD_NAMES: [&str; 5] = ["start", "interval", "mean", "std", "sharpe"];
    let output_loc = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\examples\\cpu_csv_test\\output.csv";
    match write_csv(&results, &FIELD_NAMES, output_loc) {
        Ok(()) => (),
        Err(e) => panic!("write csv error {}", e),
    };

    Ok(())
}

struct OutputStruct {
    st: u32,
    interval: u32,
    mean: f32,
    stddev: f32,
    sharpe: f32,
}
impl FieldsToStrings for OutputStruct {
    fn fields_to_strings(&self) -> Vec<String> {
        vec![self.st.to_string(), self.interval.to_string(), self.mean.to_string(), self.stddev.to_string(), self.sharpe.to_string()]
    }
}

fn run_strat(input: &InputStruct, data: &Vec<Row>, n_days: u32) -> OutputStruct {
    // let days_in_buffer = 5781;
    // println!("{}", days_in_buffer);

    let mut returns: Vec<f32> = Vec::new();
    let N = data.len();

    let mut mean_summand = 0.0;

    for i in 0..n_days {
        let start = (i*MINS_IN_DAY + input.st) as usize;
        let end = start + input.interval as usize;
        if end >= N { break; }

        let ret: f32 = data[end].value - data[start].value;
        returns.push(ret);
        mean_summand += ret;
    }
    let mean = mean_summand / n_days as f32;

    let stddev = vec_std(&returns, mean);

    OutputStruct {
        st: input.st,
        interval: input.interval,
        mean,
        stddev,
        sharpe: mean*252.0/stddev,
    }
}

fn vec_std(v: &Vec<f32>, mean: f32) -> f32 {
    let n = v.len() as f32;
    if n == 0.0 { return f32::NAN }
    let mut s: f32 = 0.0;
    for x in v.iter() {
        let t = x - mean;
        s = s + (t*t);
    }
    s = s / n;
    s.sqrt()
}


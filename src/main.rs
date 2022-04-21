// use std::collections::HashMap;
// use std::io::prelude::*;
use rustc_hash::FxHashMap;
use std::error::Error;
use chrono::{Datelike, NaiveDateTime, NaiveTime};
use backtesting::utils::*;
use backtesting::vector_utils::{vec_mean, vec_std, vec_unique, vec_where_eq};
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>>  {
    // let file_name = "ZN_continuous_1min_sample.csv";
    // let file_name = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\ZN_continuous_adjusted_1min_tail.csv";
    let file_name = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\ZN_continuous_adjusted_1min.csv";
    println!("{}", file_name);

    let v: Vec<Row> = read_csv(file_name).unwrap()
        .into_iter()
        .filter(|x| x.datetime().year() >= 2021)
        .collect();

    let now = Instant::now();
    run_analysis(&v);
    println!("{} seconds to run", now.elapsed().as_secs());
    Ok(())

}


fn run_analysis(v: &Vec<Row>) -> Result<(), Box<dyn Error>> {
    let times: Vec<NaiveDateTime> = v.iter().map(|x| x.datetime()).collect();
    let values: Vec<_> = v.iter().map(|x| x.close).collect();
    // let timeseries: HashMap<_, _> = times.iter().zip(&values).collect();

    let resolution: u64 = 1; // minutes
    let interval_rng: Vec<u64> = (2..60).map(|x| x*resolution).collect();
    let start_time_rng: Vec<NaiveTime> = time_range((8,0,0), (10,30,0), resolution);

    let total_runs = interval_rng.len()*start_time_rng.len();
    println!("Running {} times", total_runs);

    let mut sharpes: FxHashMap<(u64, NaiveTime, NaiveTime), f64> = FxHashMap::default();
    let mut progress_counter: usize = 0;
    let now = Instant::now();
    for interval in interval_rng {
        for start_time in &start_time_rng {
            progress_counter += 1;
            if progress_counter % 100 == 0 {
                println!("Running iteration {} out of {}, {} seconds elapsed",
                         progress_counter, total_runs, now.elapsed().as_secs());
            }
            let end_time = add_time(start_time, interval*60);

            let r:Vec<i32> = vec![0; values.len()];

            let entry_cond: Vec<i32> = times.iter().map(|x| if x.time()==*start_time {1} else {0}).collect();
            let r: Vec<i32> = r.iter().zip(entry_cond.iter()).map(|(&x, &y)| x+y).collect();

            let exit_cond: Vec<i32> = times.iter().map(|x| if x.time()==end_time {-1} else {0}).collect();
            let r: Vec<i32> = r.iter().zip(exit_cond.iter()).map(|(&x, &y)| x+y).collect();

            let r: Vec<usize> = fill_ids(&r);

            let mut returns: Vec<f64> = Vec::new();
            for i in vec_unique(&r).into_iter() {
                if i == &0 {continue;} // 0 means no observation
                let ix = vec_where_eq(&r, i);
                let _t = &times[ix[0]..=ix[ix.len()-1]];
                let v:Vec<f64> = values[ix[0]..=ix[ix.len()-1]].to_vec();

                if v.len() > 1 {
                    // let ret = v[v.len() - 1] - v[0];
                    // returns.insert((interval, start_time), ret);
                    // sharpes.insert((interval, start_time), sharpe);
                    returns.push(v[v.len()-1] - v[0]);
                }
            }
            let sharpe = vec_mean(&returns).unwrap() / vec_std(&returns).unwrap();
            let ann_factor = (252_f64).sqrt();
            sharpes.insert((interval, *start_time, end_time), sharpe*ann_factor);

        }
    }

    // write_csv(sharpes, &["Interval", "Start Time", "Sharpe"]);

    match write_csv(sharpes, &["Interval", "Start Time", "End Time", "Sharpe"]) {
        Err(e) => println!("Write CSV error {}", e),
        _ => ()
    }

    Ok(())
}


fn write_csv(h: FxHashMap<(u64, NaiveTime, NaiveTime), f64>, column_names: &[&str]) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path("returns_test.csv")?;
    wtr.write_record(column_names)?;
    for ((i,st, e), r) in h {
        wtr.write_record(&[i.to_string(), st.to_string(), e.to_string(), r.to_string()])?;
    }
    wtr.flush()?;
    Ok(())
}
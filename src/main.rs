use std::thread;
use std::sync::{Arc, Mutex, MutexGuard};
use rustc_hash::FxHashMap;
use std::error::Error;
use chrono::{Datelike, NaiveDateTime, NaiveTime};
use backtesting::utils::*;
use backtesting::vector_utils::{vec_mean, vec_std, vec_unique, vec_where_eq};
use std::time::Instant;

fn main() -> Result<(), Box<dyn Error>>  {
    // let file_name = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\ZN_continuous_adjusted_1min_tail.csv";
    let file_name = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\ZN_continuous_adjusted_1min.csv";
    println!("Using file: {}", file_name);

    // Read CSV
    let v: Vec<Row> = read_csv(file_name).unwrap()
        .into_iter()
        .filter(|x| x.datetime().year() >= 2021)
        .collect();
    println!("{} rows", v.len());


    // Initialize Params
    let resolution: u64 = 1; // minutes
    let interval_rng: Vec<u64> = (2..60).map(|x| x*resolution).collect();
    let start_time_rng: Vec<NaiveTime> = time_range((8,0,0), (10,30,0), resolution);

    let total_runs: u64 = (interval_rng.len()*start_time_rng.len()) as u64;
    println!("Running {} times", total_runs);


    let now = Instant::now();
    let n_threads = 8;
    let interval_rng:Vec<Vec<u64>> = interval_rng.chunks(interval_rng.len()/n_threads + 1).map(|x| x.to_vec()).collect();

    let counter = Arc::new(Mutex::new(0_u64));
    let mut handles = vec![];
    for i in 0..n_threads {
        let times: Vec<NaiveDateTime> = v.iter().map(|x| x.datetime()).collect();
        let values: Vec<f64> = v.iter().map(|x| x.close).collect();
        let st_rng: Vec<NaiveTime> = start_time_rng.clone();
        let i_rng: Vec<u64> = interval_rng[i].clone();
        let counter = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            run_analysis(times, values, &i_rng, &st_rng,
                         counter, total_runs).unwrap()
        });
        handles.push(handle);
    }

    let mut results:FxHashMap<(u64, NaiveTime, NaiveTime), f64> = FxHashMap::default();
    for handle in handles {
        let h = handle.join().unwrap();
        for (k, v) in h {
            results.insert(k, v);
        }
    }

    println!("{} seconds to run", now.elapsed().as_secs());

    println!("Writing to csv");
    match write_csv(&results, &["Interval", "Start Time", "End Time", "Sharpe"]) {
        Err(e) => println!("Write CSV error {}", e),
        _ => println!("CSV export complete"),

    }

    Ok(())

}



fn run_analysis(times: Vec<NaiveDateTime>, values: Vec<f64>,
                interval_rng: &Vec<u64>, start_time_rng: &Vec<NaiveTime>, progress_counter: Arc<Mutex<u64>>, total_runs: u64)
                -> Result<FxHashMap<(u64, NaiveTime, NaiveTime), f64>, Box<dyn Error>> {

    let mut sharpes: FxHashMap<(u64, NaiveTime, NaiveTime), f64> = FxHashMap::default();
    // let mut progress_counter: usize = 0;
    let now = Instant::now();
    for interval in interval_rng {
        for start_time in start_time_rng {
            {
                let mut p = progress_counter.lock().unwrap();
                *p += 1;
                if *p % 100 == 0 {
                    println!("Running iteration {} out of {}, {} seconds elapsed",
                             *p, total_runs, now.elapsed().as_secs());
                }
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
            // println!("{:?}", returns);
            let sharpe = vec_mean(&returns).unwrap() / vec_std(&returns).unwrap();
            // println!("{}", sharpe);
            let ann_factor = (252_f64).sqrt();
            sharpes.insert((interval.clone(), start_time.clone(), end_time), sharpe*ann_factor);

        }
    }


    // match write_csv(&sharpes, &["Interval", "Start Time", "End Time", "Sharpe"]) {
    //     Err(e) => println!("Write CSV error {}", e),
    //     _ => ()
    // }

    Ok(sharpes)

}


use std::env;
use std::thread;
use std::sync::{Arc, Mutex};
use std::error::Error;
use chrono::{NaiveDateTime, NaiveTime};
use backtesting::strategy::StrategyResult;
use backtesting::utils::*;
use backtesting::events::*;
use std::time::Instant;
use rustc_hash::FxHashMap;
use backtesting::strategy;


fn main() -> Result<(), Box<dyn Error>>  {

    // Initialize Params
    let resolution: u64 = 10; // minutes
    let interval_rng: Vec<u64> = (2..60*8).filter(|x| x % resolution == 0).collect();
    let start_time_rng: Vec<NaiveTime> = time_range((6,0,0), (16,55,0), resolution);

    let total_runs: u64 = (interval_rng.len()*start_time_rng.len()) as u64;
    println!("Running {} times", total_runs);

    // Read get events data
    let events_loc = "C:\\Users\\mbroo\\PycharmProjects\\backtesting\\calendar-event-list-new.csv";
    let event_data: FxHashMap<String, Vec<NaiveDateTime>> = get_event_calendar(events_loc);
    let mut events: FxHashMap<&str, Vec<NaiveDateTime>> = FxHashMap::default();
    for e in ["Inflation Rate YoY", "Non Farm Payrolls"] {
        events.insert(e, event_data.get(e).unwrap().to_owned());
    }

    // let cpi = event_data.get("Inflation Rate YoY").unwrap();
    // let nfp = event_data.get("Non Farm Payrolls").unwrap();
    // let mut events: FxHashMap<&str, Vec<NaiveDateTime>> = FxHashMap::default();
    // events.insert("CPI", cpi.to_owned());
    // events.insert("NFP", fomc.to_owned());

    // Read timeseries CSV
    // let file_name = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\ZN_continuous_adjusted_1min_tail.csv";
    let file_name = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\ZN_continuous_adjusted_1min.csv";
    // println!("Using file: {}", file_name);
    let mut v: Vec<Row> = read_csv(file_name).unwrap()
        .into_iter()
        .filter(|x: &Row| x.datetime() >= NaiveDateTime::parse_from_str("2019-01-01 00:00:01", "%Y-%m-%d %H:%M:%S").unwrap())
        .filter(|x| (x.datetime().time() >= start_time_rng[0]))
        .collect();

    // let event_filter_dts = events.values().flatten().collect::<Vec<_>>().iter().map(|x| x.date()).collect();
    v = filter_timeseries_by_events(v,
                                    // events.get("Inflation Rate YoY").unwrap().iter().map(|x| x.date()).collect(),
                                    &events.values().flatten().collect::<Vec<_>>().iter().map(|x| x.date()).collect(),
                                    10, 1);
    println!("{} rows after filters", v.len());

    println!("Starting at {}", chrono::Local::now());
    let is_singlethreaded: bool = match env::var("IS_SINGLETHREADED") {
        Ok(x) => {if x=="TRUE" { true } else { false }},
        Err(e) => { println!("{}", e); false },
    };

    let now = Instant::now();
    let mut results: Vec<StrategyResult> = Vec::new();
    if !is_singlethreaded {
        let n_threads = 10;
        println!("Running multi({})-threaded", n_threads);
        let mut interval_rng_: Vec<Vec<u64>> = (0..n_threads).map(|_| Vec::new() ).collect();
        for &i in interval_rng.iter() {
            interval_rng_[(i % n_threads as u64) as usize].push(i)
        }

        let counter = Arc::new(Mutex::new(0_u64));
        let mut handles = vec![];

        for i in 0..interval_rng_.len() {
        // for i in [0] {
            let datetimes: Vec<NaiveDateTime> = v.iter().map(|x| x.datetime()).collect();
            let values: Vec<f64> = v.iter().map(|x| x.close).collect();
            let start_time_rng_: Vec<NaiveTime> = start_time_rng.clone();
            let interval_rng_i_: Vec<u64> = interval_rng_[i].clone();
            let events_ = events.clone();
            let counter = Arc::clone(&counter);

            // let handle = thread::spawn(move || {
            let handle = thread::Builder::new().name(i.to_string()).spawn(move || {
                strategy::run_analysis(datetimes, values, &interval_rng_i_, &start_time_rng_, &events_,
                                       counter, total_runs, i).unwrap()
            });
            handles.push(handle.unwrap());
        }

        results = handles.into_iter().map(|h| h.join().unwrap()).flatten().collect();
    }
    if is_singlethreaded {
        // Single-threaded for profiling
        println!("Running single-threaded");
        let times: Vec<NaiveDateTime> = v.iter().map(|x| x.datetime()).collect();
        let values: Vec<f64> = v.iter().map(|x| x.close).collect();

        results = strategy::run_analysis(times, values, &interval_rng, &start_time_rng, &events,
                                         Arc::new(Mutex::new(0)), total_runs, 1_usize).unwrap();
    }
    println!("{} seconds to run,", now.elapsed().as_secs());
    println!("for a total of {} rows", results.len());

    match write_csv(&results) {
        Err(e) => println!("Write CSV error: {}", e),
        _ => (),
    }

    std::process::Command::new("cmd")
        .args(&["/C", "C:/Users/mbroo/anaconda3/envs/backtesting/python.exe C:/Users/mbroo/PycharmProjects/backtesting/from_rust.py"])
        .output()
        .expect("failed to execute process");

    Ok(())
}


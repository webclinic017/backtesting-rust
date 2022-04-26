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
    // let file_name = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\ZN_continuous_adjusted_1min_tail.csv";
    let file_name = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\ZN_continuous_adjusted_1min.csv";
    println!("Using file: {}", file_name);

    // Initialize Params
    let resolution: u64 = 1; // minutes
    let interval_rng: Vec<u64> = (2..60).map(|x| x*resolution).collect();
    let start_time_rng: Vec<NaiveTime> = time_range((8,0,0), (10,30,0), resolution);

    let total_runs: u64 = (interval_rng.len()*start_time_rng.len()) as u64;
    println!("Running {} times", total_runs);

    // Read timeseries CSV
    let v: Vec<Row> = read_csv(file_name).unwrap()
        .into_iter()
        .filter(|x: &Row| x.datetime() >= NaiveDateTime::parse_from_str("2020-01-01 00:00:01", "%Y-%m-%d %H:%M:%S").unwrap())
        .filter(|x| (x.datetime().time() >= start_time_rng[0]) &
            (x.datetime().time() <= add_time(&start_time_rng[start_time_rng.len() - 1], interval_rng[interval_rng.len()-1]*60)))
        .collect();
    println!("{} rows", v.len());

    // Read get events data
    let events_loc = "C:\\Users\\mbroo\\PycharmProjects\\backtesting\\calendar-event-list.csv";
    let event_data: FxHashMap<String, Vec<NaiveDateTime>> = get_event_calendar(events_loc, &[3]);
    let cpi = event_data.get("Consumer Price Index ex Food & Energy (YoY)").unwrap();
    let fomc = event_data.get("FOMC Press Conference").unwrap();
    let mut events: FxHashMap<&str, Vec<NaiveDateTime>> = FxHashMap::default();
    events.insert("CPI", cpi.to_owned());
    events.insert("FOMC", fomc.to_owned());

    println!("Starting at UTC {}", chrono::Local::now());

    static IS_SINGLETHREAD: bool = false;

    let now = Instant::now();
    let mut results: Vec<StrategyResult> = Vec::new();
    if !IS_SINGLETHREAD {
        let n_threads = 8;
        let n_chunks = if interval_rng.len() % n_threads > 0 {
            interval_rng.len() / n_threads + 1
        } else {
            interval_rng.len() / n_threads
        };
        let interval_rng: Vec<Vec<u64>> = interval_rng.chunks(n_chunks).map(|x| x.to_vec()).collect();

        let counter = Arc::new(Mutex::new(0_u64));
        let mut handles = vec![];
        for i in 0..interval_rng.len() {
            let datetimes: Vec<NaiveDateTime> = v.iter().map(|x| x.datetime()).collect();
            let values: Vec<f64> = v.iter().map(|x| x.close).collect();
            let start_time_rng_: Vec<NaiveTime> = start_time_rng.clone();
            let interval_rng_i_: Vec<u64> = interval_rng[i].clone();
            let events_ = events.clone();
            let counter = Arc::clone(&counter);

            let handle = thread::spawn(move || {
                strategy::run_analysis(datetimes, values, &interval_rng_i_, &start_time_rng_, &events_,
                                       counter, total_runs, i).unwrap_or(vec![])
            });
            handles.push(handle);
        }

        results = handles.into_iter().map(|h| h.join().unwrap()).flatten().collect();
    }
    if IS_SINGLETHREAD {
        // Single-threaded for profiling
        let times: Vec<NaiveDateTime> = v.iter().map(|x| x.datetime()).collect();
        let values: Vec<f64> = v.iter().map(|x| x.close).collect();

        results = strategy::run_analysis(times, values, &interval_rng, &start_time_rng, &events,
                                         Arc::new(Mutex::new(0)), total_runs, 1_usize).unwrap();
    }
    println!("{} seconds to run", now.elapsed().as_secs());

    println!("Writing to csv");

    match write_csv(&results) {
        Err(e) => println!("Write CSV error: {}", e),
        _ => println!("CSV export complete"),
    }

    std::process::Command::new("cmd")
        .args(&["/C", "C:/Users/mbroo/anaconda3/envs/backtesting/python.exe C:/Users/mbroo/PycharmProjects/backtesting/from_rust.py"])
        .output()
        .expect("failed to execute process");

    Ok(())

}


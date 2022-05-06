use std::{env, fs};
use std::thread;
use std::sync::{Arc, Mutex};
use std::error::Error;
use log::{error, info, warn};
use log4rs;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use backtesting::strategy::StrategyResult;
use backtesting::utils::*;
use backtesting::events::*;
use backtesting::analysis::run_analysis;
use std::time::Instant;
use rustc_hash::FxHashMap;
use backtesting::strategy::*;


fn main() -> Result<(), Box<dyn Error>> {
    // Set up logging
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    let file_name = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\ZN_continuous_adjusted_1min.csv";
    let data: Vec<Row> = read_csv(file_name).unwrap();

    let events_loc = "C:\\Users\\mbroo\\PycharmProjects\\backtesting\\calendar-event-list-new.csv";
    let event_data: FxHashMap<String, Vec<NaiveDateTime>> = get_event_calendar(events_loc);
    // let event_names: Vec<String> = event_data.keys().into_iter().collect();

    let output_path = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\output\\full";
    for (event_name, events) in event_data {
        println!("Running event: {}", event_name);
        let now = Instant::now();
        match main_routine(&data, event_name.as_str(), &events, output_path) {
            Ok(()) => println!("Ran {} in {}s", event_name, now.elapsed().as_secs()),
            Err(e) => { error!("{} {}", event_name, e); continue }
        }
    }
    Ok(())
}

fn main_routine(data: &Vec<Row>, event_name: &str, events: &Vec<NaiveDateTime>, output_path: &str) -> Result<(), Box<dyn Error>> {

    // Initialize Params
    let resolution: u64 = 1; // minutes
    let interval_rng: Vec<u64> = (2..=60*12).filter(|x| x % resolution == 0).collect();
    let start_time_rng: Vec<NaiveTime> = time_range((6,0,0), (16,55,0), resolution);
    info!("Inveral params (mins): {} to {}, by step {}", interval_rng[0], interval_rng[interval_rng.len()-1], resolution);
    info!("Start time params: {} to {}, with resolution {}", start_time_rng[0], start_time_rng[start_time_rng.len()-1], resolution);

    let total_runs: u64 = (interval_rng.len()*start_time_rng.len()) as u64;
    info!("Running {} times", total_runs);


    // Read cluster data
    // let cluster_loc = "C:\\Users\\mbroo\\PycharmProjects\\detrending\\cluster_data.csv";
    // let cluster = 4;
    // let clusters: FxHashMap<i32, i32> = get_clusters(cluster_loc);//.into_iter().filter(|(k,v)| v==&cluster).collect();
    // let cluster_dates: Vec<i32> = clusters.iter()
    //     .filter(|(_,&v)| v==cluster)
    //     .map(|(&k, _)| k )
    //     .collect();

    let mut v: Vec<&Row> = data
        .iter()
        // .filter(|x: &Row| x.datetime() >= NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap())
        .filter(|&x| x.datetime().time() >= start_time_rng[0])
        // .filter(|&x| cluster_dates.contains(&datestr_to_int(x.datetime_str.as_str())))
        .collect();

    let event_dates: Vec<NaiveDate> = events.iter().map(|dt| dt.date()).collect();
    v = filter_timeseries_by_events(v,
                                    &event_dates,
                                    1, 1);
    info!("{} rows after filters", v.len());

    let datetimes: Vec<NaiveDateTime> = v.iter().map(|x| x.datetime()).collect();
    let values: Vec<f64> = v.iter().map(|x| x.close).collect();
    // let event_name = "Inflation Rate YoY";

    let mut context_conditions: Vec<Vec<bool>> = Vec::new();
    context_conditions.push(day_of_strat(&datetimes, &vec_dates(events)));
    // context_conditions.push(days_offset_strat(&datetimes, event_dates.get("Non Farm Payrolls").unwrap(),
    //                                           -8, -1, true));

    info!("Starting analysis");
    let is_singlethreaded: bool = match env::var("IS_SINGLETHREADED") {
        Ok(x) => {if x=="TRUE" { true } else { false }},
        Err(e) => { error!("{}", e); false },
    };

    let now = Instant::now();
    let mut results: Vec<StrategyResult> = Vec::new();
    if !is_singlethreaded {
        let n_threads = 8;
        info!("Running multi({})-threaded", n_threads);
        let mut interval_rng_: Vec<Vec<u64>> = (0..n_threads).map(|_| Vec::new() ).collect();
        for &i in interval_rng.iter() {
            interval_rng_[(i % n_threads as u64) as usize].push(i)
        }

        let counter = Arc::new(Mutex::new(0_u64));
        let mut handles = vec![];

        for i in 0..interval_rng_.len() {
            let datetimes_ = datetimes.clone();
            let values_ = values.clone();
            let start_time_rng_: Vec<NaiveTime> = start_time_rng.clone();
            let interval_rng_i_: Vec<u64> = interval_rng_[i].clone();
            let context_conditions_ = context_conditions.clone();
            let counter = Arc::clone(&counter);

            let handle = thread::Builder::new().name(i.to_string()).spawn(move || {
                run_analysis(&datetimes_, &values_, &interval_rng_i_, &start_time_rng_,
                                       counter, total_runs, &context_conditions_).unwrap_or_default()
            });
            handles.push(handle.unwrap());
        }
        results = handles.into_iter().map(|h| h.join().unwrap()).flatten().collect();
    }
    if is_singlethreaded {
        // Single-threaded for profiling
        info!("Running single-threaded");
        results = run_analysis(&datetimes, &values, &interval_rng, &start_time_rng,
                               Arc::new(Mutex::new(0)), total_runs, &context_conditions).unwrap();
    }
    info!("{} seconds to run,", now.elapsed().as_secs());
    info!("for a total of {} rows", results.len());

    fs::create_dir(output_path);
    match write_csv(&results, &FIELD_NAMES, format!("{}/{}_returns.csv", output_path, event_name.replace(" ", "_")).as_str()) {
        Err(e) => {error!("Write CSV error: {}", e); return Err(e) },
        _ => (),
    }

    // std::process::Command::new("cmd")
    //     .args(&["/C", "C:/Users/mbroo/anaconda3/envs/backtesting/python.exe C:/Users/mbroo/PycharmProjects/backtesting/from_rust.py"])
    //     .output()
    //     .expect("failed to execute process");

    Ok(())
}


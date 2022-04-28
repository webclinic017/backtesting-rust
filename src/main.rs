use std::env;
use std::thread;
use std::sync::{Arc, Mutex};
use std::error::Error;
use chrono::{NaiveDateTime, NaiveTime};
use backtesting::strategy::StrategyResult;
use backtesting::utils::*;
use backtesting::events::*;
use backtesting::analysis::run_analysis;
use std::time::Instant;
use rustc_hash::FxHashMap;
use backtesting::strategy::*;


fn main() -> Result<(), Box<dyn Error>>  {

    // Initialize Params
    let resolution: u64 = 1; // minutes
    let interval_rng: Vec<u64> = (2..60*8).filter(|x| x % resolution == 0).collect();
    let start_time_rng: Vec<NaiveTime> = time_range((8,0,0), (10,30,0), resolution);

    let total_runs: u64 = (interval_rng.len()*start_time_rng.len()) as u64;
    println!("Running {} times", total_runs);

    // Read get events data
    let events_loc = "C:\\Users\\mbroo\\PycharmProjects\\backtesting\\calendar-event-list-new.csv";
    let event_data: FxHashMap<String, Vec<NaiveDateTime>> = get_event_calendar(events_loc);
    let mut events: FxHashMap<&str, Vec<NaiveDateTime>> = FxHashMap::default();
    // let mut event_dates:FxHashMap<&str, Vec<NaiveDate>> = FxHashMap::default();
    for e in ["Inflation Rate YoY", "Non Farm Payrolls"] {
        events.insert(e, event_data.get(e).unwrap().to_owned());
        // event_dates.insert(e, vec_dates(event_data.get(e).unwrap()));
    }

    // Read timeseries CSV
    // let file_name = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\ZN_continuous_adjusted_1min_tail.csv";
    let file_name = "C:\\Users\\mbroo\\IdeaProjects\\backtesting\\ZN_continuous_adjusted_1min.csv";
    let mut v: Vec<Row> = read_csv(file_name).unwrap()
        .into_iter()
        .filter(|x: &Row| x.datetime() >= NaiveDateTime::parse_from_str("2021-01-01 00:00:00", "%Y-%m-%d %H:%M:%S").unwrap())
        .filter(|x| (x.datetime().time() >= start_time_rng[0]))
        .collect();

    v = filter_timeseries_by_events(v,
                                    &events.values().flatten().collect::<Vec<_>>().iter().map(|x| x.date()).collect(),
                                    10, 1);
    println!("{} rows after filters", v.len());

    let datetimes: Vec<NaiveDateTime> = v.iter().map(|x| x.datetime()).collect();
    let values: Vec<f64> = v.iter().map(|x| x.close).collect();

    let mut context_conditions: Vec<Vec<bool>> = Vec::new();
    context_conditions.push(day_of_strat(&datetimes, &vec_dates(events.get("Inflation Rate YoY").unwrap())));
    // context_conditions.push(days_offset_strat(&datetimes, event_dates.get("Non Farm Payrolls").unwrap(),
    //                                           -8, -1, true));

    println!("Starting at {}", chrono::Local::now());
    let is_singlethreaded: bool = match env::var("IS_SINGLETHREADED") {
        Ok(x) => {if x=="TRUE" { true } else { false }},
        Err(e) => { println!("{}", e); false },
    };

    let now = Instant::now();
    let mut results: Vec<StrategyResult> = Vec::new();
    if !is_singlethreaded {
        let n_threads = 8;
        println!("Running multi({})-threaded", n_threads);
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
                run_analysis(datetimes_, values_, &interval_rng_i_, &start_time_rng_,
                                       counter, total_runs, &context_conditions_).unwrap()
            });
            handles.push(handle.unwrap());
        }
        results = handles.into_iter().map(|h| h.join().unwrap()).flatten().collect();
    }
    if is_singlethreaded {
        // Single-threaded for profiling
        println!("Running single-threaded");
        results = run_analysis(datetimes, values, &interval_rng, &start_time_rng,
                               Arc::new(Mutex::new(0)), total_runs, &context_conditions).unwrap();
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


use std::thread;
use std::sync::{Arc, Mutex};
use std::error::Error;
use chrono::{NaiveDateTime, NaiveTime};
use backtesting::strategy::Strategy;
use backtesting::utils::*;
use backtesting::vector_utils::*;
use std::time::Instant;


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

    // Read CSV
    let v: Vec<Row> = read_csv(file_name).unwrap()
        .into_iter()
        .filter(|x| x.datetime() >= NaiveDateTime::parse_from_str("2020-01-01 00:00:01", "%Y-%m-%d %H:%M:%S").unwrap())
        .filter(|x| (x.datetime().time() >= start_time_rng[0]) &
            (x.datetime().time() <= add_time(&start_time_rng[start_time_rng.len() - 1], interval_rng[interval_rng.len()-1]*60)))
        .collect();
    println!("{} rows", v.len());

    let now = Instant::now();
    let n_threads = 8;
    let n_chunks = if interval_rng.len() % n_threads > 0 {
        interval_rng.len() / n_threads + 1
    }
                                                          else {
        interval_rng.len() / n_threads
    };
    let interval_rng:Vec<Vec<u64>> = interval_rng.chunks(n_chunks).map(|x| x.to_vec()).collect();

    let counter = Arc::new(Mutex::new(0_u64));
    let mut handles = vec![];
    for i in 0..interval_rng.len() {
        let times: Vec<NaiveDateTime> = v.iter().map(|x| x.datetime()).collect();
        let values: Vec<f64> = v.iter().map(|x| x.close).collect();
        let st_rng: Vec<NaiveTime> = start_time_rng.clone();
        let i_rng: Vec<u64> = interval_rng[i].clone();
        let counter = Arc::clone(&counter);

        let handle = thread::spawn(move || {
            run_analysis(times, values, &i_rng, &st_rng,
                         counter, total_runs, i).unwrap()
        });
        handles.push(handle);
    }

    let results = handles.into_iter().map(|h| h.join().unwrap()).flatten().collect();

    println!("{} seconds to run", now.elapsed().as_secs());

    println!("Writing to csv");
    match write_csv(&results) {
        Err(e) => println!("Write CSV error {}", e),
        _ => println!("CSV export complete"),
    }

    Ok(())

}


fn run_analysis(times: Vec<NaiveDateTime>, values: Vec<f64>,
                interval_rng: &Vec<u64>, start_time_rng: &Vec<NaiveTime>,
                progress_counter: Arc<Mutex<u64>>, total_runs: u64, i_thread: usize)
                -> Result<Vec<Strategy>, Box<dyn Error>> {

    // let mut ret: FxHashMap<(u64, NaiveTime, NaiveTime), (f64, f64, f64, usize)> = FxHashMap::default();
    let mut ret: Vec<Strategy> = Vec::new();
    let now = Instant::now();
    for interval in interval_rng {
        for start_time in start_time_rng {
            {
                let mut p = progress_counter.lock().unwrap();
                *p += 1;
                if *p % 100 == 0 {
                    println!("Running iteration {} out of {} on thread {}, {} seconds elapsed",
                             *p, total_runs, i_thread, now.elapsed().as_secs());
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
            let mut drawups: Vec<f64> = Vec::new();
            let mut drawdowns: Vec<f64> = Vec::new();
            let mut n_obs= 0_usize;
            for i in vec_unique(&r).into_iter() {
                if i == &0 {continue;} // 0 means no observation
                let ix = vec_where_eq(&r, i);
                let _t = &times[ix[0]..=ix[ix.len()-1]];
                let v:Vec<f64> = values[ix[0]..=ix[ix.len()-1]].to_vec();

                if v.len() > 2 {
                    n_obs += 1;
                    // let d:Vec<f64> = (0..(v.len()-1)).map(|i| &v[i+1] - &v[i]).collect();
                    let mut d = vec_diff(&v, 1).unwrap();
                    d = vec_cumsum(&d).unwrap();
                    d.sort_by(|a, b| comp_f64(a,b));
                    returns.push(v[v.len()-1] - v[0]);
                    drawups.push(d[0]);
                    drawdowns.push(d[d.len() - 1]);
                }
            }
            // println!("{:?}", returns);
            let sharpe = vec_mean(&returns).unwrap() / vec_std(&returns).unwrap();
            let ann_factor = (252_f64).sqrt();

            drawups.sort_by(|a, b| comp_f64(a,b));
            drawdowns.sort_by(|a, b| comp_f64(a,b));

            ret.push(Strategy {
                interval: Some(*interval),
                start_time: Some(*start_time),
                end_time: Some(end_time),
                sharpe: Some(sharpe*ann_factor),
                max_drawup: Some(drawups[0]),
                max_drawdown: Some(drawdowns[drawdowns.len() - 1]),
                n_obs: Some(n_obs),
            });
        }
    }
    Ok(ret)
}


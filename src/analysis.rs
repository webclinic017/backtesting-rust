use log::{error, info, warn};
use std::thread;
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::time::Instant;
use chrono::{NaiveTime, NaiveDateTime};
use simple_error::SimpleError;
pub use crate::strategy::*;
pub use crate::utils::*;
pub use crate::vector_utils::*;


pub fn run_analysis(datetimes: &Vec<NaiveDateTime>, values: &Vec<f64>,
                    interval_rng: &Vec<u64>, start_time_rng: &Vec<NaiveTime>,
                    progress_counter: Arc<Mutex<u64>>, total_runs: u64, context_conditions: &Vec<Vec<bool>>)
                    -> Result<Vec<StrategyResult>, Box<dyn Error>> {
    let thread_name = match thread::current().name() {
        Some(x) => String::from(x),
        None => String::from("no thread???")
    };

    let mut context_condition = vec![true; values.len()];
    if !context_conditions.is_empty() {
        for c in context_conditions {
            context_condition = context_condition.iter().zip(c.iter()).map(|(x, y)| x & y).collect();
        }
    }

    let mut ret: Vec<StrategyResult> = Vec::new();
    let now = Instant::now();
    for interval in interval_rng {
        for start_time in start_time_rng {
            {
                let mut p = progress_counter.lock().unwrap();
                *p += 1;
                if *p % 500 == 0 {
                    let elapsed = now.elapsed().as_secs_f32();
                    let pct = (*p as f32)/(total_runs as f32);
                    info!("Iteration {} ({:.1}%) out of {} on thread {}, {:.1}s elapsed  (total {:.0}s expected)",
                             *p, pct*100., total_runs, thread_name, elapsed, (elapsed as f32)/pct);
                }
            }
            let end_time = add_time(start_time, interval*60);
            if end_time >= NaiveTime::from_hms(17,0,0) { continue; } // End of day for futures

            // Set entry condition
            let mut entry_cond: Vec<bool> = datetimes.iter().map(|x| x.time()==*start_time).collect(); // absolute time strat

            // Set exit condition
            let mut exit_cond: Vec<bool> = datetimes.iter().map(|x| x.time()==end_time).collect();

            entry_cond = entry_cond.iter().zip(&context_condition).map(|(x,y)| x&y).collect();
            exit_cond = exit_cond.iter().zip(&context_condition).map(|(x,y)| x&y).collect();

            // Set r as total condition vector
            let r:Vec<i32> = entry_cond.iter().zip(exit_cond.iter())
                .map(|(&x, &y)| (x as i32) - (y as i32))
                .collect();

            let r: Vec<usize> = fill_ids(&r);

            let mut returns: Vec<f64> = Vec::new();
            let mut drawups: Vec<f64> = Vec::new();
            let mut drawdowns: Vec<f64> = Vec::new();
            // let mut datetime_data: Vec<Vec<NaiveDateTime>> = Vec::new();
            // let mut value_data: Vec<Vec<f64>> = Vec::new();
            let mut n_obs= 0_usize;
            for i in vec_unique(&r).into_iter()
            {
                if i == &0 { continue; } // 0 means no observation
                let ix = vec_where_eq(&r, i);
                // let t = datetimes[ix[0]..=ix[ix.len()-1]].to_vec();
                let v:Vec<f64> = values[ix[0]..=ix[ix.len()-1]].to_vec();

                if v.len() > 2
                {
                    n_obs += 1;
                    let mut d = vec_diff(&v, 1).unwrap();
                    d = vec_cumsum(&d).unwrap();
                    d.sort_by(|a, b| comp_f64(a,b));
                    returns.push(v[v.len()-1] - v[0]);
                    drawups.push(d[0]);
                    drawdowns.push(d[d.len() - 1]);
                    // datetime_data.push(t);
                    // value_data.push(v);
                }
            }
            let sharpe = vec_mean(&returns).unwrap_or(f64::NAN) / vec_std(&returns).unwrap_or(f64::NAN);
            if !sharpe.is_normal() { continue }

            let ann_factor = (252_f64).sqrt();
            drawups.sort_by(|a, b| comp_f64(a,b));
            drawdowns.sort_by(|a, b| comp_f64(a,b));

            let max_drawup = match drawups.get(1)
            {
                Some(x) => x,
                None => continue
            };
            let max_drawdown = match drawdowns.get(drawdowns.len() - 1)
            {
                Some(x) => x,
                None => continue
            };

            ret.push(StrategyResult
                {
                    interval: *interval,
                    start_time: *start_time,
                    end_time: end_time,
                    sharpe: sharpe*ann_factor,
                    max_drawup: *max_drawup,
                    max_drawdown: *max_drawdown,
                    n_obs,
                    // datetime_data,
                    // value_data,
                }
            );
        }
    }

    match ret.len()
    {
        0 => {
            let msg = format!("Returns on thread {} length was zero", thread_name);
            error!("{}", msg);
            Err(Box::new(SimpleError::new(msg)))
        },
        _ => Ok(ret)
    }
}


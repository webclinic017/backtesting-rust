use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use rustc_hash::FxHashMap;
use std::sync::{Arc, Mutex};
use std::error::Error;
use std::time::Instant;
use bdays::HolidayCalendar;
use simple_error::SimpleError;
use crate::utils::{add_time, comp_f64, fill_ids, vec_cumsum, vec_diff, vec_mean, vec_std, vec_unique, vec_where_eq};

pub const N_FIELDS: usize = 7;
pub static FIELD_NAMES: [&str; N_FIELDS] = ["interval", "start time", "end time", "sharpe", "max drawup", "max drawdown", "n obs"];
pub struct StrategyResult {
    pub interval: u64,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub sharpe: f64,
    pub max_drawup: f64,
    pub max_drawdown: f64,
    pub n_obs: usize,
}
impl StrategyResult {
    pub fn fields_to_strings(&self) -> [String; N_FIELDS] {
        [self.interval.to_string(), self.start_time.to_string(), self.end_time.to_string(),
            self.sharpe.to_string(), self.max_drawup.to_string(), self.max_drawdown.to_string(),
            self.n_obs.to_string()]
    }
}
impl Default for StrategyResult {
    fn default() -> Self {
        Self {
            interval: 0,
            start_time: NaiveTime::from_hms(1,0,0),
            end_time: NaiveTime::from_hms(1,0,0),
            sharpe: f64::NAN,
            max_drawup: f64::NAN,
            max_drawdown: f64::NAN,
            n_obs: 0,
        }
    }
}

pub fn day_of_strat(datetimes: &Vec<NaiveDateTime>, event_dates: &Vec<NaiveDate>) -> Vec<bool> {
    datetimes.iter()
        .map(|x| event_dates.contains(&x.date()))
        .collect()
}

static BUS_DAY_CAL:bdays::calendars::us::USSettlement = bdays::calendars::us::USSettlement;
pub fn days_offset_strat(datetimes: &Vec<NaiveDateTime>, event_dates: &Vec<NaiveDate>,
                         early_offset_days: i64, late_offset_days: i64, is_bus_days: bool) -> Vec<bool> {
    assert!(early_offset_days<=late_offset_days);
    let mut event_date_ranges = event_dates.clone();
    for &dt in event_dates {
        for i in early_offset_days..=late_offset_days {
            if is_bus_days {
                event_date_ranges.push(BUS_DAY_CAL.advance_bdays(dt, i as i32))
            }
            else {
                event_date_ranges.push(dt + Duration::days(i));
            }
        }
    }
    datetimes.iter().map(|x| event_date_ranges.contains(&x.date())).collect()
}


pub fn run_analysis(datetimes: Vec<NaiveDateTime>, values: Vec<f64>,
                    interval_rng: &Vec<u64>, start_time_rng: &Vec<NaiveTime>, events: &FxHashMap<&str, Vec<NaiveDateTime>>,
                    progress_counter: Arc<Mutex<u64>>, total_runs: u64, i_thread: usize)
                    -> Result<Vec<StrategyResult>, Box<dyn Error>> {

    let mut event_dates:FxHashMap<&str, Vec<NaiveDate>> = FxHashMap::default();
    for (&k, v) in events {
        event_dates.insert(k, v.iter().map(|x| x.date()).collect());
    }

    let mut ret: Vec<StrategyResult> = Vec::new();
    let now = Instant::now();
    for interval in interval_rng {
        for start_time in start_time_rng {
            {
                let mut p = progress_counter.lock().unwrap();
                *p += 1;
                if *p % 100 == 0 {
                    let elapsed = now.elapsed().as_secs();
                    let pct = (*p as f32)/(total_runs as f32);
                    println!("Running iteration {} ({:.1}%) out of {} on thread {}, {} seconds elapsed  (total {:.0}s expected)",
                             *p, pct*100., total_runs, i_thread, elapsed, (elapsed as f32)/pct);
                }
            }
            let end_time = add_time(start_time, interval*60);
            if end_time >= NaiveTime::from_hms(17,0,0) { continue; } // End of day for futures

            // General (context) conditions (save overhead)
            let mut gen_cond: Vec<Vec<bool>> = Vec::new();
            gen_cond.push(day_of_strat(&datetimes, event_dates.get("CPI").unwrap()));
            // gen_cond.push(days_offset_strat(&datetimes, event_dates.get("FOMC").unwrap(),
            //                                 -10, -1, true));
            if gen_cond.is_empty() { gen_cond.push(vec![true; values.len()]) }

            // Set entry conditions
            let entry_cond1: Vec<bool> = datetimes.iter().map(|x| x.time()==*start_time).collect(); // absolute time strat
            let mut entry_cond:Vec<bool> = vec![true; values.len()];
            for c in vec![entry_cond1].iter().chain(gen_cond.iter()) {
                entry_cond = entry_cond.iter().zip(c).map(|(x,y)| x&y).collect();
            }

            // Set exit conditions
            let exit_cond1: Vec<bool> = datetimes.iter().map(|x| x.time()==end_time).collect();
            let mut exit_cond:Vec<bool> = vec![true; values.len()];
            for c in vec![exit_cond1].iter().chain(gen_cond.iter()) {
                exit_cond = exit_cond.iter().zip(c).map(|(x,y)| x&y).collect();
            }

            // Set r as total condition vector
            let r:Vec<i32> = entry_cond.iter().zip(exit_cond.iter())
                .map(|(&x, &y)| (x as i32) - (y as i32))
                .collect();

            let r: Vec<usize> = fill_ids(&r);

            let mut returns: Vec<f64> = Vec::new();
            let mut drawups: Vec<f64> = Vec::new();
            let mut drawdowns: Vec<f64> = Vec::new();
            let mut n_obs= 0_usize;
            for i in vec_unique(&r).into_iter() {
                if i == &0 { continue; } // 0 means no observation
                let ix = vec_where_eq(&r, i);
                let _t = &datetimes[ix[0]..=ix[ix.len()-1]];
                let v:Vec<f64> = values[ix[0]..=ix[ix.len()-1]].to_vec();

                if v.len() > 2 {
                    n_obs += 1;
                    let mut d = vec_diff(&v, 1).unwrap();
                    d = vec_cumsum(&d).unwrap();
                    d.sort_by(|a, b| comp_f64(a,b));
                    returns.push(v[v.len()-1] - v[0]);
                    drawups.push(d[0]);
                    drawdowns.push(d[d.len() - 1]);
                }
            }
            // if (returns.len() == 0) | (drawups.len() == 0) | (drawdowns.len() == 0) { continue }
            let sharpe = vec_mean(&returns).unwrap_or(f64::NAN) / vec_std(&returns).unwrap_or(f64::NAN);
            if !sharpe.is_normal() { continue }
            // let sharpe = vec_sharpe(&returns).unwrap_or(f64::NAN);
            // if !sharpe.is_normal() { println!("sharpe is weird! {}", sharpe) }

            let ann_factor = (252_f64).sqrt();
            drawups.sort_by(|a, b| comp_f64(a,b));
            drawdowns.sort_by(|a, b| comp_f64(a,b));

            // println!("{}, {}", interval, start_time);
            ret.push(StrategyResult {
                interval: *interval,
                start_time: *start_time,
                end_time: end_time,
                sharpe: sharpe*ann_factor,
                max_drawup: drawups[0],
                max_drawdown: drawdowns[drawdowns.len() - 1],
                n_obs: n_obs,
            });
        }
    }

    match ret.len() {
        0 => Err(Box::new(SimpleError::new("Returns length was zero"))),
        _ => Ok(ret)
    }
}


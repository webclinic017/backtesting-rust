use crate::BUS_DAY_CAL;
pub use crate::vector_utils::*;
use rustc_hash::FxHashMap;
use std::fs::File;
use std::io::prelude::*;
use std::error::Error;
use std::time::Duration;
use csv::{ByteRecord, ReaderBuilder};
use serde::de;
use serde_derive::Deserialize;
use bdays::HolidayCalendar;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use std::cmp::Ordering;
use simple_error::SimpleError;


#[derive(Deserialize)]
pub struct Row {
    pub datetime_str: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}
impl Row {
    pub fn datetime(&self) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(&self.datetime_str,
                                      "%Y-%m-%d %H:%M:%S")
            .unwrap()
    }
}

pub fn read_csv<T: de::DeserializeOwned>(file_name: &str) -> Result<Vec<T>, Box<dyn Error>> {
    println!("Reading CSV from {}", file_name);
    let mut file = File::open(file_name)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut rdr = ReaderBuilder::new()
        .from_reader(contents.as_bytes());

    let records = rdr
        .byte_records()
        .collect::<Result<Vec<ByteRecord>, csv::Error>>()?;

    let mut v:Vec<T> = Vec::new();

    for r in records.iter() {
        let row:T = r.deserialize(None)?;
        v.push(row);
    }
    println!("Finished reading CSV");
    Ok(v)
}

use crate::strategy::{StrategyResult, FIELD_NAMES};
pub fn write_csv(v: &Vec<StrategyResult>) -> Result<(), Box<dyn Error>> {
    println!("Writing to csv");
    match v.len() {
        0 => Err(Box::new(SimpleError::new("CSV output has length zero"))),
        _ => {
            let mut wtr = csv::Writer::from_path("returns_test.csv")?;
            wtr.write_record(&FIELD_NAMES)?;
            // for ((i,st, e), (sharpe, drawups, drawdowns, n_obs)) in h {
            for strat in v {
                wtr.write_record(strat.fields_to_strings())?;
            }
            wtr.flush()?;
            println!("Finished writing to csv");
            Ok(())
        },
    }
}

pub fn filter_timeseries_by_events(datetimes: Vec<Row>, event_dates: &Vec<NaiveDate>, back_threshold_bdays: u32, fwd_threshold_bdays: u32) -> Vec<Row> {
    let filter_dates:Vec<NaiveDate> = (-(back_threshold_bdays as i32)..=(fwd_threshold_bdays as i32)).map(|i| {
        event_dates.iter().map(move |&x| BUS_DAY_CAL.advance_bdays(x, i))
    })
        .flatten()
        .collect();

    datetimes.into_iter().filter(|x| filter_dates.contains(&x.datetime().date())).collect()
}

pub fn time_range(start_time: (u32, u32, u32), end_time: (u32, u32, u32), step_mins: u64) -> Vec<NaiveTime> {
    let (start_hr, start_min, start_sec) = start_time;
    let (end_hr, end_min, end_sec) = end_time;

    let start_time_nt: NaiveTime = NaiveTime::from_hms(start_hr, start_min, start_sec);
    let end_time_nt: NaiveTime = NaiveTime::from_hms(end_hr, end_min, end_sec);

    let mut v: Vec<NaiveTime> = vec![start_time_nt];
    let mut i: u64 = 0;
    loop {
        let t = start_time_nt + chrono::Duration::from_std(Duration::from_secs(step_mins*i*60)).unwrap();
        if &t<=&end_time_nt {
            v.push(t);
        }
        else {break}
        i += 1;
    }
    v
}

pub fn add_time(time: &NaiveTime, secs: u64) -> NaiveTime {
    time.clone() + chrono::Duration::from_std(Duration::from_secs(secs)).unwrap()
}

pub fn fill_ids(r: &Vec<i32>) -> Vec<usize> {
    // Check validity
    let unique_check = vec_unique(r);
    if unique_check.len() != 3 {
        return vec![0; r.len()]
    }

    // Fill IDs
    let starts = vec_where_eq(r, &1);
    let ends = vec_where_eq(r, &-1);

    let mut pairs:FxHashMap<usize, usize> = FxHashMap::default();
    for st in starts {
        let abs_diff_vec: Vec<usize> = ends.iter().map(|x| x.wrapping_sub(st)).collect();
        let e:usize = ends[vec_where_eq(&abs_diff_vec, abs_diff_vec.iter().min().unwrap())[0]];

        if e > st {
            pairs.insert(e, st);
        }
    }

    let mut v = vec![0; r.len()];
    for (i, (e, st)) in pairs.into_iter().enumerate() {
        for j in st..=e {
            // std::mem::replace(&mut v[j], i+2);
            v[j] = i+2;
        }
    }
    v
}

pub fn comp_f64(a: &f64, b: &f64) -> Ordering {
    if a < b {
        return Ordering::Less;
    } else if a > b {
        return Ordering::Greater;
    }
    Ordering::Equal
}
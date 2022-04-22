extern crate log;
extern crate csv;
#[macro_use]
extern crate serde_derive;

use crate::vector_utils::*;

pub mod utils {
    use crate::vector_utils::vec_where_eq;
    // use std::collections::HashMap;
    use rustc_hash::FxHashMap;
    use std::fs::File;
    use std::io::prelude::*;
    use std::error::Error;
    use std::time::Duration;
    use csv::{ByteRecord, ReaderBuilder};
    use chrono::{NaiveDateTime, NaiveTime};
    use crate::vec_unique;

    #[derive(Deserialize)]
    pub struct Row {
        pub datetime_str: String,
        pub open: f64,
        pub high: f64,
        pub low: f64,
        pub close: f64,
        pub volume: f64,
        // pub datetime: NaiveDateTime,
    }
    impl Row {
        pub fn datetime(&self) -> NaiveDateTime {
            NaiveDateTime::parse_from_str(&self.datetime_str,
                                          "%Y-%m-%d %H:%M:%S")
                .unwrap()
        }
    }

    pub fn read_csv(file_name: &str) -> Result<Vec<Row>, Box<dyn Error>> {
        let mut file = File::open(file_name)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let mut rdr = ReaderBuilder::new()
            .from_reader(contents.as_bytes());

        let records = rdr
            .byte_records()
            // .records()
            .collect::<Result<Vec<ByteRecord>, csv::Error>>()?;

        let mut v:Vec<Row> = Vec::new();

        // for (i, r) in records.iter().enumerate() {
        for r in records.iter() {
            let row:Row = r.deserialize(None)?;
            v.push(row);
            // if i > 100 {break;}
        }

        Ok(v)
    }

    pub fn write_csv(h: &FxHashMap<(u64, NaiveTime, NaiveTime), f64>, column_names: &[&str]) -> Result<(), Box<dyn Error>> {
        let mut wtr = csv::Writer::from_path("returns_test.csv")?;
        wtr.write_record(column_names)?;
        for ((i,st, e), r) in h {
            wtr.write_record(&[i.to_string(), st.to_string(), e.to_string(), r.to_string()])?;
        }
        wtr.flush()?;
        Ok(())
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
        let max_diff: usize = ends.iter().max().unwrap() - starts.iter().min().unwrap();

        let mut pairs:FxHashMap<usize, usize> = FxHashMap::default();
        for st in starts {
            let abs_diff_vec: Vec<usize> = ends.iter().map(|x| if x>&st {x-st} else {max_diff}).collect();
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
}

pub mod vector_utils {
    use std::cmp::{PartialEq, PartialOrd, Eq};
    // use std::collections::{HashMap, HashSet};
    use std::hash::Hash;
    use rustc_hash::FxHashSet;

    pub fn vec_unique<T: Eq+Hash>(r: &Vec<T>) -> FxHashSet<&T> {
        // let mut unique = (*r).clone();
        // unique.sort();
        // unique.dedup();
        // unique
        let mut s: FxHashSet<&T> = FxHashSet::default();
        for i in r {
            s.insert(i);
        }
        s
    }

    pub fn vec_where_eq<T: PartialEq + PartialOrd>(v: &Vec<T>, val: &T) -> Vec<usize> {
        let z:Vec<usize> = v.iter()
            .enumerate()
            .filter(|(_x, y)| y == &val)
            .map(|(x, _y)| x)
            .collect();
        z
    }
    pub fn vec_where_lt<T: PartialEq + PartialOrd>(v: &Vec<T>, val: &T) -> Vec<usize> {
        let z:Vec<usize> = v.iter()
            .enumerate()
            .filter(|(_x, y)| y < &val)
            .map(|(x, _y)| x)
            .collect();
        z
    }
    pub fn vec_where_gt<T: PartialEq + PartialOrd>(v: &Vec<T>, val: &T) -> Vec<usize> {
        let z:Vec<usize> = v.iter()
            .enumerate()
            .filter(|(_x, y)| y > &val)
            .map(|(x, _y)| x)
            .collect();
        z
        // let mut loc_vec: Vec<usize> = Vec::new();
        // let mut counter:usize = 0;
        // for x in v.iter() {
        //     if x == val {
        //         loc_vec.push(counter);
        //     }
        //     counter += 1;
        // }
        // loc_vec
    }

    pub fn vec_mean(v: &Vec<f64>) -> Option<f64> {
        let sum = v.iter().sum::<f64>() as f64;
        let count = v.len();

        match count {
            positive if positive > 0 => Some(sum / count as f64),
            _ => None,
        }
    }

    pub fn vec_variance(v: &Vec<f64>) -> Option<f64> {
        match (vec_mean(v), v.len()) {
            (Some(data_mean), count) if count > 0 => {
                let variance = v.iter().map(|value| {
                    let diff = data_mean - (*value as f64);

                    diff * diff
                }).sum::<f64>() / (count - 1) as f64;
                Some(variance)
            },
            _ => None
        }
    }
    pub fn vec_std(v: &Vec<f64>) -> Option<f64> {
        match (vec_variance(v), v.len()) {
            (Some(variance), count) if count > 0 => {
                let std = variance.sqrt();
                Some(std)
            },
            _ => None,
        }
    }
}

#[test]
fn playground_test() {
    let mut v = [1,0,0,0,1,-1,0,0,-1,0,0,1,-1,0,0,0,0,0,0,0,1,0,0,0,1,0,0,-1,0,0,0,0,1];

    let n_threads = 4;
    let interval_rng: Vec<u64> = (2..60).map(|x| x).collect();
    let interval_rng_split:Vec<Vec<u64>> = interval_rng.chunks((interval_rng.len()/n_threads + 1)).map(|x| x.to_vec()).collect();

    for i in interval_rng_split {
        println!("{:?}", i);
    }

}

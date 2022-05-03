use std::cmp::{PartialEq, PartialOrd, Eq};
use std::convert::From;
use std::hash::Hash;
use std::iter::Sum;
use std::ops::{Add, Sub, Div, Mul, AddAssign};
use std::process::Output;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use itertools::Itertools;
use log::warn;
use rustc_hash::{FxHashMap, FxHashSet};

pub fn vec_unique<T: Eq+Hash>(r: &Vec<T>) -> FxHashSet<&T> {
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
}
pub fn vec_mean<T>(v: &Vec<T>) -> Option<T>
    where T: Clone + Copy + From<f32> + Into<f64> + PartialOrd + Add<Output = T> + Div<Output = T> + Sum<T> {
    let sum: T = v.iter().map(|&x| x.clone()).sum();
    let count = v.len() as f32;

    match count {
        positive if positive > 0.0 => Some(sum/count.into()),
        _ => {
            warn!("vec_mean: vector has length zero");
            None
        },
    }
}
pub fn vec_variance<T>(v: &Vec<T>) -> Option<T>
    where T: Clone + Copy + From<f32> + Into<f64> + PartialOrd + Add<Output = T> + Div<Output = T>
    + Sum<T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> {
    match (vec_mean(v), v.len()) {
        (Some(data_mean), count) if count > 0 => {
            let variance = v.iter().map(|value| {
                let diff = data_mean - *value;

                diff * diff
            }).sum::<T>() / (count as f32 - 1.0).into();
            Some(variance)
        },
        _ => {
            warn!("vec_variance: vector has length zero");
            None
        }
    }
}
pub fn vec_std<T>(v: &Vec<T>) -> Option<T>
    where T: Clone + Copy + From<f32> + Into<f64> + PartialOrd + Add<Output = T> + Div<Output = T>
    + Sum<T> + Sub<Output = T> + Mul<Output = T> + Div<Output = T> {
    match (vec_variance(v), v.len()) {
        (Some(variance), count) if count > 0 => {
            let std: f64 = variance.into();
            let std: T = (std.sqrt() as f32).into();
            Some(std)
        },
        _ => {
            warn!("vec_std: vector has length zero");
            None
        },
    }
}
pub fn vec_diff(v: &Vec<f64>, diff: usize) -> Option<Vec<f64>> {
    let count = v.len();
    if count <= diff {
        warn!("vec_diff: vector has length {}, which is not greater than diff {}", count, diff);
        return None
    }
    let d:Vec<f64> = (0..(v.len()-diff)).map(|i| &v[i+diff] - &v[i]).collect();
    Some(d)
}
pub fn vec_cumsum<T>(v: &Vec<T>) -> Option<Vec<T>> where T: Add<Output = T> + AddAssign + Into<f64> + From<f64> + Copy {
    let count = v.len();
    if count == 1 { return None }
    let mut u = v.clone();
    u.iter_mut().fold(0.0, |acc, x| {
        *x += acc.into();
        (*x).into()
    });
    Some(u)
}
pub fn vec_add_scalar<T: Copy + Add<Output=T>>(v: &Vec<T>, scalar: T) -> Vec<T> {
    v.into_iter().map(|&x| x + scalar).collect_vec()
}
pub fn vec_sub_scalar<T: Copy + Sub<Output=T>>(v: &Vec<T>, scalar: T) -> Vec<T> {
    v.into_iter().map(|&x| x - scalar).collect_vec()
}
pub fn vec_dates(v: &Vec<NaiveDateTime>) -> Vec<NaiveDate> {
    v.iter().map(|x| x.date()).collect()
}
pub fn vec_times(v: &Vec<NaiveDateTime>) -> Vec<NaiveTime> {
    v.iter().map(|x| x.time()).collect()
}
pub fn vec_rmse<T>(v: &Vec<T>) -> Option<f32>
    where T: Clone + Copy + From<f32> + Into<f64> + PartialOrd + Add<Output = T> + Div<Output = T> + Sum<T>
                            + Mul<Output = T>{
    let r: Vec<T> = v.iter().map(|&x| x*x).collect();
    // match vec_mean(&r.iter().map(|&x| x as f64).collect()) {
    match vec_mean(&r) {
        Some(mean) => Some(mean.into().sqrt() as f32),
        None => None,
    }
}
pub fn extremeum_hashmap_by_values<T: Copy + Into<f64> + PartialOrd>(hm: &FxHashMap<usize, T>, kind: &str) -> (usize, T) {
    let anypair = hm.iter().next().unwrap();
    let mut res = (*anypair.0, *anypair.1);
    for (&i, &x) in hm.iter() {
        match kind {
            "maximum" => if x > res.1 { res = (i, x); },
            "minimum" => if x < res.1 { res = (i, x); },
            _ => panic!("Not acceptable kind {}", kind),
        }
    }
    res
}

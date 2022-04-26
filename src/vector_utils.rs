use std::cmp::{PartialEq, PartialOrd, Eq};
use std::hash::Hash;
use itertools::Itertools;
use rustc_hash::FxHashSet;

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
pub fn vec_diff(v: &Vec<f64>, diff: usize) -> Option<Vec<f64>> {
    let count = v.len();
    if count == 1 { return None }
    let d:Vec<f64> = (0..(v.len()-diff)).map(|i| &v[i+diff] - &v[i]).collect();
    Some(d)
}
pub fn vec_cumsum(v: &Vec<f64>) -> Option<Vec<f64>> {
    let count = v.len();
    if count == 1 { return None }
    let mut u = v.clone();
    u.iter_mut().fold(0.0, |acc, x| {
        *x += acc;
        *x
    });
    Some(u)
}
pub fn vec_add_scalar<T: Copy + std::ops::Add<Output=T>>(v: &Vec<T>, scalar: T) -> Vec<T> {
    v.into_iter().map(|&x| x + scalar).collect_vec()
}
pub fn vec_sub_scalar<T: Copy + std::ops::Sub<Output=T>>(v: &Vec<T>, scalar: T) -> Vec<T> {
    v.into_iter().map(|&x| x - scalar).collect_vec()
}
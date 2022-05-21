use std::iter::Zip;
use crate::utils::*;
use crate::vector_utils::*;
// use std::time::Duration;
use crate::events::get_event_calendar;
use rustc_hash::FxHashMap;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime, Duration};
use log::{error, info, warn};
use log4rs;
use rand::Rng;

#[test]
fn general_test() {
    use bytemuck;
    use rand::prelude::*;

    let mut rng = rand::thread_rng();
    let v: Vec<f32> = (0..1000).map(|_| rng.gen::<f32>()*100.0).collect();

    println!("{v:?}");

}
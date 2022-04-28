use std::iter::Zip;
use crate::utils::*;
use crate::vector_utils::*;
// use std::time::Duration;
use crate::events::get_event_calendar;
use rustc_hash::FxHashMap;
use chrono::{NaiveDateTime, NaiveDate, NaiveTime, Duration};
use log::{error, info, warn};
use log4rs;

#[test]
fn general_test() {

    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    let v = vec![5.2];
    let u:Vec<f64> = vec![];

    vec_diff(&v, 1);
    vec_diff(&u, 1);


}
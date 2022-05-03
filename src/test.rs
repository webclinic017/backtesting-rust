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

    // log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    let events_loc = "C:\\Users\\mbroo\\PycharmProjects\\backtesting\\calendar-event-list-new.csv";
    let event_data: FxHashMap<String, Vec<NaiveDateTime>> = get_event_calendar(events_loc);
    for (event_name, _) in event_data.iter() {
        println!("{}", event_name);
    }
}
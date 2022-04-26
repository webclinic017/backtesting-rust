pub mod vector_utils;
pub mod utils;
pub mod strategy;
pub mod events;

use rustc_hash::FxHashMap;
use chrono::NaiveDateTime;
use crate::events::*;

#[test]
fn playground_test() {
    let data = vec![1,0,0,0,1,-1,0,0,-1,0,0,1,-1,0,0,0,0,0,0,0,1,0,0,0,1,0,0,-1,0,0,0,0,1];
    // let data = vec![1, 3, -2, -2, 1, 0, 1, 2];

    let events_loc = "C:\\Users\\mbroo\\PycharmProjects\\backtesting\\calendar-event-list.csv";
    let events: FxHashMap<String, Vec<NaiveDateTime>> = get_event_calendar(events_loc, &[3]);
    let cpi = events.get("Consumer Price Index ex Food & Energy (YoY)").unwrap();
    println!("{:?}", cpi);
}

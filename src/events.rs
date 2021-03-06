use chrono::NaiveDateTime;
use std::collections::hash_map::Entry;
use rustc_hash::FxHashMap;
use crate::utils::read_csv;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct EventRow {
    // _id: String,
    pub name: String,
    _dt: String,
    // _impact: String,
    // pub currency: String,
}
impl EventRow {
    pub fn datetime(&self) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(&self._dt,
                                      "%Y-%m-%d %H:%M:%S")
            .unwrap()
    }

    // pub fn impact(&self) -> i32 {
    //     match self._impact.as_str() {
    //         "LOW" => 1,
    //         "MED" => 2,
    //         "HIGH" => 3,
    //         _ => 0,
    //     }
    // }
}

pub fn get_event_calendar(file_name: &str) -> FxHashMap<String, Vec<NaiveDateTime>> {
    let raw: Vec<EventRow> = read_csv(file_name).unwrap().into_iter()
        // .filter(|x: &EventRow| impacts.contains(&x.impact()))
        .collect();

    let mut hm:FxHashMap<String, Vec<NaiveDateTime>> = FxHashMap::default();
    for row in raw {
        match hm.entry(row.name.clone()) {
            Entry::Vacant(e) => { e.insert(vec![row.datetime()]); },
            Entry::Occupied(mut e) => { e.get_mut().push(row.datetime()); }
        }
    }
    hm
}

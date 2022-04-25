use chrono::NaiveDateTime;
use crate::utils::read_csv;
use serde_derive::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Event {
    _id: String,
    _start: String,
    pub name: String,
    _impact: String,
    pub currency: String,
}
impl Event {
    pub fn datetime(&self) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(&self._start,
                                      "%m/%d/%Y %H:%M:%S")
            .unwrap()
    }

    pub fn impact(&self) -> i32 {
        match self._impact.as_str() {
            "LOW" => 1,
            "MED" => 2,
            "HIGH" => 3,
            _ => 0,
        }
    }

}

pub fn get_event_calendar(file_name: &str, impacts: &[i32]) -> Vec<Event> {
    read_csv(file_name).unwrap().into_iter()
        .filter(|x: &Event| impacts.contains(&x.impact()))
        .collect()
}

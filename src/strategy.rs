use crate::BUS_DAY_CAL;
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime};
use bdays::HolidayCalendar;

pub const N_FIELDS: usize = 7;
pub static FIELD_NAMES: [&str; N_FIELDS] = ["interval", "start time", "end time", "sharpe", "max drawup", "max drawdown", "n obs"];
pub struct StrategyResult {
    pub interval: u64,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub sharpe: f64,
    pub max_drawup: f64,
    pub max_drawdown: f64,
    pub n_obs: usize,
}
impl StrategyResult {
    pub fn fields_to_strings(&self) -> [String; N_FIELDS] {
        [self.interval.to_string(), self.start_time.to_string(), self.end_time.to_string(),
            self.sharpe.to_string(), self.max_drawup.to_string(), self.max_drawdown.to_string(),
            self.n_obs.to_string()]
    }
}
impl Default for StrategyResult {
    fn default() -> Self {
        Self {
            interval: 0,
            start_time: NaiveTime::from_hms(1,0,0),
            end_time: NaiveTime::from_hms(1,0,0),
            sharpe: f64::NAN,
            max_drawup: f64::NAN,
            max_drawdown: f64::NAN,
            n_obs: 0,
        }
    }
}

pub trait ContextCondition {}

pub struct DayOfCondition;
impl DayOfCondition {
    pub fn run(datetimes: &Vec<NaiveDateTime>, event_dates: &Vec<NaiveDate>) -> Vec<bool> {
        datetimes.iter()
            .map(|x| event_dates.contains(&x.date()))
            .collect()
    }
}
impl ContextCondition for DayOfCondition {}

pub struct DayOffsetCondition;
impl DayOffsetCondition {
    pub fn run(datetimes: &Vec<NaiveDateTime>, event_dates: &Vec<NaiveDate>,
               early_offset_days: i64, late_offset_days: i64, is_bus_days: bool) -> Vec<bool> {
        assert!(early_offset_days<=late_offset_days);
        let mut event_date_ranges = event_dates.clone();
        for &dt in event_dates {
            for i in early_offset_days..=late_offset_days {
                if is_bus_days {
                    event_date_ranges.push(BUS_DAY_CAL.advance_bdays(dt, -i as i32))
                }
                else {
                    event_date_ranges.push(dt + Duration::days(-i));
                }
            }
        }
        datetimes.iter().map(|x| event_date_ranges.contains(&x.date())).collect()
    }
}
impl ContextCondition for DayOffsetCondition {}

pub fn day_of_strat(datetimes: &Vec<NaiveDateTime>, event_dates: &Vec<NaiveDate>) -> Vec<bool> {
    datetimes.iter()
        .map(|x| event_dates.contains(&x.date()))
        .collect()
}

pub fn days_offset_strat(datetimes: &Vec<NaiveDateTime>, event_dates: &Vec<NaiveDate>,
                         early_offset_days: i64, late_offset_days: i64, is_bus_days: bool) -> Vec<bool> {
    assert!(early_offset_days<=late_offset_days);
    let mut event_date_ranges = event_dates.clone();
    for &dt in event_dates {
        for i in early_offset_days..=late_offset_days {
            if is_bus_days {
                event_date_ranges.push(BUS_DAY_CAL.advance_bdays(dt, -i as i32))
            }
            else {
                event_date_ranges.push(dt + Duration::days(-i));
            }
        }
    }
    datetimes.iter().map(|x| event_date_ranges.contains(&x.date())).collect()
}



use chrono::NaiveTime;

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

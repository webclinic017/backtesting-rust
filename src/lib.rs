static BUS_DAY_CAL:bdays::calendars::us::USSettlement = bdays::calendars::us::USSettlement;
// static LOGGER: log4rs::config::file = log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

pub mod vector_utils;
pub mod utils;
pub mod strategy;
pub mod events;
pub mod analysis;

#[cfg(test)]
mod test;

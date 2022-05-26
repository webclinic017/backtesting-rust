use std::error::Error;
use serde::{de, Deserialize, Deserializer};
use std::str::FromStr;

#[derive(Debug, Default)]
pub struct DateTime {
    pub year: u32,
    pub month: u32,
    pub day: u32,
    pub hours: u32,
    pub mins: u32,
    pub secs: u32,
    pub secs_from_midnight: u32,
}
impl FromStr for DateTime {
    type Err = Box<dyn Error>;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        //          1234567890123456789
        // FORMAT: "YYYY-MM-DD HH:MM:SS"
        let year = u32::from_str(&s[0..4])?;
        let month = u32::from_str(&s[5..7])?;
        let day = u32::from_str(&s[8..10])?;

        let hours = u32::from_str(&s[11..13])?;
        let mins = u32::from_str(&s[14..16])?;
        let secs = u32::from_str(&s[17..])?;

        let secs_from_midnight = secs + mins * 60 + hours * 60 * 60;

        Ok(Self {
            year,
            month,
            day,
            hours,
            mins,
            secs,
            secs_from_midnight,
        })
    }
}
impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(de::Error::custom)
    }
}


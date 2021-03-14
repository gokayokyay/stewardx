use std::str::FromStr;

use chrono::DateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Interval {
    Seconds,
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Years,
}

impl Interval {
    pub fn to_seconds(&self, value: i64) -> i64 {
        // Dont judge pls will refactor soon (hopefully)
        match &self {
            Interval::Seconds => return value,
            Interval::Minutes => return value * 60,
            Interval::Hours => return value * 60 * 60,
            Interval::Days => return value * 60 * 60 * 24,
            Interval::Weeks => return value * 60 * 60 * 24 * 7,
            Interval::Months => return value * 60 * 60 * 24 * 7 * 30,
            Interval::Years => return value * 60 * 60 * 24 * 7 * 30 * 12,
        }
    }
}

pub enum FrequencyDeserializeError {
    MalformedData,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Frequency {
    Every(Box<String>),
    Hook,
    AfterInterval,
}

impl Frequency {
    pub fn get_next(&self) -> Option<DateTime<chrono::Utc>> {
        match &self {
            Frequency::Every(s) => {
                let cronified = if s.split(" ").collect::<Vec<&str>>().len() < 7 {
                    format!("0 {}", s)
                } else {
                    s.to_string()
                };

                return cron::Schedule::from_str(&cronified)
                    .unwrap()
                    .upcoming(chrono::Utc)
                    .next();
            }
            Frequency::AfterInterval => {}
            Frequency::Hook => {}
        }
        return None;
    }
}

impl ToString for Frequency {
    fn to_string(&self) -> String {
        match &self {
            Frequency::Every(s) => return format!("Every({})", s),
            Frequency::AfterInterval => return String::from("AfterInterval"),
            Frequency::Hook => return String::from("Hook"),
        }
    }
}

impl FromStr for Frequency {
    type Err = FrequencyDeserializeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with("Every") {
            Ok(Self::Every(
                s.strip_prefix("Every(")
                    .unwrap()
                    .strip_suffix(")")
                    .unwrap()
                    .to_string()
                    .into(),
            ))
        } else if s.starts_with("AfterInterval") {
            Ok(Self::AfterInterval)
        } else {
            Err(FrequencyDeserializeError::MalformedData)
        }
    }
}

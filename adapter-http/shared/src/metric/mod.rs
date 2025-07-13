use myhomelab_prelude::current_timestamp;

pub mod create;
pub mod query;

const HOUR: i64 = 60 * 60;
const DAY: i64 = HOUR * 24;
const WEEK: i64 = DAY * 7;
const MONTH: i64 = WEEK * 4;

#[derive(Debug, serde::Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum TimeRange {
    LastHour,
    LastDay,
    LastWeek,
    LastMonth,
    Range {
        start: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        end: Option<i64>,
    },
}

impl Default for TimeRange {
    fn default() -> Self {
        Self::LastDay
    }
}

impl From<TimeRange> for myhomelab_metric::query::TimeRange {
    fn from(value: TimeRange) -> Self {
        match value {
            TimeRange::LastHour => myhomelab_metric::query::TimeRange {
                start: current_timestamp() as i64 - HOUR,
                end: None,
            },
            TimeRange::LastDay => myhomelab_metric::query::TimeRange {
                start: current_timestamp() as i64 - DAY,
                end: None,
            },
            TimeRange::LastWeek => myhomelab_metric::query::TimeRange {
                start: current_timestamp() as i64 - WEEK,
                end: None,
            },
            TimeRange::LastMonth => myhomelab_metric::query::TimeRange {
                start: current_timestamp() as i64 - MONTH,
                end: None,
            },
            TimeRange::Range { start, end } => myhomelab_metric::query::TimeRange { start, end },
        }
    }
}

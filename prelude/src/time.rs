pub const HOUR: i64 = 60 * 60;
pub const DAY: i64 = HOUR * 24;
pub const WEEK: i64 = DAY * 7;
pub const MONTH: i64 = WEEK * 4;

pub fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time went backward")
        .as_secs()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub struct AbsoluteTimeRange {
    pub start: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<i64>,
}

impl AbsoluteTimeRange {
    pub fn since(start: i64) -> Self {
        Self { start, end: None }
    }

    pub fn until(mut self, end: i64) -> Self {
        self.end = Some(end);
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RelativeTimeRange {
    LastHour,
    LastDay,
    LastWeek,
    LastMonth,
}

impl RelativeTimeRange {
    pub const fn duration(&self) -> i64 {
        match self {
            Self::LastHour => HOUR,
            Self::LastDay => DAY,
            Self::LastWeek => WEEK,
            Self::LastMonth => MONTH,
        }
    }

    pub fn into_absolute(self) -> AbsoluteTimeRange {
        let now = current_timestamp() as i64;
        AbsoluteTimeRange {
            start: now - self.duration(),
            end: None,
        }
    }
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, derive_more::From, serde::Deserialize, serde::Serialize,
)]
#[serde(untagged)]
pub enum TimeRange {
    Absolute(AbsoluteTimeRange),
    Relative(RelativeTimeRange),
}

impl Default for TimeRange {
    fn default() -> Self {
        Self::Relative(RelativeTimeRange::LastDay)
    }
}

impl TimeRange {
    pub fn into_absolute(self) -> AbsoluteTimeRange {
        match self {
            Self::Absolute(inner) => inner,
            Self::Relative(inner) => inner.into_absolute(),
        }
    }
}

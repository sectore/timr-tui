use std::fmt;
use std::time::Duration;

pub const ONE_SECOND: Duration = Duration::from_secs(1);
pub const ONE_MINUTE: Duration = Duration::from_secs(SECS_PER_MINUTE);
pub const ONE_HOUR: Duration = Duration::from_secs(MINS_PER_HOUR * SECS_PER_MINUTE);

// unstable
// https://doc.rust-lang.org/src/core/time.rs.html#32
pub const SECS_PER_MINUTE: u64 = 60;
// unstable
// https://doc.rust-lang.org/src/core/time.rs.html#34
pub const MINS_PER_HOUR: u64 = 60;
// unstable
// https://doc.rust-lang.org/src/core/time.rs.html#36
const HOURS_PER_DAY: u64 = 24;

#[derive(Debug, Clone, Copy, PartialOrd)]
pub struct DurationEx {
    inner: Duration,
}

impl PartialEq for DurationEx {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl From<Duration> for DurationEx {
    fn from(inner: Duration) -> Self {
        Self { inner }
    }
}

impl From<DurationEx> for Duration {
    fn from(ex: DurationEx) -> Self {
        ex.inner
    }
}

impl DurationEx {
    pub fn seconds(&self) -> u64 {
        self.inner.as_secs()
    }

    pub fn seconds_mod(&self) -> u64 {
        self.seconds() % SECS_PER_MINUTE
    }

    pub fn hours(&self) -> u64 {
        self.seconds() / (SECS_PER_MINUTE * MINS_PER_HOUR)
    }

    pub fn hours_mod(&self) -> u64 {
        self.hours() % HOURS_PER_DAY
    }

    pub fn minutes(&self) -> u64 {
        self.seconds() / MINS_PER_HOUR
    }

    pub fn minutes_mod(&self) -> u64 {
        self.minutes() % SECS_PER_MINUTE
    }

    // deciseconds
    pub fn decis(&self) -> u64 {
        (self.inner.subsec_millis() / 100) as u64
    }
    // milliseconds
    pub fn millis(&self) -> u128 {
        self.inner.as_millis()
    }

    pub fn saturating_add(&self, ex: DurationEx) -> Self {
        let inner = self.inner.saturating_add(ex.inner);
        Self { inner }
    }

    pub fn saturating_sub(&self, ex: DurationEx) -> Self {
        let inner = self.inner.saturating_sub(ex.inner);
        Self { inner }
    }
}

impl fmt::Display for DurationEx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.hours() >= 10 {
            write!(
                f,
                "{:02}:{:02}:{:02}",
                self.hours_mod(),
                self.minutes_mod(),
                self.seconds_mod(),
            )
        } else if self.hours() >= 1 {
            write!(
                f,
                "{}:{:02}:{:02}",
                self.hours(),
                self.minutes_mod(),
                self.seconds_mod()
            )
        } else if self.minutes() >= 10 {
            write!(f, "{:02}:{:02}", self.minutes_mod(), self.seconds_mod())
        } else if self.minutes() >= 1 {
            write!(f, "{}:{:02}", self.minutes(), self.seconds_mod())
        } else if self.seconds() >= 10 {
            write!(f, "{:02}", self.seconds_mod())
        } else {
            write!(f, "{}", self.seconds())
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::time::Duration;

    #[test]
    fn test_fmt() {
        // hh:mm:ss
        let ex: DurationEx = Duration::from_secs(36001).into();
        assert_eq!(format!("{}", ex), "10:00:01");
        // h:mm:ss
        let ex: DurationEx = Duration::from_secs(3601).into();
        assert_eq!(format!("{}", ex), "1:00:01");
        // mm:ss
        let ex: DurationEx = Duration::from_secs(71).into();
        assert_eq!(format!("{}", ex), "1:11");
        // m:ss
        let ex: DurationEx = Duration::from_secs(61).into();
        assert_eq!(format!("{}", ex), "1:01");
        // ss
        let ex: DurationEx = Duration::from_secs(11).into();
        assert_eq!(format!("{}", ex), "11");
        // s
        let ex: DurationEx = Duration::from_secs(1).into();
        assert_eq!(format!("{}", ex), "1");
    }
}

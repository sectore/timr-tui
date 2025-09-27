use color_eyre::{
    Report,
    eyre::{ensure, eyre},
};
use std::fmt;
use std::time::Duration;

pub const ONE_DECI_SECOND: Duration = Duration::from_millis(100);
pub const ONE_SECOND: Duration = Duration::from_secs(1);
pub const ONE_MINUTE: Duration = Duration::from_secs(SECS_PER_MINUTE);
pub const ONE_HOUR: Duration = Duration::from_secs(MINS_PER_HOUR * SECS_PER_MINUTE);
pub const ONE_DAY: Duration = Duration::from_secs(HOURS_PER_DAY * MINS_PER_HOUR * SECS_PER_MINUTE);
pub const ONE_YEAR: Duration =
    Duration::from_secs(DAYS_PER_YEAR * HOURS_PER_DAY * MINS_PER_HOUR * SECS_PER_MINUTE);

// unstable
// https://doc.rust-lang.org/src/core/time.rs.html#32
pub const SECS_PER_MINUTE: u64 = 60;
// unstable
// https://doc.rust-lang.org/src/core/time.rs.html#34
pub const MINS_PER_HOUR: u64 = 60;
// unstable
// https://doc.rust-lang.org/src/core/time.rs.html#36
const HOURS_PER_DAY: u64 = 24;

// Days per year
// "There are 365 days in a year in a common year of the Gregorian calendar and 366 days in a leap year.
// Leap years occur every four years. The average number of days in a year is 365.2425 days."
// ^ https://www.math.net/days-in-a-year
const DAYS_PER_YEAR: u64 = 365; // ignore leap year of 366 days

// max. 999y 364d 23:59:59 (1000 years - 1 second)
pub const MAX_DURATION: Duration = ONE_YEAR.saturating_mul(1000).saturating_sub(ONE_SECOND);

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
    pub fn years(&self) -> u64 {
        self.days() / DAYS_PER_YEAR
    }

    pub fn days(&self) -> u64 {
        self.hours() / HOURS_PER_DAY
    }

    /// Days in a year
    pub fn days_mod(&self) -> u64 {
        self.days() % DAYS_PER_YEAR
    }

    pub fn hours(&self) -> u64 {
        self.seconds() / (SECS_PER_MINUTE * MINS_PER_HOUR)
    }

    /// Hours as 24-hour clock
    pub fn hours_mod(&self) -> u64 {
        self.hours() % HOURS_PER_DAY
    }

    /// Hours as 12-hour clock
    pub fn hours_mod_12(&self) -> u64 {
        // 0 => 12,
        // 1..=12 => hours,
        // 13..=23 => hours - 12,
        (self.hours_mod() + 11) % 12 + 1
    }

    pub fn minutes(&self) -> u64 {
        self.seconds() / MINS_PER_HOUR
    }

    pub fn minutes_mod(&self) -> u64 {
        self.minutes() % SECS_PER_MINUTE
    }

    pub fn seconds(&self) -> u64 {
        self.inner.as_secs()
    }

    pub fn seconds_mod(&self) -> u64 {
        self.seconds() % SECS_PER_MINUTE
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

    pub fn to_string_with_decis(self) -> String {
        format!("{}.{}", self, self.decis())
    }
}

impl fmt::Display for DurationEx {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.years() >= 1 {
            write!(
                f,
                "{}y {}d {:02}:{:02}:{:02}",
                self.years(),
                self.days_mod(),
                self.hours_mod(),
                self.minutes_mod(),
                self.seconds_mod(),
            )
        } else if self.hours() >= HOURS_PER_DAY {
            write!(
                f,
                "{}d {:02}:{:02}:{:02}",
                self.days_mod(),
                self.hours_mod(),
                self.minutes_mod(),
                self.seconds_mod(),
            )
        } else if self.hours() >= 10 {
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

/// Parses  `Duration` from `hh:mm:ss`, `mm:ss` or `ss`
pub fn parse_duration(arg: &str) -> Result<Duration, Report> {
    let parts: Vec<&str> = arg.split(':').rev().collect();

    let parse_seconds = |s: &str| -> Result<u64, Report> {
        let secs = s.parse::<u64>().map_err(|_| eyre!("Invalid seconds"))?;
        ensure!(secs < 60, "Seconds must be less than 60.");
        Ok(secs)
    };

    let parse_minutes = |m: &str| -> Result<u64, Report> {
        let mins = m.parse::<u64>().map_err(|_| eyre!("Invalid minutes"))?;
        ensure!(mins < 60, "Minutes must be less than 60.");
        Ok(mins)
    };

    let parse_hours = |h: &str| -> Result<u64, Report> {
        let hours = h.parse::<u64>().map_err(|_| eyre!("Invalid hours"))?;
        ensure!(hours < 100, "Hours must be less than 100.");
        Ok(hours)
    };

    let seconds = match parts.as_slice() {
        [ss] => parse_seconds(ss)?,
        [ss, mm] => {
            let s = parse_seconds(ss)?;
            let m = parse_minutes(mm)?;
            m * 60 + s
        }
        [ss, mm, hh] => {
            let s = parse_seconds(ss)?;
            let m = parse_minutes(mm)?;
            let h = parse_hours(hh)?;
            h * 60 * 60 + m * 60 + s
        }
        _ => return Err(eyre!("Invalid time format. Use 'ss', mm:ss, or hh:mm:ss")),
    };

    Ok(Duration::from_secs(seconds))
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::time::Duration;

    #[test]
    fn test_fmt() {
        const DAY_IN_SECONDS: u64 = 86400; // 24 * 60 * 60
        const YEAR_IN_SECONDS: u64 = 31536000; // 365 * 86400
        // 1y Dd hh:mm:ss (single year)
        let ex: DurationEx =
            Duration::from_secs(YEAR_IN_SECONDS + 10 * DAY_IN_SECONDS + 36001).into();
        assert_eq!(format!("{ex}"), "1y 10d 10:00:01");
        // 5y Dd hh:mm:ss (multiple years)
        let ex: DurationEx =
            Duration::from_secs(5 * YEAR_IN_SECONDS + 100 * DAY_IN_SECONDS + 36001).into();
        assert_eq!(format!("{ex}"), "5y 100d 10:00:01");
        // 150y Dd hh:mm:ss (more than 100 years)
        let ex: DurationEx =
            Duration::from_secs(150 * YEAR_IN_SECONDS + 200 * DAY_IN_SECONDS + 36001).into();
        assert_eq!(format!("{ex}"), "150y 200d 10:00:01");
        // 366d hh:mm:ss (days more than a year)
        let ex: DurationEx = Duration::from_secs(366 * DAY_IN_SECONDS + 36001).into();
        assert_eq!(format!("{ex}"), "1y 1d 10:00:01");
        // 1d hh:mm:ss (single day)
        let ex: DurationEx = Duration::from_secs(DAY_IN_SECONDS + 36001).into();
        assert_eq!(format!("{ex}"), "1d 10:00:01");
        // 2d hh:mm:ss (multiple days)
        let ex: DurationEx = Duration::from_secs(2 * DAY_IN_SECONDS + 36001).into();
        assert_eq!(format!("{ex}"), "2d 10:00:01");
        // hh:mm:ss
        let ex: DurationEx = Duration::from_secs(36001).into();
        assert_eq!(format!("{ex}"), "10:00:01");
        // h:mm:ss
        let ex: DurationEx = Duration::from_secs(3601).into();
        assert_eq!(format!("{ex}"), "1:00:01");
        // mm:ss
        let ex: DurationEx = Duration::from_secs(71).into();
        assert_eq!(format!("{ex}"), "1:11");
        // m:ss
        let ex: DurationEx = Duration::from_secs(61).into();
        assert_eq!(format!("{ex}"), "1:01");
        // ss
        let ex: DurationEx = Duration::from_secs(11).into();
        assert_eq!(format!("{ex}"), "11");
        // s
        let ex: DurationEx = Duration::from_secs(1).into();
        assert_eq!(format!("{ex}"), "1");
    }

    #[test]
    fn test_saturating_sub() {
        let ex: DurationEx = Duration::from_secs(10).into();
        let ex2: DurationEx = Duration::from_secs(1).into();
        let ex3 = ex.saturating_sub(ex2);
        assert_eq!(format!("{ex3}"), "9");
    }

    #[test]
    fn test_saturating_add() {
        let ex: DurationEx = Duration::from_secs(10).into();
        let ex2: DurationEx = Duration::from_secs(1).into();
        let ex3 = ex.saturating_add(ex2);
        assert_eq!(format!("{ex3}"), "11");
    }

    #[test]
    fn test_hours_mod_12() {
        // 24 -> 12
        let ex: DurationEx = ONE_HOUR.saturating_mul(24).into();
        let result = ex.hours_mod_12();
        assert_eq!(result, 12);

        // 12 -> 12
        let ex: DurationEx = ONE_HOUR.saturating_mul(12).into();
        let result = ex.hours_mod_12();
        assert_eq!(result, 12);

        // 0 -> 12
        let ex: DurationEx = ONE_SECOND.into();
        let result = ex.hours_mod_12();
        assert_eq!(result, 12);

        // 13 -> 1
        let ex: DurationEx = ONE_HOUR.saturating_mul(13).into();
        let result = ex.hours_mod_12();
        assert_eq!(result, 1);

        // 1 -> 1
        let ex: DurationEx = ONE_HOUR.saturating_mul(1).into();
        let result = ex.hours_mod_12();
        assert_eq!(result, 1);
    }

    #[test]
    fn test_parse_duration() {
        // ss
        assert_eq!(parse_duration("50").unwrap(), Duration::from_secs(50));
        // mm:ss
        assert_eq!(
            parse_duration("01:30").unwrap(),
            Duration::from_secs(60 + 30)
        );
        // hh:mm:ss
        assert_eq!(
            parse_duration("01:30:00").unwrap(),
            Duration::from_secs(60 * 60 + 30 * 60)
        );
        // errors
        assert!(parse_duration("1:60").is_err()); // invalid seconds
        assert!(parse_duration("60:00").is_err()); // invalid minutes
        assert!(parse_duration("100:00:00").is_err()); // invalid hours
        assert!(parse_duration("abc").is_err()); // invalid input
        assert!(parse_duration("01:02:03:04").is_err()); // too many parts
    }
}

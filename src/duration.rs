use color_eyre::{
    Report,
    eyre::{ensure, eyre},
};
use std::cmp::min;
use std::fmt;
use std::time::Duration;
use time::OffsetDateTime;

use crate::common::AppTime;

// unstable
// https://doc.rust-lang.org/src/core/time.rs.html#32
pub const SECS_PER_MINUTE: u64 = 60;
// unstable
// https://doc.rust-lang.org/src/core/time.rs.html#34
pub const MINS_PER_HOUR: u64 = 60;
// unstable
// https://doc.rust-lang.org/src/core/time.rs.html#36
const HOURS_PER_DAY: u64 = 24;

pub const ONE_DECI_SECOND: Duration = Duration::from_millis(100);
pub const ONE_SECOND: Duration = Duration::from_secs(1);
pub const ONE_MINUTE: Duration = Duration::from_secs(SECS_PER_MINUTE);
pub const ONE_HOUR: Duration = Duration::from_secs(MINS_PER_HOUR * SECS_PER_MINUTE);
pub const ONE_DAY: Duration = Duration::from_secs(HOURS_PER_DAY * MINS_PER_HOUR * SECS_PER_MINUTE);
pub const ONE_YEAR: Duration =
    Duration::from_secs(DAYS_PER_YEAR * HOURS_PER_DAY * MINS_PER_HOUR * SECS_PER_MINUTE);

// Days per year
// "There are 365 days in a year in a common year of the Gregorian calendar and 366 days in a leap year.
// Leap years occur every four years. The average number of days in a year is 365.2425 days."
// ^ https://www.math.net/days-in-a-year
const DAYS_PER_YEAR: u64 = 365; // ignore leap year of 366 days

// max. 999y 364d 23:59:59.9 (1000 years - 1 decisecond)
pub const MAX_DURATION: Duration = ONE_YEAR
    .saturating_mul(1000)
    .saturating_sub(ONE_DECI_SECOND);

/// Trait for duration types that can be displayed in clock widgets.
///
/// This trait abstracts over different duration calculation strategies:
/// - `DurationEx`: Uses fixed 365-day years (fast, simple)
/// - `CalendarDuration`: Uses actual calendar dates (accounts for leap years)
pub trait ClockDuration {
    /// Total years
    fn years(&self) -> u64;

    /// Total days
    fn days(&self) -> u64;

    /// Days within the current year (0-364 or 0-365 for leap years)
    fn days_mod(&self) -> u64;

    /// Total hours
    fn hours(&self) -> u64;

    /// Hours within the current day (0-23)
    fn hours_mod(&self) -> u64;

    /// Hours as 12-hour clock (1-12)
    fn hours_mod_12(&self) -> u64;

    /// Total minutes
    fn minutes(&self) -> u64;

    /// Minutes within the current hour (0-59)
    fn minutes_mod(&self) -> u64;

    /// Total seconds
    fn seconds(&self) -> u64;

    /// Seconds within the current minute (0-59)
    fn seconds_mod(&self) -> u64;

    /// Deciseconds (tenths of a second, 0-9)
    fn decis(&self) -> u64;

    /// Total milliseconds
    fn millis(&self) -> u128;
}

/// `Duration` with direction in time (past or future)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DirectedDuration {
    /// Time `until` a future moment (positive `Duration`)
    Until(Duration),
    /// Time `since` a past moment (negative duration, but still represented as positive `Duration`)
    Since(Duration),
}

impl From<DirectedDuration> for Duration {
    fn from(directed: DirectedDuration) -> Self {
        match directed {
            DirectedDuration::Until(d) => d,
            DirectedDuration::Since(d) => d,
        }
    }
}

impl DirectedDuration {
    pub fn from_offset_date_times(value_a: OffsetDateTime, value_b: OffsetDateTime) -> Self {
        let diff = value_a - value_b;

        if diff.is_negative() {
            Self::Since(Duration::from_millis(
                diff.whole_milliseconds().unsigned_abs() as u64,
            ))
        } else {
            Self::Until(Duration::from_millis(diff.whole_milliseconds() as u64))
        }
    }
}

/// Calendar-aware duration that accounts for leap years.
///
/// Unlike `DurationEx` which uses fixed 365-day years, this calculates
/// years and days based on actual calendar dates, properly handling leap years.
///
/// All calculations are performed on-demand from the stored dates.
#[derive(Debug, Clone, Copy)]
pub struct CalendarDuration {
    earlier: OffsetDateTime,
    later: OffsetDateTime,
}

impl CalendarDuration {
    /// Create a new CalendarDuration between two dates.
    ///
    /// The order of arguments doesn't matter - the struct will automatically
    /// determine which is earlier and which is later.
    pub fn between(a: OffsetDateTime, b: OffsetDateTime) -> Self {
        if a <= b {
            Self {
                earlier: a,
                later: b,
            }
        } else {
            Self {
                earlier: b,
                later: a,
            }
        }
    }
}

impl From<CalendarDuration> for Duration {
    fn from(cal_duration: CalendarDuration) -> Self {
        let diff = cal_duration.later - cal_duration.earlier;
        Duration::from_millis(diff.whole_milliseconds().max(0) as u64)
    }
}

impl ClockDuration for CalendarDuration {
    fn years(&self) -> u64 {
        let mut years = (self.later.year() - self.earlier.year()) as i64;

        // Check if we've completed a full year by comparing month/day/time
        let intermediate = self
            .earlier
            .replace_year(self.later.year())
            .unwrap_or(self.earlier);

        if intermediate > self.later {
            years -= 1;
        }

        years.max(0) as u64
    }

    fn days_mod(&self) -> u64 {
        let year_count = self.years();

        // Calculate intermediate date after adding complete years
        let target_year = self.earlier.year() + year_count as i32;
        let intermediate = self
            .earlier
            .replace_year(target_year)
            .unwrap_or(self.earlier);

        let remaining = self.later - intermediate;
        remaining.whole_days().max(0) as u64
    }

    fn days(&self) -> u64 {
        (self.later - self.earlier).whole_days().max(0) as u64
    }

    fn hours_mod(&self) -> u64 {
        let total_hours = (self.later - self.earlier).whole_hours();
        (total_hours % 24).max(0) as u64
    }

    fn hours(&self) -> u64 {
        (self.later - self.earlier).whole_hours().max(0) as u64
    }

    fn hours_mod_12(&self) -> u64 {
        let hours = self.hours_mod();
        (hours + 11) % 12 + 1
    }

    fn minutes_mod(&self) -> u64 {
        let total_minutes = (self.later - self.earlier).whole_minutes();
        (total_minutes % 60).max(0) as u64
    }

    fn minutes(&self) -> u64 {
        (self.later - self.earlier).whole_minutes().max(0) as u64
    }

    fn seconds_mod(&self) -> u64 {
        let total_seconds = (self.later - self.earlier).whole_seconds();
        (total_seconds % 60).max(0) as u64
    }

    fn seconds(&self) -> u64 {
        (self.later - self.earlier).whole_seconds().max(0) as u64
    }

    fn decis(&self) -> u64 {
        let total_millis = (self.later - self.earlier).whole_milliseconds();
        ((total_millis % 1000) / 100).max(0) as u64
    }

    fn millis(&self) -> u128 {
        (self.later - self.earlier).whole_milliseconds().max(0) as u128
    }
}

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

// TODO: Question: Should we call `DurationEx::anything` here or move all of those
// functions into this `impl`?
impl ClockDuration for DurationEx {
    fn years(&self) -> u64 {
        self.days() / DAYS_PER_YEAR
    }

    fn days(&self) -> u64 {
        self.hours() / HOURS_PER_DAY
    }

    fn days_mod(&self) -> u64 {
        self.days() % DAYS_PER_YEAR
    }

    fn hours(&self) -> u64 {
        self.seconds() / (SECS_PER_MINUTE * MINS_PER_HOUR)
    }

    fn hours_mod(&self) -> u64 {
        self.hours() % HOURS_PER_DAY
    }

    fn hours_mod_12(&self) -> u64 {
        // 0 => 12,
        // 1..=12 => hours,
        // 13..=23 => hours - 12,
        (self.hours_mod() + 11) % 12 + 1
    }

    fn minutes(&self) -> u64 {
        self.seconds() / MINS_PER_HOUR
    }

    fn minutes_mod(&self) -> u64 {
        self.minutes() % SECS_PER_MINUTE
    }

    fn seconds(&self) -> u64 {
        self.inner.as_secs()
    }

    fn seconds_mod(&self) -> u64 {
        self.seconds() % SECS_PER_MINUTE
    }

    fn decis(&self) -> u64 {
        (self.inner.subsec_millis() / 100) as u64
    }

    fn millis(&self) -> u128 {
        self.inner.as_millis()
    }
}

impl DurationEx {
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
        use ClockDuration as _; // Import trait methods
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

/// Parse seconds (must be < 60)
fn parse_seconds(s: &str) -> Result<u8, Report> {
    let secs = s.parse::<u8>().map_err(|_| eyre!("Invalid seconds"))?;
    ensure!(secs < 60, "Seconds must be less than 60.");
    Ok(secs)
}

/// Parse minutes (must be < 60)
fn parse_minutes(m: &str) -> Result<u8, Report> {
    let mins = m.parse::<u8>().map_err(|_| eyre!("Invalid minutes"))?;
    ensure!(mins < 60, "Minutes must be less than 60.");
    Ok(mins)
}

/// Parse hours
fn parse_hours(h: &str) -> Result<u8, Report> {
    let hours = h.parse::<u8>().map_err(|_| eyre!("Invalid hours"))?;
    Ok(hours)
}

/// Parses `DirectedDuration` from following formats:
/// - `yyyy-mm-dd hh:mm:ss`
/// - `yyyy-mm-dd hh:mm`
/// - `hh:mm:ss`
/// - `hh:mm`
/// - `mm`
///
/// Returns `DirectedDuration::Until` for future times, `DirectedDuration::Since` for past times
#[allow(dead_code)]
pub fn parse_duration_by_time(arg: &str) -> Result<DirectedDuration, Report> {
    use time::{OffsetDateTime, PrimitiveDateTime, macros::format_description};

    let now: OffsetDateTime = AppTime::new().into();

    let target_time = if arg.contains('-') {
        // First: `YYYY-MM-DD HH:MM:SS`
        // Then: `YYYY-MM-DD HH:MM`
        let format_with_seconds =
            format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
        let format_without_seconds = format_description!("[year]-[month]-[day] [hour]:[minute]");

        let pdt = PrimitiveDateTime::parse(arg, format_with_seconds)
            .or_else(|_| PrimitiveDateTime::parse(arg, format_without_seconds))
            .map_err(|e| {
                eyre!("Invalid datetime '{}'. Use format 'yyyy-mm-dd hh:mm:ss' or 'yyyy-mm-dd hh:mm'. Error: {}", arg, e)
            })?;
        pdt.assume_offset(now.offset())
    } else {
        // Parse time parts: interpret as HH:MM:SS, HH:MM, or SS
        let parts: Vec<&str> = arg.split(':').collect();

        let (hour, minute, second) = match parts.as_slice() {
            [mm] => {
                // Single part: treat as minutes in current hour
                let m = parse_minutes(mm)?;
                (now.hour(), m, 0)
            }
            [hh, mm] => {
                // Two parts: treat as HH:MM (time of day)
                let h = parse_hours(hh)?;
                let m = parse_minutes(mm)?;
                (h, m, 0)
            }
            [hh, mm, ss] => {
                // Three parts: HH:MM:SS
                let h = parse_hours(hh)?;
                let m = parse_minutes(mm)?;
                let s = parse_seconds(ss)?;
                (h, m, s)
            }
            _ => {
                return Err(eyre!(
                    "Invalid time format. Use 'hh:mm:ss', 'hh:mm', or 'mm'"
                ));
            }
        };

        now.replace_time(
            time::Time::from_hms(hour, minute, second).map_err(|_| eyre!("Invalid time"))?,
        )
    };

    let mut duration_secs = (target_time - now).whole_seconds();

    // `Since` for past times
    if duration_secs < 0 {
        duration_secs *= -1;
        Ok(DirectedDuration::Since(Duration::from_secs(
            duration_secs as u64,
        )))
    } else
    // `Until` for future times,
    {
        Ok(DirectedDuration::Until(Duration::from_secs(
            duration_secs as u64,
        )))
    }
}

/// Parses  `Duration` from `hh:mm:ss`, `mm:ss` or `ss`
pub fn parse_duration(arg: &str) -> Result<Duration, Report> {
    let parts: Vec<&str> = arg.split(':').collect();

    let (hours, minutes, seconds) = match parts.as_slice() {
        [ss] => {
            // Single part: seconds only
            let s = parse_seconds(ss)?;
            (0u64, 0u64, s as u64)
        }
        [mm, ss] => {
            // Two parts: MM:SS
            let m = parse_minutes(mm)?;
            let s = parse_seconds(ss)?;
            (0u64, m as u64, s as u64)
        }
        [hh, mm, ss] => {
            // Three parts: HH:MM:SS
            let h = parse_hours(hh)?;
            let m = parse_minutes(mm)?;
            let s = parse_seconds(ss)?;
            (h as u64, m as u64, s as u64)
        }
        _ => {
            return Err(eyre!(
                "Invalid time format. Use 'ss', 'mm:ss', or 'hh:mm:ss'"
            ));
        }
    };

    let total_seconds = hours * 3600 + minutes * 60 + seconds;
    Ok(Duration::from_secs(total_seconds))
}

/// Similar to `parse_duration`, but it parses `years` and `days` in addition
/// Formats: `Yy Dd`, `Yy` or `Dd` in any combination to other time formats
/// Examples: `10y 3d 12:10:03`, `2d 10:00`, `101y 33`, `5:30`
pub fn parse_long_duration(arg: &str) -> Result<Duration, Report> {
    let arg = arg.trim();

    // parts are separated by whitespaces:
    // 3 parts: years, days, time
    let parts: Vec<&str> = arg.split_whitespace().collect();
    ensure!(parts.len() <= 3, "Invalid format. Too many parts.");

    let mut total_duration = Duration::ZERO;
    let mut time_part: Option<&str> = None;

    for part in parts {
        // years
        if let Some(years_str) = part.strip_suffix('y') {
            let years = years_str
                .parse::<u64>()
                .map_err(|_| eyre!("Invalid years value: '{}'", years_str))?;
            total_duration = total_duration.saturating_add(ONE_YEAR.saturating_mul(years as u32));
        }
        // days
        else if let Some(days_str) = part.strip_suffix('d') {
            let days = days_str
                .parse::<u64>()
                .map_err(|_| eyre!("Invalid days value: '{}'", days_str))?;
            total_duration = total_duration.saturating_add(ONE_DAY.saturating_mul(days as u32));
        }
        // possible time format
        else {
            time_part = Some(part);
        }
    }

    // time format
    if let Some(time) = time_part {
        let time_duration = parse_duration(time)?;
        total_duration = total_duration.saturating_add(time_duration);
    }

    // avoid overflow
    total_duration = min(MAX_DURATION, total_duration);

    Ok(total_duration)
}

#[cfg(test)]
mod tests {

    use super::ClockDuration; // Import trait for DurationEx methods
    use super::*;
    use std::time::Duration;

    const MINUTE_IN_SECONDS: u64 = ONE_MINUTE.as_secs();
    const HOUR_IN_SECONDS: u64 = ONE_HOUR.as_secs();
    const DAY_IN_SECONDS: u64 = ONE_DAY.as_secs();
    const YEAR_IN_SECONDS: u64 = ONE_YEAR.as_secs();

    #[test]
    fn test_fmt() {
        // 1y Dd hh:mm:ss (single year)
        let ex: DurationEx =
            Duration::from_secs(YEAR_IN_SECONDS + 10 * DAY_IN_SECONDS + 36001).into();
        assert_eq!(format!("{ex}"), "1y 10d 10:00:01");
        // 5y Dd hh:mm:ss (multiple years)
        let ex: DurationEx = Duration::from_secs(
            5 * YEAR_IN_SECONDS + 100 * DAY_IN_SECONDS + 10 * HOUR_IN_SECONDS + 1,
        )
        .into();
        assert_eq!(format!("{ex}"), "5y 100d 10:00:01");
        // 150y Dd hh:mm:ss (more than 100 years)
        let ex: DurationEx = Duration::from_secs(
            150 * YEAR_IN_SECONDS + 200 * DAY_IN_SECONDS + 10 * HOUR_IN_SECONDS + 1,
        )
        .into();
        assert_eq!(format!("{ex}"), "150y 200d 10:00:01");
        // 366d hh:mm:ss (days more than a year)
        let ex: DurationEx =
            Duration::from_secs(366 * DAY_IN_SECONDS + 10 * HOUR_IN_SECONDS + 1).into();
        assert_eq!(format!("{ex}"), "1y 1d 10:00:01");
        // 1d hh:mm:ss (single day)
        let ex: DurationEx = Duration::from_secs(DAY_IN_SECONDS + 10 * HOUR_IN_SECONDS + 1).into();
        assert_eq!(format!("{ex}"), "1d 10:00:01");
        // 2d hh:mm:ss (multiple days)
        let ex: DurationEx =
            Duration::from_secs(2 * DAY_IN_SECONDS + 10 * HOUR_IN_SECONDS + 1).into();
        assert_eq!(format!("{ex}"), "2d 10:00:01");
        // hh:mm:ss
        let ex: DurationEx = Duration::from_secs(10 * HOUR_IN_SECONDS + 1).into();
        assert_eq!(format!("{ex}"), "10:00:01");
        // h:mm:ss
        let ex: DurationEx = Duration::from_secs(HOUR_IN_SECONDS + 1).into();
        assert_eq!(format!("{ex}"), "1:00:01");
        // mm:ss
        let ex: DurationEx = Duration::from_secs(MINUTE_IN_SECONDS + 11).into();
        assert_eq!(format!("{ex}"), "1:11");
        // m:ss
        let ex: DurationEx = Duration::from_secs(MINUTE_IN_SECONDS + 1).into();
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
    fn test_from_offset_date_times() {
        use time::macros::datetime;

        // Future time (Until)
        let now = datetime!(2024-01-01 12:00:00).assume_utc();
        let future = datetime!(2024-01-01 13:00:00).assume_utc();
        assert!(matches!(
            DirectedDuration::from_offset_date_times(future, now),
            DirectedDuration::Until(_)
        ));

        // Past time (Since)
        assert!(matches!(
            DirectedDuration::from_offset_date_times(now, future),
            DirectedDuration::Since(_)
        ));

        // Same time (Until with 0 duration)
        assert!(matches!(
            DirectedDuration::from_offset_date_times(now, now),
            DirectedDuration::Until(_)
        ));
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
        assert!(parse_duration("abc").is_err()); // invalid input
        assert!(parse_duration("01:02:03:04").is_err()); // too many parts
    }

    #[test]
    fn test_parse_duration_by_time() {
        // YYYY-MM-DD HH:MM:SS - future
        assert!(matches!(
            parse_duration_by_time("2050-06-15 14:30:45"),
            Ok(DirectedDuration::Until(_))
        ));

        // YYYY-MM-DD HH:MM - future
        assert!(matches!(
            parse_duration_by_time("2050-06-15 14:30"),
            Ok(DirectedDuration::Until(_))
        ));

        // HH:MM:SS - past
        assert!(matches!(
            parse_duration_by_time("2000-01-01 23:59:59"),
            Ok(DirectedDuration::Since(_))
        ));

        // HH:MM - Until or Since depending on current time
        assert!(parse_duration_by_time("18:00").is_ok());

        // MM - Until or Since depending on current time
        assert!(parse_duration_by_time("45").is_ok());

        // errors
        assert!(parse_duration_by_time("60").is_err()); // invalid minutes
        assert!(parse_duration_by_time("24:00").is_err()); // invalid hours
        assert!(parse_duration_by_time("24:00:00").is_err()); // invalid hours
        assert!(parse_duration_by_time("2030-13-01 12:00:00").is_err()); // invalid month
        assert!(parse_duration_by_time("2030-06-32 12:00:00").is_err()); // invalid day
        assert!(parse_duration_by_time("abc").is_err()); // invalid input
        assert!(parse_duration_by_time("01:02:03:04").is_err()); // too many parts
    }

    #[test]
    fn test_parse_long_duration() {
        // `Yy`
        assert_eq!(
            parse_long_duration("10y").unwrap(),
            Duration::from_secs(10 * YEAR_IN_SECONDS)
        );
        assert_eq!(
            parse_long_duration("101y").unwrap(),
            Duration::from_secs(101 * YEAR_IN_SECONDS)
        );

        // `Dd`
        assert_eq!(
            parse_long_duration("2d").unwrap(),
            Duration::from_secs(2 * DAY_IN_SECONDS)
        );

        // `Yy Dd`
        assert_eq!(
            parse_long_duration("10y 3d").unwrap(),
            Duration::from_secs(10 * YEAR_IN_SECONDS + 3 * DAY_IN_SECONDS)
        );

        // `Yy Dd hh:mm:ss`
        assert_eq!(
            parse_long_duration("10y 3d 12:10:03").unwrap(),
            Duration::from_secs(
                10 * YEAR_IN_SECONDS
                    + 3 * DAY_IN_SECONDS
                    + 12 * HOUR_IN_SECONDS
                    + 10 * MINUTE_IN_SECONDS
                    + 3
            )
        );

        // `Dd hh:mm`
        assert_eq!(
            parse_long_duration("2d 10:00").unwrap(),
            Duration::from_secs(2 * DAY_IN_SECONDS + 10 * 60)
        );

        // `Yy ss`
        assert_eq!(
            parse_long_duration("101y 33").unwrap(),
            Duration::from_secs(101 * YEAR_IN_SECONDS + 33)
        );

        // time formats (backward compatibility with `parse_duration`)
        assert_eq!(
            parse_long_duration("5:30").unwrap(),
            Duration::from_secs(5 * MINUTE_IN_SECONDS + 30)
        );
        assert_eq!(
            parse_long_duration("01:30:45").unwrap(),
            Duration::from_secs(HOUR_IN_SECONDS + 30 * MINUTE_IN_SECONDS + 45)
        );
        assert_eq!(parse_long_duration("42").unwrap(), Duration::from_secs(42));

        // `Dd ss`
        assert_eq!(
            parse_long_duration("5d 30").unwrap(),
            Duration::from_secs(5 * DAY_IN_SECONDS + 30)
        );

        // `Yy hh:mm:ss`
        assert_eq!(
            parse_long_duration("1y 01:30:00").unwrap(),
            Duration::from_secs(YEAR_IN_SECONDS + HOUR_IN_SECONDS + 30 * MINUTE_IN_SECONDS)
        );

        // Whitespace handling
        assert_eq!(
            parse_long_duration("  2d   10:00  ").unwrap(),
            Duration::from_secs(2 * DAY_IN_SECONDS + 10 * MINUTE_IN_SECONDS)
        );

        // MAX_DURATION clamping
        assert_eq!(parse_long_duration("1000y").unwrap(), MAX_DURATION);
        assert_eq!(
            parse_long_duration("999y 364d 23:59:59").unwrap(),
            Duration::from_secs(
                999 * YEAR_IN_SECONDS
                    + 364 * DAY_IN_SECONDS
                    + 23 * HOUR_IN_SECONDS
                    + 59 * MINUTE_IN_SECONDS
                    + 59
            )
        );

        // errors
        assert!(parse_long_duration("10x").is_err()); // invalid unit
        assert!(parse_long_duration("abc").is_err()); // invalid input
        assert!(parse_long_duration("10y 60:00").is_err()); // invalid minutes in time part
        assert!(parse_long_duration("5d 1:60").is_err()); // invalid seconds in time part
        assert!(parse_long_duration("1y 2d 3d 4:00").is_err()); // too many parts (4 parts)
        assert!(parse_long_duration("1y 2d 3h 4m 5s").is_err()); // too many parts (5 parts)
    }

    #[test]
    fn test_calendar_duration_leap_year() {
        use time::macros::datetime;

        // 2024 is a leap year (366 days)
        let start = datetime!(2024-01-01 00:00:00 UTC);
        let end = datetime!(2025-01-01 00:00:00 UTC);
        let cal_dur = CalendarDuration::between(start, end);

        assert_eq!(cal_dur.years(), 1, "Should be exactly 1 year");
        assert_eq!(cal_dur.days_mod(), 0, "Should be 0 remaining days");
        assert_eq!(cal_dur.days(), 366, "2024 has 366 days (leap year)");
    }

    #[test]
    fn test_calendar_duration_non_leap_year() {
        use time::macros::datetime;

        // 2023 is not a leap year (365 days)
        let start = datetime!(2023-01-01 00:00:00 UTC);
        let end = datetime!(2024-01-01 00:00:00 UTC);
        let cal_dur = CalendarDuration::between(start, end);

        assert_eq!(cal_dur.years(), 1, "Should be exactly 1 year");
        assert_eq!(cal_dur.days_mod(), 0, "Should be 0 remaining days");
        assert_eq!(cal_dur.days(), 365, "2023 has 365 days (non-leap year)");
    }

    #[test]
    fn test_calendar_duration_partial_year_with_leap_day() {
        use time::macros::datetime;

        // Span including Feb 29, 2024
        let start = datetime!(2024-02-01 00:00:00 UTC);
        let end = datetime!(2024-03-15 00:00:00 UTC);
        let cal_dur = CalendarDuration::between(start, end);

        assert_eq!(cal_dur.years(), 0, "Should be 0 years");
        // Feb 2024 has 29 days, so: 29 days (rest of Feb) + 15 days (March) = 44 days
        assert_eq!(
            cal_dur.days(),
            43,
            "Should be 43 days (29 in Feb + 14 partial March)"
        );
    }

    #[test]
    fn test_calendar_duration_partial_year_without_leap_day() {
        use time::macros::datetime;

        // Same dates but in 2023 (non-leap year)
        let start = datetime!(2023-02-01 00:00:00 UTC);
        let end = datetime!(2023-03-15 00:00:00 UTC);
        let cal_dur = CalendarDuration::between(start, end);

        assert_eq!(cal_dur.years(), 0, "Should be 0 years");
        // Feb 2023 has 28 days, so: 28 days (rest of Feb) + 15 days (March) = 43 days
        assert_eq!(
            cal_dur.days(),
            42,
            "Should be 42 days (28 in Feb + 14 partial March)"
        );
    }

    #[test]
    fn test_calendar_duration_multiple_years_spanning_leap_years() {
        use time::macros::datetime;

        // From 2023 (non-leap) through 2024 (leap) to 2025
        let start = datetime!(2023-03-01 10:00:00 UTC);
        let end = datetime!(2025-03-01 10:00:00 UTC);
        let cal_dur = CalendarDuration::between(start, end);

        assert_eq!(cal_dur.years(), 2, "Should be exactly 2 years");
        assert_eq!(cal_dur.days_mod(), 0, "Should be 0 remaining days");
        // Total days: 365 (2023 partial + 2024 partial) + 366 (full 2024 year conceptually included)
        // Actually: From 2023-03-01 to 2025-03-01 = 365 + 366 = 731 days
        assert_eq!(cal_dur.days(), 731, "Should be 731 total days");
    }

    #[test]
    fn test_calendar_duration_year_boundary() {
        use time::macros::datetime;

        // Test incomplete year - just before year boundary
        let start = datetime!(2024-01-01 00:00:00 UTC);
        let end = datetime!(2024-12-31 23:59:59 UTC);
        let cal_dur = CalendarDuration::between(start, end);

        assert_eq!(cal_dur.years(), 0, "Should be 0 years (not complete)");
        assert_eq!(cal_dur.days(), 365, "Should be 365 days");
    }

    #[test]
    fn test_calendar_duration_hours_minutes_seconds() {
        use time::macros::datetime;

        let start = datetime!(2024-01-01 10:30:45 UTC);
        let end = datetime!(2024-01-02 14:25:50 UTC);
        let cal_dur = CalendarDuration::between(start, end);

        assert_eq!(cal_dur.years(), 0);
        assert_eq!(cal_dur.days(), 1);
        assert_eq!(cal_dur.hours_mod(), 3, "Should be 3 hours past midnight");
        assert_eq!(cal_dur.minutes_mod(), 55, "Should be 55 minutes");
        assert_eq!(cal_dur.seconds_mod(), 5, "Should be 5 seconds");
    }

    #[test]
    fn test_calendar_duration_reversed_dates() {
        use time::macros::datetime;

        // CalendarDuration::between should handle reversed order
        let later = datetime!(2025-01-01 00:00:00 UTC);
        let earlier = datetime!(2024-01-01 00:00:00 UTC);
        let cal_dur = CalendarDuration::between(later, earlier);

        assert_eq!(cal_dur.years(), 1, "Should still calculate 1 year");
        assert_eq!(cal_dur.days(), 366, "Should still be 366 days");
    }

    #[test]
    fn test_calendar_duration_same_date() {
        use time::macros::datetime;

        let date = datetime!(2024-06-15 12:00:00 UTC);
        let cal_dur = CalendarDuration::between(date, date);

        assert_eq!(cal_dur.years(), 0);
        assert_eq!(cal_dur.days(), 0);
        assert_eq!(cal_dur.hours(), 0);
        assert_eq!(cal_dur.minutes(), 0);
        assert_eq!(cal_dur.seconds(), 0);
    }

    #[test]
    fn test_calendar_duration_deciseconds() {
        use time::macros::datetime;

        let start = datetime!(2024-01-01 00:00:00.000 UTC);
        let end = datetime!(2024-01-01 00:00:00.750 UTC);
        let cal_dur = CalendarDuration::between(start, end);

        assert_eq!(
            cal_dur.decis(),
            7,
            "Should be 7 deciseconds (750ms = 7.5 decis, truncated to 7)"
        );
        assert_eq!(cal_dur.millis(), 750, "Should be 750 milliseconds");
    }
}

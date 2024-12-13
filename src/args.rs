use clap::Parser;
use std::time::Duration;

#[derive(Parser)]
pub struct Args {
    #[arg(long, short, value_parser = parse_duration,
        default_value="10:00" /* 10min */,
        help = "Countdown time to start from. Format: seconds, 'mm:ss', or 'hh:mm:ss'"
    )]
    pub countdown: Duration,
    #[arg(long, short, value_parser = parse_duration,
        default_value="25:00" /* 25min */,
        help = "Work time to count down from. Format: seconds, 'mm:ss', or 'hh:mm:ss'"
    )]
    pub work: Duration,
    #[arg(long, short, value_parser = parse_duration,
        default_value="5:00" /* 5min */,
        help = "Pause time to count down from. Format: seconds, 'mm:ss', or 'hh:mm:ss'"
    )]
    pub pause: Duration,
}

fn parse_duration(arg: &str) -> Result<Duration, String> {
    if let Ok(seconds) = arg.parse::<u64>() {
        return Ok(Duration::from_secs(seconds));
    }

    let parts: Vec<&str> = arg.split(':').collect();
    if parts.len() > 3 {
        return Err("Invalid time format. Use seconds, mm:ss, or hh:mm:ss".to_string());
    }

    let mut duration = Duration::ZERO;

    let seconds = parts
        .last()
        .ok_or("Missing seconds")?
        .parse::<u64>()
        .map_err(|_| "Invalid seconds")?;
    if seconds >= 60 {
        return Err("Seconds must be less than 60".to_string());
    }
    duration = duration.saturating_add(Duration::from_secs(seconds));

    if let Some(&minutes_str) = parts.get(parts.len().wrapping_sub(2)) {
        let minutes = minutes_str.parse::<u64>().map_err(|_| "Invalid minutes")?;
        if minutes >= 60 {
            return Err("Minutes must be less than 60".to_string());
        }
        duration = duration.saturating_add(Duration::from_secs(minutes * 60));
    }

    if let Some(&hours_str) = parts.first() {
        if parts.len() == 3 {
            let hours = hours_str.parse::<u64>().map_err(|_| "Invalid hours")?;
            if hours >= 100 {
                return Err("Hours must be less than 100".to_string());
            }
            duration = duration.saturating_add(Duration::from_secs(hours * 3600));
        }
    }

    Ok(duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        // Seconds
        assert_eq!(parse_duration("60").unwrap(), Duration::from_secs(60));

        // MM:SS
        assert_eq!(parse_duration("01:30").unwrap(), Duration::from_secs(90));

        // HH:MM:SS
        assert_eq!(
            parse_duration("01:30:00").unwrap(),
            Duration::from_secs(5400)
        );

        // Invalid formats
        assert!(parse_duration("1:60").is_err()); // invalid seconds
        assert!(parse_duration("60:00").is_err()); // invalid minutes
        assert!(parse_duration("100:00:00").is_err()); // invalid hours
        assert!(parse_duration("abc").is_err()); // invalid input
        assert!(parse_duration("01:02:03:04").is_err()); // too many parts
    }
}

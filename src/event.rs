use time::macros::format_description;

#[derive(Debug, Clone)]
pub struct Event {
    pub date_time: time::PrimitiveDateTime,
    pub title: Option<String>,
}

pub fn get_default_event() -> Event {
    Event {
        date_time: time::PrimitiveDateTime::parse(
            // First (initial) commit of `timr-tui`
            // https://github.com/sectore/timr-tui/pull/1/commits/49c59bc2769faa1e31f8cb980a87c4caaa5149b3
            "2024-11-27 20:55",
            format_description!("[year]-[month]-[day] [hour]:[minute]"),
        )
        .unwrap(),
        title: Some("first (initial) commit".into()),
    }
}

/// Parses an `Event`
/// Supports two formats:
/// (1) "YYYY-MM-DD HH:MM:SS"
/// (2) "time=YYYY-MM-DD HH:MM:SS,title=my event"
pub fn parse_event(s: &str) -> Result<Event, String> {
    let s = s.trim();

    // check + parse (2)
    if s.contains('=') {
        parse_event_key_value(s)
    } else {
        // parse (1)
        parse_event_date_time(s)
    }
}

/// Parses an `Event` based on "YYYY-MM-DD HH:MM:SS" format
fn parse_event_date_time(s: &str) -> Result<Event, String> {
    let time = time::PrimitiveDateTime::parse(
        s,
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    )
    .map_err(|e| {
        format!(
            "Failed to parse event date_time '{}': {}. Expected format: 'YYYY-MM-DD HH:MM:SS'",
            s, e
        )
    })?;

    Ok(Event {
        date_time: time,
        title: None,
    })
}

/// Parses an `Event` defined by a `key=value` pair.
/// Valid keys: `time` and `title`.
/// Format: "time=YYYY-MM-DD HH:MM:SS,title=my event"
fn parse_event_key_value(s: &str) -> Result<Event, String> {
    let mut time_str = None;
    let mut title_str = None;

    // k/v pairs are splitted by commas
    for part in s.split(',') {
        let part = part.trim();
        if let Some((key, value)) = part.split_once('=') {
            match key.trim() {
                "time" => time_str = Some(value.trim()),
                "title" => title_str = Some(value.trim()),
                unknown => {
                    return Err(format!(
                        "Unknown key '{}'. Valid keys: 'time', 'title'",
                        unknown
                    ));
                }
            }
        } else {
            return Err(format!(
                "Invalid key=value pair: '{}'. Expected format: 'key=value'",
                part
            ));
        }
    }

    let time_str = time_str.ok_or(
        "Missing required 'time' field. Expected format: 'time=YYYY-MM-DD HH:MM:SS[,title=...]'",
    )?;
    let time = time::PrimitiveDateTime::parse(
        time_str,
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second]"),
    )
    .map_err(|e| {
        format!(
            "Failed to parse event time '{}': {}. Expected format: 'YYYY-MM-DD HH:MM:SS'",
            time_str, e
        )
    })?;

    let title = title_str.filter(|t| !t.is_empty()).map(|t| t.to_string());

    Ok(Event {
        date_time: time,
        title,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn test_parse_event() {
        // Simple format: time only
        let result = parse_event("2024-01-01 14:30:00").unwrap();
        assert_eq!(result.date_time, datetime!(2024-01-01 14:30:00));
        assert_eq!(result.title, None);

        // Simple format: with leading/trailing whitespace (outer trim works)
        let result = parse_event("  2025-12-25 12:30:00  ").unwrap();
        assert_eq!(result.date_time, datetime!(2025-12-25 12:30:00));
        assert_eq!(result.title, None);

        // Key=value format: time only
        let result = parse_event("time=2025-10-10 14:30:00").unwrap();
        assert_eq!(result.date_time, datetime!(2025-10-10 14:30:00));
        assert_eq!(result.title, None);

        // Key=value format: time and title
        let result = parse_event("time=2025-10-10 14:30:00,title=Team Meeting").unwrap();
        assert_eq!(result.date_time, datetime!(2025-10-10 14:30:00));
        assert_eq!(result.title, Some("Team Meeting".to_string()));

        // Key=value format: order independent
        let result = parse_event("title=Stand-up,time=2025-10-10 09:00:00").unwrap();
        assert_eq!(result.date_time, datetime!(2025-10-10 09:00:00));
        assert_eq!(result.title, Some("Stand-up".to_string()));

        // Key=value format: title with spaces and special chars
        let result =
            parse_event("time=2025-10-10 14:30:00,title=Sprint Planning: Q1 Review").unwrap();
        assert_eq!(result.date_time, datetime!(2025-10-10 14:30:00));
        assert_eq!(result.title, Some("Sprint Planning: Q1 Review".to_string()));

        // Key=value format: empty title treated as None
        let result = parse_event("time=2025-10-10 14:30:00,title=").unwrap();
        assert_eq!(result.date_time, datetime!(2025-10-10 14:30:00));
        assert_eq!(result.title, None);

        // Key=value format: whitespace handling
        let result = parse_event(" time = 2025-10-10 14:30:00 , title = My Event ").unwrap();
        assert_eq!(result.date_time, datetime!(2025-10-10 14:30:00));
        assert_eq!(result.title, Some("My Event".to_string()));

        // Error cases: invalid time format
        assert!(parse_event("2025-13-01 00:00:00").is_err());
        assert!(parse_event("invalid").is_err());
        assert!(parse_event("2025/10/10 14:30:00").is_err());

        // Error cases: missing time in key=value format
        assert!(parse_event("title=My Event").is_err());

        // Error cases: unknown key
        assert!(parse_event("time=2025-10-10 14:30:00,foo=bar").is_err());

        // Error cases: malformed key=value pair
        assert!(parse_event("time=2025-10-10 14:30:00,notapair").is_err());
    }
}

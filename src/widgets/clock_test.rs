use crate::{
    common::ClockTypeId,
    duration::{
        DurationEx, MAX_DURATION, ONE_DAY, ONE_DECI_SECOND, ONE_HOUR, ONE_MINUTE, ONE_SECOND,
        ONE_YEAR,
    },
    widgets::clock::*,
};
use std::time::Duration;

fn default_args() -> ClockStateArgs {
    ClockStateArgs {
        initial_value: ONE_HOUR,
        current_value: ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    }
}

#[test]
fn test_type_id() {
    let c = ClockState::<Timer>::new(default_args());
    assert!(matches!(c.get_type_id(), ClockTypeId::Timer));
    let c = ClockState::<Countdown>::new(default_args());
    assert!(matches!(c.get_type_id(), ClockTypeId::Countdown));
}

#[test]
fn test_get_format_seconds() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_SECOND * 5,
        current_value: ONE_SECOND * 5,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });
    // S
    assert_eq!(c.get_format(), &Format::S);
    // Ss
    c.set_current_value(Duration::from_secs(15).into());
    assert_eq!(c.get_format(), &Format::Ss);
}

#[test]
fn test_get_format_minutes() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_MINUTE,
        current_value: ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });
    // MSs
    assert_eq!(c.get_format(), &Format::MSs);
    // MmSs
    c.set_current_value((ONE_MINUTE * 11).into()); // 10+ minutes
    assert_eq!(c.get_format(), &Format::MmSs);
}

#[test]
fn test_get_format_hours() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_HOUR,
        current_value: ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });
    // HMmSS
    assert_eq!(c.get_format(), &Format::HMmSs);
    // HhMmSs
    c.set_current_value((10 * ONE_HOUR).into());
    assert_eq!(c.get_format(), &Format::HhMmSs);
}

#[test]
fn test_format_by_duration_boundaries() {
    // S
    assert_eq!(
        format_by_duration::<DurationEx>(&(ONE_SECOND * 9).into()),
        Format::S
    );
    // Ss
    assert_eq!(
        format_by_duration::<DurationEx>(&(10 * ONE_SECOND).into()),
        Format::Ss
    );
    // Ss
    assert_eq!(
        format_by_duration::<DurationEx>(&(59 * ONE_SECOND).into()),
        Format::Ss
    );
    // MSs
    assert_eq!(
        format_by_duration::<DurationEx>(&ONE_MINUTE.into()),
        Format::MSs
    );
    // HhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(ONE_DAY.saturating_sub(ONE_SECOND)).into()),
        Format::HhMmSs
    );
    // DHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&ONE_DAY.into()),
        Format::DHhMmSs
    );
    // DHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&((10 * ONE_DAY).saturating_sub(ONE_SECOND)).into()),
        Format::DHhMmSs
    );
    // DdHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(10 * ONE_DAY).into()),
        Format::DdHhMmSs
    );
    // DdHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&((100 * ONE_DAY).saturating_sub(ONE_SECOND)).into()),
        Format::DdHhMmSs
    );
    // DddHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(100 * ONE_DAY).into()),
        Format::DddHhMmSs
    );
    // DddHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(ONE_YEAR.saturating_sub(ONE_SECOND).into())),
        Format::DddHhMmSs
    );
    // YDHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&ONE_YEAR.into()),
        Format::YDHhMmSs
    );
    // YDdHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(
            &(ONE_YEAR + (100 * ONE_DAY).saturating_sub(ONE_SECOND)).into()
        ),
        Format::YDdHhMmSs
    );
    // YDddHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(ONE_YEAR + 100 * ONE_DAY).into()),
        Format::YDddHhMmSs
    );
    // YDddHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&((10 * ONE_YEAR).saturating_sub(ONE_SECOND)).into()),
        Format::YDddHhMmSs
    );
    // YyDHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(10 * ONE_YEAR).into()),
        Format::YyDHhMmSs
    );
    // YyDdHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(10 * ONE_YEAR + 10 * ONE_DAY).into()),
        Format::YyDdHhMmSs
    );
    // YyDdHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(
            &(10 * ONE_YEAR + (100 * ONE_DAY).saturating_sub(ONE_SECOND)).into()
        ),
        Format::YyDdHhMmSs
    );
    // YyDddHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(10 * ONE_YEAR + 100 * ONE_DAY).into()),
        Format::YyDddHhMmSs
    );
    // YyDddHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&((100 * ONE_YEAR).saturating_sub(ONE_SECOND)).into()),
        Format::YyDddHhMmSs
    );
    // YyyDHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(100 * ONE_YEAR).into()),
        Format::YyyDHhMmSs
    );
    // YyyDdHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(100 * ONE_YEAR + 10 * ONE_DAY).into()),
        Format::YyyDdHhMmSs
    );
    // YyyDdHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(
            &(100 * ONE_YEAR + (100 * ONE_DAY).saturating_sub(ONE_SECOND)).into()
        ),
        Format::YyyDdHhMmSs
    );
    // YyyDddHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(100 * ONE_YEAR + 100 * ONE_DAY).into()),
        Format::YyyDddHhMmSs
    );

    // YyyyDdHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(1000 * ONE_YEAR + 10 * ONE_DAY).into()),
        Format::YyyyDdHhMmSs
    );
    // YyyyDdHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(
            &(1000 * ONE_YEAR + (100 * ONE_DAY).saturating_sub(ONE_SECOND)).into()
        ),
        Format::YyyyDdHhMmSs
    );
    // YyyyDddHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(1000 * ONE_YEAR + 100 * ONE_DAY).into()),
        Format::YyyyDddHhMmSs
    );
}

#[test]
fn test_format_by_duration_days() {
    // DHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&ONE_DAY.into()),
        Format::DHhMmSs
    );
    // DdHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(10 * ONE_DAY).into()),
        Format::DdHhMmSs
    );
    // DddHhMmSs
    assert_eq!(
        format_by_duration::<DurationEx>(&(101 * ONE_DAY).into()),
        Format::DddHhMmSs
    );
}

#[test]
fn test_format_by_duration_years() {
    // YDHhMmSs (1 year, 0 days)
    assert_eq!(
        format_by_duration::<DurationEx>(&ONE_YEAR.into()),
        Format::YDHhMmSs
    );

    // YDHhMmSs (1 year, 1 day)
    assert_eq!(
        format_by_duration::<DurationEx>(&(ONE_YEAR + ONE_DAY).into()),
        Format::YDHhMmSs
    );

    // YDdHhMmSs (1 year, 10 days)
    assert_eq!(
        format_by_duration::<DurationEx>(&(ONE_YEAR + 10 * ONE_DAY).into()),
        Format::YDdHhMmSs
    );

    // YDddHhMmSs (1 year, 100 days)
    assert_eq!(
        format_by_duration::<DurationEx>(&(ONE_YEAR + 100 * ONE_DAY).into()),
        Format::YDddHhMmSs
    );

    // YyDHhMmSs (10 years)
    assert_eq!(
        format_by_duration::<DurationEx>(&(10 * ONE_YEAR).into()),
        Format::YyDHhMmSs
    );

    // YyDdHhMmSs (10 years, 10 days)
    assert_eq!(
        format_by_duration::<DurationEx>(&(10 * ONE_YEAR + 10 * ONE_DAY).into()),
        Format::YyDdHhMmSs
    );

    // YyDddHhMmSs (10 years, 100 days)
    assert_eq!(
        format_by_duration::<DurationEx>(&(10 * ONE_YEAR + 100 * ONE_DAY).into()),
        Format::YyDddHhMmSs
    );

    // YyyDHhMmSs (100 years)
    assert_eq!(
        format_by_duration::<DurationEx>(&(100 * ONE_YEAR).into()),
        Format::YyyDHhMmSs
    );

    // YyyDdHhMmSs (100 years, 10 days)
    assert_eq!(
        format_by_duration::<DurationEx>(&(100 * ONE_YEAR + 10 * ONE_DAY).into()),
        Format::YyyDdHhMmSs
    );

    // YyyDddHhMmSs (100 years, 100 days)
    assert_eq!(
        format_by_duration::<DurationEx>(&(100 * ONE_YEAR + 100 * ONE_DAY).into()),
        Format::YyyDddHhMmSs
    );
}

#[test]
fn test_default_edit_mode_hhmmss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        with_decis: true,
        ..default_args()
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_default_edit_mode_mmss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_MINUTE,
        current_value: ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });
    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_default_edit_mode_ss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_SECOND,
        current_value: ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });
    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_up_stays_in_seconds() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_MINUTE - ONE_SECOND,
        current_value: ONE_MINUTE - ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_up();
    // Edit mode should stay on seconds
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_up_stays_in_minutes() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_HOUR - ONE_SECOND,
        current_value: ONE_HOUR - ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
    c.edit_up();
    // Edit mode should stay on minutes
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_up_stays_in_hours() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_DAY - ONE_SECOND,
        current_value: ONE_DAY - ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    c.edit_up();
    // Edit mode should stay on hours
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
}

#[test]
fn test_edit_up_stays_in_days() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_YEAR - ONE_DAY,
        current_value: ONE_YEAR - ONE_DAY,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    c.toggle_edit();
    c.edit_next(); // Hours
    c.edit_next(); // Days
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Days, _)));
    c.edit_up();
    // Edit mode should stay on days
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Days, _)));
}

#[test]
fn test_edit_up_overflow_protection() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: MAX_DURATION.saturating_sub(ONE_SECOND),
        current_value: MAX_DURATION.saturating_sub(ONE_SECOND),
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    c.toggle_edit();
    c.edit_next(); // Hours
    c.edit_next(); // Days
    c.edit_next(); // Years
    c.edit_up(); // +1y
    assert!(Duration::from(*c.get_current_value()) <= MAX_DURATION);
    c.edit_prev(); // Days
    c.edit_up(); // +1d
    assert!(Duration::from(*c.get_current_value()) <= MAX_DURATION);
    c.edit_prev(); // Hours
    c.edit_up(); // +1h
    assert!(Duration::from(*c.get_current_value()) <= MAX_DURATION);
    c.edit_prev(); // Minutes
    c.edit_up(); // +1m
    assert!(Duration::from(*c.get_current_value()) <= MAX_DURATION);
    c.edit_prev(); // Sec.
    c.edit_up(); // +1s
    c.edit_up(); // +1s
    c.edit_up(); // +1s
    assert!(Duration::from(*c.get_current_value()) <= MAX_DURATION);
}

#[test]
fn test_edit_down_years_to_days() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_YEAR + ONE_DAY,
        current_value: ONE_YEAR + ONE_DAY,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    c.toggle_edit();
    c.edit_next(); // Hours
    c.edit_next(); // Days
    c.edit_next(); // Years
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Years, _)));
    c.edit_down();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Days, _)));
}

#[test]
fn test_edit_down_days_to_hours() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_DAY + ONE_HOUR,
        current_value: ONE_DAY + ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    c.toggle_edit();
    c.edit_next(); // Hours
    c.edit_next(); // Days
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Days, _)));
    c.edit_down();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
}

#[test]
fn test_edit_down_hours_to_minutes() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_HOUR + ONE_MINUTE,
        current_value: ONE_HOUR + ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    c.toggle_edit();
    c.edit_next(); // Hours
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    c.edit_down();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_down_minutes_to_seconds() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_MINUTE,
        current_value: ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
    c.edit_down();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_next_ydddhhmmssd() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_YEAR,
        current_value: ONE_YEAR,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on - should start at Minutes
    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Days, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Years, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_hours_in_dhhmmss_format() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_DAY + ONE_HOUR,
        current_value: ONE_DAY + ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    c.toggle_edit();
    c.edit_next(); // Move to Hours
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));

    // Increment hours - should stay in Hours edit mode
    c.edit_up();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    assert_eq!(
        Duration::from(*c.get_current_value()),
        ONE_DAY + 2 * ONE_HOUR
    );
}

#[test]
fn test_edit_next_ydddhhmmss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_YEAR,
        current_value: ONE_YEAR,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    // toggle on - should start at Minutes
    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Days, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Years, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_next_dhhmmssd() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_DAY,
        current_value: ONE_DAY,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on - should start at Minutes (following existing pattern)
    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Days, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Years, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_next_hhmmssd() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_HOUR,
        current_value: ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Days, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Years, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_next_hhmmss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_HOUR,
        current_value: ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Days, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Years, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_next_mmssd() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_MINUTE,
        current_value: ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_next_mmss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_MINUTE,
        current_value: ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_next_ssd() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_SECOND * 3,
        current_value: ONE_SECOND * 3,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
}

#[test]
fn test_edit_next_sd() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_SECOND,
        current_value: ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_next_ss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_SECOND * 3,
        current_value: ONE_SECOND * 3,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_next_s() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_SECOND,
        current_value: ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_prev_ydddhhmmssd() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_YEAR,
        current_value: ONE_YEAR,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on - should start at Minutes
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Years, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Days, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_prev_ydddhhmmss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_YEAR,
        current_value: ONE_YEAR,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    // toggle on - should start at Minutes
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Years, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Days, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_prev_dhhmmssd() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_DAY,
        current_value: ONE_DAY,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on - should start at Minutes
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Days, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_prev_hhmmssd() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_HOUR,
        current_value: ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
}

#[test]
fn test_edit_prev_hhmmss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_HOUR,
        current_value: ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
}

#[test]
fn test_edit_prev_mmssd() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_MINUTE,
        current_value: ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_prev_mmss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_MINUTE,
        current_value: ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_prev_ssd() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_SECOND,
        current_value: ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_prev_sd() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_SECOND,
        current_value: ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_prev_ss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_SECOND,
        current_value: ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_prev_s() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: ONE_SECOND,
        current_value: ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_up_ss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: Duration::ZERO,
        current_value: Duration::ZERO,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    // +1s
    c.edit_up();
    assert_eq!(Duration::from(*c.get_current_value()), ONE_SECOND);
}

#[test]
fn test_edit_up_mmss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: Duration::ZERO,
        current_value: Duration::from_secs(60),
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    // +1m
    c.edit_up();
    assert_eq!(
        Duration::from(*c.get_current_value()),
        Duration::from_secs(120)
    );
}

#[test]
fn test_edit_up_hhmmss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: Duration::ZERO,
        current_value: Duration::from_secs(3600),
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    // edit hh
    c.edit_next();
    // +1h
    c.edit_up();
    assert_eq!(
        Duration::from(*c.get_current_value()),
        Duration::from_secs(3600 + 3600)
    );
}

#[test]
fn test_edit_down_ss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: Duration::ZERO,
        current_value: ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    // -1s
    c.edit_down();
    assert_eq!(Duration::from(*c.get_current_value()), Duration::ZERO);
    // and again: -1s
    c.edit_down();
    // still ZERO
    assert_eq!(Duration::from(*c.get_current_value()), Duration::ZERO);
}

#[test]
fn test_edit_down_mmss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: Duration::ZERO,
        current_value: Duration::from_secs(120),
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    // -1m
    c.edit_down();
    assert_eq!(
        Duration::from(*c.get_current_value()),
        Duration::from_secs(60)
    );
    // and again: -1m
    c.edit_down();
    assert_eq!(Duration::from(*c.get_current_value()), Duration::ZERO);
}

#[test]
fn test_edit_down_hhmmss() {
    let mut c = ClockState::<Timer>::new(ClockStateArgs {
        initial_value: Duration::ZERO,
        current_value: Duration::from_secs(3600),
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
        app_tx: None,
    });

    // toggle on
    c.toggle_edit();
    // edit hh
    c.edit_next();
    // +1h
    c.edit_down();
    assert_eq!(Duration::from(*c.get_current_value()), Duration::ZERO);
}

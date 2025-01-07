use crate::{
    duration::{ONE_DECI_SECOND, ONE_HOUR, ONE_MINUTE, ONE_SECOND},
    widgets::clock::*,
};
use std::time::Duration;

#[test]
fn test_toggle_edit() {
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_HOUR,
        current_value: ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
    });
    // off by default
    assert!(!c.is_edit_mode());
    // toggle on
    c.toggle_edit();
    assert!(c.is_edit_mode());
    // toggle off
    c.toggle_edit();
    assert!(!c.is_edit_mode());
}

#[test]
fn test_default_edit_mode_hhmmss() {
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_HOUR,
        current_value: ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_default_edit_mode_mmss() {
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_MINUTE,
        current_value: ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
    });
    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_default_edit_mode_ss() {
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_SECOND,
        current_value: ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
    });
    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_next_hhmmssd() {
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_HOUR,
        current_value: ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
    });

    // toggle on
    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_next_hhmmss() {
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_HOUR,
        current_value: ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
    });

    // toggle on
    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Hours, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Minutes, _)));
}

#[test]
fn test_edit_next_mmssd() {
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_MINUTE,
        current_value: ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
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
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_MINUTE,
        current_value: ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
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
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_SECOND * 3,
        current_value: ONE_SECOND * 3,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
    });

    // toggle on
    c.toggle_edit();
    c.edit_next();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Decis, _)));
}

#[test]
fn test_edit_next_ss() {
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_SECOND * 3,
        current_value: ONE_SECOND * 3,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
    });

    // toggle on
    c.toggle_edit();
    c.edit_next();
    println!("mode -> {:?}", c.get_mode());
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_prev_hhmmssd() {
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_HOUR,
        current_value: ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
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
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_HOUR,
        current_value: ONE_HOUR,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
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
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_MINUTE,
        current_value: ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
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
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_MINUTE,
        current_value: ONE_MINUTE,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
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
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_SECOND,
        current_value: ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: true,
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
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: ONE_SECOND,
        current_value: ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
    });

    // toggle on
    c.toggle_edit();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
    c.edit_prev();
    assert!(matches!(c.get_mode(), Mode::Editable(Time::Seconds, _)));
}

#[test]
fn test_edit_up_ss() {
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: Duration::ZERO,
        current_value: Duration::ZERO,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
    });

    // toggle on
    c.toggle_edit();
    // +1s
    c.edit_up();
    assert_eq!(Duration::from(*c.get_current_value()), ONE_SECOND);
}

#[test]
fn test_edit_up_mmss() {
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: Duration::ZERO,
        current_value: Duration::from_secs(60),
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
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
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: Duration::ZERO,
        current_value: Duration::from_secs(3600),
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
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
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: Duration::ZERO,
        current_value: ONE_SECOND,
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
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
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: Duration::ZERO,
        current_value: Duration::from_secs(120),
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
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
    let mut c = Clock::<Timer>::new(ClockArgs {
        initial_value: Duration::ZERO,
        current_value: Duration::from_secs(3600),
        tick_value: ONE_DECI_SECOND,
        with_decis: false,
    });

    // toggle on
    c.toggle_edit();
    // edit hh
    c.edit_next();
    // +1h
    c.edit_down();
    assert_eq!(Duration::from(*c.get_current_value()), Duration::ZERO);
}

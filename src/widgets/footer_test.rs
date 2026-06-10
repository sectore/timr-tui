use crate::{
    common::{AppEditMode, AppTime, AppTimeFormat, Content},
    widgets::{
        footer::{Footer, FooterState},
        test_utils::{AssertSnapshotArgs, FIXED_TIME, assert_snapshot},
    },
};

// create widget with `default` (test) values
fn w() -> Footer {
    Footer {
        running_clock: false,
        selected_content: Content::Countdown,
        app_edit_mode: AppEditMode::None,
        app_time: AppTime::Local(FIXED_TIME),
        pomodoro_auto_switch: false,
    }
}

// create widget state with `default` (test) values
fn st() -> FooterState {
    FooterState::new(
        true,  // show_menu
        None,  // app_time_format
        false, // vim_motions
    )
}

fn assert(w: Footer, st: FooterState) {
    assert_snapshot(AssertSnapshotArgs {
        widget: w,
        state: st,
        width: 120,
        height: 6,
    });
}

#[test]
fn test_menu_hidden() {
    let st = st().with_show_menu(false);
    assert(w(), st);
}

#[test]
fn test_menu_countdown_stopped() {
    assert(w(), st());
}

#[test]
fn test_menu_countdown_running() {
    let w = Footer {
        running_clock: true,
        ..w()
    };
    assert(w, st());
}

#[test]
fn test_menu_pomodoro_auto_switch_off() {
    let w = Footer {
        selected_content: Content::Pomodoro,
        ..w()
    };
    assert(w, st());
}

#[test]
fn test_menu_pomodoro_auto_switch_on() {
    let w = Footer {
        selected_content: Content::Pomodoro,
        pomodoro_auto_switch: true,
        ..w()
    };
    assert(w, st());
}

#[test]
fn test_menu_countdown_edit_mode() {
    let w = Footer {
        app_edit_mode: AppEditMode::Clock,
        ..w()
    };
    assert(w, st());
}

#[test]
fn test_menu_countdown_vim_motions() {
    let st = st().with_vim_motions(true);
    assert(w(), st);
}

#[test]
fn test_menu_time_format_hh_mm_ss() {
    let st = st().with_app_time_format(AppTimeFormat::HhMmSs);
    assert(w(), st);
}

#[test]
fn test_menu_time_format_hh_mm() {
    let st = st().with_app_time_format(AppTimeFormat::HhMm);
    assert(w(), st);
}

#[test]
fn test_menu_time_format_hh_12_mm() {
    let st = st().with_app_time_format(AppTimeFormat::Hh12Mm);
    assert(w(), st);
}

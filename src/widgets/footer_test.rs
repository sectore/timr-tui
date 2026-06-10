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

// countdown

#[test]
fn test_menu_countdown_stopped() {
    assert(w(), st());
}

#[test]
fn test_menu_countdown_running() {
    let w = Footer { running_clock: true, ..w() };
    assert(w, st());
}

#[test]
fn test_menu_countdown_edit_mode() {
    let w = Footer { app_edit_mode: AppEditMode::Clock, ..w() };
    assert(w, st());
}

// timer

#[test]
fn test_menu_timer_stopped() {
    let w = Footer { selected_content: Content::Timer, ..w() };
    assert(w, st());
}

#[test]
fn test_menu_timer_running() {
    let w = Footer { selected_content: Content::Timer, running_clock: true, ..w() };
    assert(w, st());
}

#[test]
fn test_menu_timer_edit_mode() {
    let w = Footer { selected_content: Content::Timer, app_edit_mode: AppEditMode::Clock, ..w() };
    assert(w, st());
}

// pomodoro

#[test]
fn test_menu_pomodoro_auto_switch_off() {
    let w = Footer { selected_content: Content::Pomodoro, ..w() };
    assert(w, st());
}

#[test]
fn test_menu_pomodoro_auto_switch_on() {
    let w = Footer { selected_content: Content::Pomodoro, pomodoro_auto_switch: true, ..w() };
    assert(w, st());
}

#[test]
fn test_menu_pomodoro_edit_mode() {
    let w = Footer { selected_content: Content::Pomodoro, app_edit_mode: AppEditMode::Clock, ..w() };
    assert(w, st());
}

// event

#[test]
fn test_menu_event() {
    let w = Footer { selected_content: Content::Event, ..w() };
    assert(w, st());
}

#[test]
fn test_menu_event_edit_mode() {
    let w = Footer { selected_content: Content::Event, app_edit_mode: AppEditMode::Event, ..w() };
    assert(w, st());
}

// local time

#[test]
fn test_menu_local_time() {
    let w = Footer { selected_content: Content::LocalTime, ..w() };
    assert(w, st());
}

// vim motions

#[test]
fn test_menu_countdown_vim() {
    let st = st().with_vim_motions(true);
    assert(w(), st);
}

#[test]
fn test_menu_countdown_edit_mode_vim() {
    let w = Footer { app_edit_mode: AppEditMode::Clock, ..w() };
    let st = st().with_vim_motions(true);
    assert(w, st);
}

// time formats

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

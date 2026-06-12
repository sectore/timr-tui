use crate::{
    common::{AppTime, AppTimeFormat, Style},
    widgets::{
        local_time::{LocalTimeState, LocalTimeStateArgs, LocalTimeWidget},
        test_utils::{DrawArgs, FIXED_TIME, FIXED_TIME_AM, draw},
    },
};
use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend};

fn args() -> LocalTimeStateArgs {
    LocalTimeStateArgs {
        app_time: AppTime::Utc(FIXED_TIME),
        app_time_format: AppTimeFormat::HhMmSs,
    }
}

fn st_with_args(args: LocalTimeStateArgs) -> LocalTimeState {
    LocalTimeState::new(args)
}

fn w() -> LocalTimeWidget {
    LocalTimeWidget {
        style: Style::default(),
    }
}

fn terminal(w: LocalTimeWidget, st: LocalTimeState) -> Terminal<TestBackend> {
    draw(DrawArgs {
        widget: w,
        state: st,
        width: 70,
        height: 16,
    })
}

#[test]
fn test_local_time_hhmmss() {
    let t = terminal(w(), st_with_args(args()));
    assert_snapshot!("local_time_hhmmss", t.backend());
}

#[test]
fn test_local_time_hhmm() {
    let t = terminal(
        w(),
        st_with_args(LocalTimeStateArgs {
            app_time_format: AppTimeFormat::HhMm,
            ..args()
        }),
    );
    assert_snapshot!("local_time_hhmm", t.backend());
}

#[test]
fn test_local_time_hh12mm_pm() {
    let t = terminal(
        w(),
        st_with_args(LocalTimeStateArgs {
            app_time_format: AppTimeFormat::Hh12Mm,
            ..args()
        }),
    );
    assert_snapshot!("local_time_hh12mm_pm", t.backend());
}

#[test]
fn test_local_time_hh12mm_am() {
    let t = terminal(
        w(),
        st_with_args(LocalTimeStateArgs {
            app_time: AppTime::Utc(FIXED_TIME_AM),
            app_time_format: AppTimeFormat::Hh12Mm,
        }),
    );
    assert_snapshot!("local_time_hh12mm_am", t.backend());
}

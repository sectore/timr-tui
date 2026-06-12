use crate::{
    common::{AppTime, AppTimeFormat, Style},
    event::Event,
    widgets::{
        event::{EventState, EventStateArgs, EventWidget},
        test_utils::{DrawArgs, FIXED_TIME, draw},
    },
};
use insta::assert_snapshot;
use ratatui::{Terminal, backend::TestBackend};
use time::macros::datetime;

fn app_tx() -> crate::events::AppEventTx {
    tokio::sync::mpsc::unbounded_channel().0
}

fn args() -> EventStateArgs {
    EventStateArgs {
        app_time: AppTime::Utc(FIXED_TIME),
        event: Event::default(),
        with_decis: false,
        app_tx: app_tx(),
        event_time_format: None,
    }
}

fn st_with_args(args: EventStateArgs) -> EventState {
    EventState::new(args)
}

fn w() -> EventWidget {
    EventWidget {
        style: Style::default(),
        blink: false,
    }
}

fn terminal(w: EventWidget, st: EventState) -> Terminal<TestBackend> {
    draw(DrawArgs {
        widget: w,
        state: st,
        width: 100,
        height: 16,
    })
}

#[test]
fn test_event_since() {
    let t = terminal(w(), st_with_args(args()));
    assert_snapshot!("event_since", t.backend());
}

#[test]
fn test_event_since_hhmmss() {
    let st = st_with_args(EventStateArgs {
        event_time_format: Some(AppTimeFormat::HhMmSs),
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("event_since_hhmmss", t.backend());
}

#[test]
fn test_event_since_hhmm() {
    let st = st_with_args(EventStateArgs {
        event_time_format: Some(AppTimeFormat::HhMm),
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("event_since_hhmm", t.backend());
}

#[test]
fn test_event_since_hh12mm() {
    let st = st_with_args(EventStateArgs {
        event_time_format: Some(AppTimeFormat::Hh12Mm),
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("event_since_hh12mm", t.backend());
}

#[test]
fn test_event_since_decis() {
    let st = st_with_args(EventStateArgs {
        with_decis: true,
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("event_since_decis", t.backend());
}

#[test]
fn test_event_until() {
    let st = st_with_args(EventStateArgs {
        event: Event {
            date_time: datetime!(2050-01-01 00:00),
            title: Some("hello future".into()),
        },
        ..args()
    });
    let t = terminal(w(), st);
    assert_snapshot!("event_until", t.backend());
}

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEventKind};
use futures::{Stream, StreamExt};
use ratatui::layout::Position;
use std::{pin::Pin, time::Duration};
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio_stream::{StreamMap, wrappers::IntervalStream};

use crate::common::ClockTypeId;
use crate::constants::TICK_VALUE_MS;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum StreamKey {
    Ticks,
    Crossterm,
}

#[derive(Clone, Debug)]
pub enum TuiEvent {
    Error,
    Tick,
    Crossterm(CrosstermEvent),
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    ClockDone(ClockTypeId, String),
    SetCursor(Option<Position>),
}

pub type AppEventTx = mpsc::UnboundedSender<AppEvent>;
pub type AppEventRx = mpsc::UnboundedReceiver<AppEvent>;

pub struct Events {
    streams: StreamMap<StreamKey, Pin<Box<dyn Stream<Item = TuiEvent>>>>,
    app_channel: (AppEventTx, AppEventRx),
}

impl Default for Events {
    fn default() -> Self {
        Self {
            streams: StreamMap::from_iter([
                (StreamKey::Ticks, tick_stream()),
                (StreamKey::Crossterm, crossterm_stream()),
            ]),
            app_channel: mpsc::unbounded_channel(),
        }
    }
}

pub enum Event {
    Terminal(TuiEvent),
    App(AppEvent),
}

impl Events {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn next(&mut self) -> Option<Event> {
        let streams = &mut self.streams;
        let app_rx = &mut self.app_channel.1;
        tokio::select! {
            Some((_, event)) = streams.next() => Some(Event::Terminal(event)),
            Some(app_event) = app_rx.recv() => Some(Event::App(app_event)),
        }
    }

    pub fn get_app_event_tx(&self) -> AppEventTx {
        self.app_channel.0.clone()
    }
}

fn tick_stream() -> Pin<Box<dyn Stream<Item = TuiEvent>>> {
    let tick_interval = interval(Duration::from_millis(TICK_VALUE_MS));
    Box::pin(IntervalStream::new(tick_interval).map(|_| TuiEvent::Tick))
}

fn crossterm_stream() -> Pin<Box<dyn Stream<Item = TuiEvent>>> {
    Box::pin(
        EventStream::new()
            .fuse()
            // we are not interested in all events
            .filter_map(|result| async move {
                match result {
                    // filter `KeyEventKind::Press` out to ignore all the other `CrosstermEvent::Key` events
                    Ok(CrosstermEvent::Key(key)) => (key.kind == KeyEventKind::Press)
                        .then_some(TuiEvent::Crossterm(CrosstermEvent::Key(key))),
                    Ok(other) => Some(TuiEvent::Crossterm(other)),
                    Err(_) => Some(TuiEvent::Error),
                }
            }),
    )
}

pub trait TuiEventHandler {
    fn update(&mut self, _: TuiEvent) -> Option<TuiEvent>;
}

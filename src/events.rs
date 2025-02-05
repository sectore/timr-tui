use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::{Stream, StreamExt};
use std::{pin::Pin, time::Duration};
use tokio::sync::mpsc;
use tokio::time::interval;
use tokio_stream::{wrappers::IntervalStream, StreamMap};

use crate::common::ClockTypeId;
use crate::constants::{FPS_VALUE_MS, TICK_VALUE_MS};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum StreamKey {
    Ticks,
    Render,
    Crossterm,
}

#[derive(Clone, Debug)]
pub enum TuiEvent {
    Error,
    Tick,
    Render,
    Key(KeyEvent),
    Resize,
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    ClockDone(ClockTypeId, String),
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
                (StreamKey::Render, render_stream()),
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

fn render_stream() -> Pin<Box<dyn Stream<Item = TuiEvent>>> {
    let render_interval = interval(Duration::from_millis(FPS_VALUE_MS));
    Box::pin(IntervalStream::new(render_interval).map(|_| TuiEvent::Render))
}

fn crossterm_stream() -> Pin<Box<dyn Stream<Item = TuiEvent>>> {
    Box::pin(
        EventStream::new()
            .fuse()
            // we are not interested in all events
            .filter_map(|event| async move {
                match event {
                    Ok(CrosstermEvent::Key(key)) if key.kind == KeyEventKind::Press => {
                        Some(TuiEvent::Key(key))
                    }
                    Ok(CrosstermEvent::Resize(_, _)) => Some(TuiEvent::Resize),
                    Err(_) => Some(TuiEvent::Error),
                    _ => None,
                }
            }),
    )
}

pub trait TuiEventHandler {
    fn update(&mut self, _: TuiEvent) -> Option<TuiEvent>;
}

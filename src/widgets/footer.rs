use std::collections::BTreeMap;

use crate::common::{AppEditMode, AppTime, AppTimeFormat, Content};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    symbols::{border, scrollbar},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, Widget},
};

#[derive(Debug, Clone)]
pub struct FooterState {
    show_menu: bool,
    app_time_format: AppTimeFormat,
}

impl FooterState {
    pub const fn new(show_menu: bool, app_time_format: AppTimeFormat) -> Self {
        Self {
            show_menu,
            app_time_format,
        }
    }

    pub fn set_show_menu(&mut self, value: bool) {
        self.show_menu = value;
    }

    pub const fn get_show_menu(&self) -> bool {
        self.show_menu
    }

    pub const fn app_time_format(&self) -> &AppTimeFormat {
        &self.app_time_format
    }

    pub fn toggle_app_time_format(&mut self) {
        self.app_time_format = self.app_time_format.next();
    }
}

#[derive(Debug)]
pub struct Footer {
    pub running_clock: bool,
    pub selected_content: Content,
    pub app_edit_mode: AppEditMode,
    pub app_time: AppTime,
}

impl StatefulWidget for Footer {
    type State = FooterState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let content_labels: BTreeMap<Content, &str> = BTreeMap::from([
            (Content::Countdown, "[c]ountdown"),
            (Content::Timer, "[t]imer"),
            (Content::Pomodoro, "[p]omodoro"),
        ]);

        let [_, area] =
            Layout::horizontal([Constraint::Length(1), Constraint::Percentage(100)]).areas(area);

        let [border_area, menu_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)]).areas(area);

        Block::new()
            .borders(Borders::TOP)
            .title(
                format! {"[m]enu {:} ", if state.show_menu {scrollbar::VERTICAL.end} else {scrollbar::VERTICAL.begin}},
            )
            .title(
                Line::from(
                    match state.app_time_format {
                        // `Hidden` -> no (empty) title
                        AppTimeFormat::Hidden => "".into(),
                        // others -> add some space around
                        _ => format!(" {} ", self.app_time.format(&state.app_time_format))
                    }
                ).right_aligned())
            .border_set(border::PLAIN)
            .render(border_area, buf);
        // show menu
        if state.show_menu {
            let content_labels: Vec<Span> = content_labels
                .iter()
                .enumerate()
                .map(|(index, (content, label))| {
                    let mut style = Style::default();
                    // Add space for all except last
                    let label = if index < content_labels.len() - 1 {
                        format!("{label}  ")
                    } else {
                        label.to_string()
                    };
                    if *content == self.selected_content {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    Span::styled(label, style)
                })
                .collect();

            const SPACE: &str = "  "; // 2 empty spaces
            let widths = [Constraint::Length(12), Constraint::Percentage(100)];
            let table = Table::new(
                [
                    // screens
                    Row::new(vec![
                        Cell::from(Span::styled(
                            "screens",
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Line::from(content_labels)),
                    ]),
                    // appearance
                    Row::new(vec![
                        Cell::from(Span::styled(
                            "appearance",
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Line::from(vec![
                            Span::from("[,]change style"),
                            Span::from(SPACE),
                            Span::from("[.]toggle deciseconds"),
                            Span::from(SPACE),
                            Span::from(format!(
                                "[:]toggle {} time",
                                match self.app_time {
                                    AppTime::Local(_) => "local",
                                    AppTime::Utc(_) => "utc",
                                }
                            )),
                        ])),
                    ]),
                    // controls - 1. row
                    Row::new(vec![
                        Cell::from(Span::styled(
                            "controls",
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Line::from({
                            match self.app_edit_mode {
                                AppEditMode::None => {
                                    let mut spans = vec![Span::from(if self.running_clock {
                                        "[s]top"
                                    } else {
                                        "[s]tart"
                                    })];
                                    spans.extend_from_slice(&[
                                        Span::from(SPACE),
                                        Span::from("[e]dit"),
                                    ]);
                                    if self.selected_content == Content::Countdown {
                                        spans.extend_from_slice(&[
                                            Span::from(SPACE),
                                            Span::from("[^e]dit by local time"),
                                        ]);
                                    }
                                    spans.extend_from_slice(&[
                                        Span::from(SPACE),
                                        Span::from("[r]eset clock"),
                                    ]);
                                    if self.selected_content == Content::Pomodoro {
                                        spans.extend_from_slice(&[
                                            Span::from(SPACE),
                                            Span::from("[^r]eset clocks+rounds"),
                                        ]);
                                    }
                                    spans
                                }
                                _ => {
                                    let mut spans = vec![Span::from("[s]ave changes")];
                                    if self.selected_content == Content::Countdown
                                        || self.selected_content == Content::Pomodoro
                                    {
                                        spans.extend_from_slice(&[
                                            Span::from(SPACE),
                                            Span::from("[^s]ave initial value"),
                                        ]);
                                    }
                                    spans.extend_from_slice(&[
                                        Span::from(SPACE),
                                        Span::from("[esc]skip changes"),
                                    ]);
                                    spans
                                }
                            }
                        })),
                    ]),
                    // controls - 2. row
                    Row::new(vec![
                        Cell::from(Line::from("")),
                        Cell::from(Line::from({
                            match self.app_edit_mode {
                                AppEditMode::None => {
                                    let mut spans = vec![];
                                    if self.selected_content == Content::Pomodoro {
                                        spans.extend_from_slice(&[Span::from(
                                            "[← →]switch work/pause",
                                        )]);
                                    }
                                    spans
                                }
                                _ => vec![
                                    Span::from(format!(
                                        // ← →,
                                        "[{} {}]change selection",
                                        scrollbar::HORIZONTAL.begin,
                                        scrollbar::HORIZONTAL.end
                                    )),
                                    Span::from(SPACE),
                                    Span::from(format!(
                                        // ↑
                                        "[{}]edit up",
                                        scrollbar::VERTICAL.begin
                                    )),
                                    Span::from(SPACE),
                                    Span::from(format!(
                                        // ↓
                                        "[{}]edit up",
                                        scrollbar::VERTICAL.end
                                    )),
                                ],
                            }
                        })),
                    ]),
                ],
                widths,
            )
            .column_spacing(1);

            Widget::render(table, menu_area, buf);
        }
    }
}

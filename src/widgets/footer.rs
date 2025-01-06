use std::collections::BTreeMap;

use crate::common::{Content, LocalTimeFormat};
use chrono::{DateTime, Local};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    symbols::{border, scrollbar},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, Widget},
};

#[derive(Debug, Clone, Default)]
pub struct FooterState {
    show_menu: bool,
    local_time: DateTime<Local>,
    local_time_format: LocalTimeFormat,
}

impl FooterState {
    pub const fn new(
        show_menu: bool,
        local_time: DateTime<Local>,
        local_time_format: LocalTimeFormat,
    ) -> Self {
        Self {
            show_menu,
            local_time,
            local_time_format,
        }
    }

    pub fn set_show_menu(&mut self, value: bool) {
        self.show_menu = value;
    }

    pub const fn get_show_menu(&self) -> bool {
        self.show_menu
    }

    pub fn set_local_time(&mut self, value: DateTime<Local>) {
        self.local_time = value;
    }

    pub const fn get_local_time_format(&self) -> &LocalTimeFormat {
        &self.local_time_format
    }

    pub fn toggle_local_time_format(&mut self) {
        self.local_time_format = self.local_time_format.next();
    }
}

#[derive(Debug, Clone)]
pub struct Footer {
    pub running_clock: bool,
    pub selected_content: Content,
    pub edit_mode: bool,
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

        let local_time_str = match state.local_time_format {
            LocalTimeFormat::Empty => "".into(),
            _ => format!(
                " {}", // empty space to add padding after border
                state.local_time.format(state.local_time_format.fmt())
            ),
        };

        Block::new()
            .borders(Borders::TOP)
            .title(
                format! {"[m]enu {:} ", if state.show_menu {scrollbar::VERTICAL.end} else {scrollbar::VERTICAL.begin}},
            )
            .title(
                Line::from(local_time_str).right_aligned())
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
                        format!("{}  ", label)
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
                    // content
                    Row::new(vec![
                        Cell::from(Span::styled(
                            "screens",
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Line::from(content_labels)),
                    ]),
                    // format
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
                            Span::from("[l]ocal time"),
                        ])),
                    ]),
                    // edit
                    Row::new(vec![
                        Cell::from(Span::styled(
                            "controls",
                            Style::default().add_modifier(Modifier::BOLD),
                        )),
                        Cell::from(Line::from({
                            if self.edit_mode {
                                vec![
                                    Span::from("[e]dit done"),
                                    Span::from(SPACE),
                                    Span::from(format!(
                                        "[{} {}]edit selection",
                                        scrollbar::HORIZONTAL.begin,
                                        scrollbar::HORIZONTAL.end
                                    )), // ← →,
                                    Span::from(SPACE),
                                    Span::from(format!("[{}]edit up", scrollbar::VERTICAL.begin)), // ↑
                                    Span::from(SPACE),
                                    Span::from(format!("[{}]edit up", scrollbar::VERTICAL.end)), // ↓,
                                ]
                            } else {
                                let mut spans = vec![
                                    Span::from(if self.running_clock {
                                        "[s]top"
                                    } else {
                                        "[s]tart"
                                    }),
                                    Span::from(SPACE),
                                    Span::from("[r]eset"),
                                    Span::from(SPACE),
                                    Span::from("[e]dit"),
                                ];
                                if self.selected_content == Content::Pomodoro {
                                    spans.extend_from_slice(&[
                                        Span::from(SPACE),
                                        Span::from("[← →]switch work/pause"),
                                    ]);
                                }
                                spans
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

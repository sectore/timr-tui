use std::collections::BTreeMap;

use crate::common::{AppEditMode, AppTime, AppTimeFormat, Content};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::Style,
    symbols::{border, scrollbar},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, StatefulWidget, Table, Widget},
};

#[derive(Debug, Clone)]
pub struct FooterState {
    show_menu: bool,
    app_time_format: Option<AppTimeFormat>,
}

impl FooterState {
    pub const fn new(show_menu: bool, app_time_format: Option<AppTimeFormat>) -> Self {
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

    pub const fn app_time_format(&self) -> &Option<AppTimeFormat> {
        &self.app_time_format
    }

    pub const fn set_app_time_format(&mut self, value: Option<AppTimeFormat>) {
        self.app_time_format = value;
    }
}

#[derive(Debug)]
pub struct Footer {
    pub running_clock: bool,
    pub selected_content: Content,
    pub app_edit_mode: AppEditMode,
    pub app_time: AppTime,
}

const SPACE: &str = " "; // single (empty) SPACE
const WIDE_SPACE: &str = "   "; // three (empty) SPACEs
const BOLD: Style = Style::new().bold();
const ITALIC: Style = Style::new().italic();

impl StatefulWidget for Footer {
    type State = FooterState;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let content_labels: BTreeMap<Content, &str> = BTreeMap::from([
            (Content::Countdown, "countdown"),
            (Content::Timer, "timer"),
            (Content::Pomodoro, "pomodoro"),
            (Content::Event, "event"),
            (Content::LocalTime, "local time"),
        ]);

        let [_, area] =
            Layout::horizontal([Constraint::Length(1), Constraint::Percentage(100)]).areas(area);

        let [border_area, menu_area] =
            Layout::vertical([Constraint::Length(1), Constraint::Percentage(100)]).areas(area);

        Block::new()
            .borders(Borders::TOP)
            .title(Line::from(vec![
                Span::styled(
                    if state.show_menu {
                        scrollbar::VERTICAL.end
                    } else {
                        scrollbar::VERTICAL.begin
                    },
                    BOLD,
                ),
                Span::from(SPACE),
                Span::from("menu"),
                Span::from(SPACE),
            ]))
            .title(
                Line::from(match (state.app_time_format, self.selected_content) {
                    // Show time
                    (Some(v), content) if content != Content::LocalTime => format!(
                        "{SPACE}{}{SPACE}", // keep SPACE around
                        self.app_time.format(&v)
                    ),
                    // Hide time -> empty string
                    _ => "".into(),
                })
                .right_aligned(),
            )
            .border_set(border::PLAIN)
            .render(border_area, buf);
        // show menu
        if state.show_menu {
            let mut content_labels: Vec<Span> = content_labels
                .iter()
                .enumerate()
                .flat_map(|(index, (content, label))| {
                    let no = index + 1;
                    let is_last = index == content_labels.len() - 1;
                    let is_selected = *content == self.selected_content;
                    let label_text = if is_last {
                        label.to_string()
                    } else {
                        format!("{label}{WIDE_SPACE}")
                    };
                    [
                        Span::styled(format!("{no}"), BOLD),
                        Span::from(SPACE),
                        Span::styled(label_text, if is_selected { BOLD.italic() } else { ITALIC }),
                    ]
                })
                .collect();

            content_labels.extend_from_slice(&[
                Span::from(WIDE_SPACE),
                Span::styled("← →", BOLD),
                Span::from(SPACE),
                Span::styled("switch screens", ITALIC),
            ]);

            let widths = [Constraint::Length(12), Constraint::Percentage(100)];
            let mut table_rows = vec![
                // screens
                Row::new(vec![
                    Cell::from(Span::from("screens")),
                    Cell::from(Line::from(content_labels)),
                ]),
                // appearance
                Row::new(vec![
                    Cell::from(Span::from("appearance")),
                    Cell::from(Line::from(vec![
                        Span::styled(",", BOLD),
                        Span::from(SPACE),
                        Span::styled("change style", ITALIC),
                        Span::from(WIDE_SPACE),
                        Span::styled(".", BOLD),
                        Span::from(SPACE),
                        Span::styled("toggle deciseconds", ITALIC),
                        Span::from(WIDE_SPACE),
                        Span::styled(":", BOLD),
                        Span::from(SPACE),
                        Span::styled(
                            format!(
                                "toggle {} time",
                                match self.app_time {
                                    AppTime::Local(_) => "local",
                                    AppTime::Utc(_) => "utc",
                                }
                            ),
                            ITALIC,
                        ),
                    ])),
                ]),
            ];

            // Controls (except for `localtime`)
            if self.selected_content != Content::LocalTime {
                table_rows.extend_from_slice(&[
                    // controls - 1. row
                    Row::new(vec![
                        Cell::from(Span::from("controls")),
                        Cell::from(Line::from({
                            match self.app_edit_mode {
                                AppEditMode::None if self.selected_content != Content::Event => {
                                    let mut spans = vec![
                                        Span::styled("␣", BOLD),
                                        Span::from(SPACE),
                                        Span::styled(
                                            if self.running_clock { "stop" } else { "start" },
                                            ITALIC,
                                        ),
                                    ];
                                    spans.extend_from_slice(&[
                                        Span::from(WIDE_SPACE),
                                        Span::styled("e", BOLD),
                                        Span::from(SPACE),
                                        Span::styled("edit", ITALIC),
                                    ]);
                                    if self.selected_content == Content::Countdown {
                                        spans.extend_from_slice(&[
                                            Span::from(WIDE_SPACE),
                                            Span::styled("⌃e", BOLD),
                                            Span::from(SPACE),
                                            Span::styled("edit by local time", ITALIC),
                                        ]);
                                    }
                                    spans.extend_from_slice(&[
                                        Span::from(WIDE_SPACE),
                                        Span::styled("r", BOLD),
                                        Span::from(SPACE),
                                        Span::styled("reset clock", ITALIC),
                                    ]);
                                    if self.selected_content == Content::Pomodoro {
                                        spans.extend_from_slice(&[
                                            Span::from(WIDE_SPACE),
                                            Span::styled("⌃r", BOLD),
                                            Span::from(SPACE),
                                            Span::styled("reset clocks/rounds", ITALIC),
                                        ]);
                                    }
                                    spans
                                }
                                AppEditMode::None if self.selected_content == Content::Event => {
                                    vec![
                                        Span::styled("e", BOLD),
                                        Span::from(SPACE),
                                        Span::styled("edit", ITALIC),
                                    ]
                                }
                                AppEditMode::Clock | AppEditMode::Time | AppEditMode::Event => {
                                    let mut spans = vec![
                                        Span::styled("s", BOLD),
                                        Span::from(SPACE),
                                        Span::styled("save changes", ITALIC),
                                    ];

                                    if self.selected_content == Content::Event {
                                        spans[0] = Span::styled("↵", BOLD);
                                    };

                                    if self.selected_content == Content::Countdown
                                        || self.selected_content == Content::Pomodoro
                                    {
                                        spans.extend_from_slice(&[
                                            Span::from(WIDE_SPACE),
                                            Span::styled("⌃s", BOLD),
                                            Span::from(SPACE),
                                            Span::styled("save initial value", ITALIC),
                                        ]);
                                    }
                                    spans.extend_from_slice(&[
                                        Span::from(WIDE_SPACE),
                                        Span::styled("esc", BOLD),
                                        Span::from(SPACE),
                                        Span::styled("skip changes", ITALIC),
                                    ]);

                                    if self.selected_content == Content::Event {
                                        spans.extend_from_slice(&[
                                            Span::from(WIDE_SPACE),
                                            Span::styled("tab", BOLD),
                                            Span::from(SPACE),
                                            Span::styled("switch input", ITALIC),
                                        ]);
                                    }
                                    spans
                                }
                                _ => vec![],
                            }
                        })),
                    ]),
                    // controls - 2. row
                    Row::new(if self.selected_content == Content::Event {
                        vec![]
                    } else {
                        vec![
                            Cell::from(Line::from("")),
                            Cell::from(Line::from({
                                match self.app_edit_mode {
                                    AppEditMode::None => {
                                        let mut spans = vec![];
                                        if self.selected_content == Content::Pomodoro {
                                            spans.extend_from_slice(&[
                                                Span::styled("⌃← ⌃→", BOLD),
                                                Span::from(SPACE),
                                                Span::styled("switch work/pause", ITALIC),
                                            ]);
                                        }
                                        spans
                                    }
                                    _ => vec![
                                        Span::styled(scrollbar::HORIZONTAL.begin, BOLD),
                                        Span::from(SPACE),
                                        Span::styled(scrollbar::HORIZONTAL.end, BOLD),
                                        Span::from(SPACE),
                                        Span::styled("change selection", ITALIC),
                                        Span::from(WIDE_SPACE),
                                        Span::styled(scrollbar::VERTICAL.begin, BOLD),
                                        Span::from(SPACE),
                                        Span::styled("edit up", ITALIC),
                                        Span::from(WIDE_SPACE),
                                        Span::styled(
                                            format!("⌃{}", scrollbar::VERTICAL.begin),
                                            BOLD,
                                        ),
                                        Span::from(SPACE),
                                        Span::styled("edit up 10x", ITALIC),
                                        Span::from(WIDE_SPACE),
                                        Span::styled(scrollbar::VERTICAL.end, BOLD),
                                        Span::from(SPACE),
                                        Span::styled("edit down", ITALIC),
                                        Span::from(WIDE_SPACE),
                                        Span::styled(format!("⌃{}", scrollbar::VERTICAL.end), BOLD),
                                        Span::from(SPACE),
                                        Span::styled("edit down 10x", ITALIC),
                                    ],
                                }
                            })),
                        ]
                    }),
                ])
            }

            let table = Table::new(table_rows, widths).column_spacing(1);

            Widget::render(table, menu_area, buf);
        }
    }
}

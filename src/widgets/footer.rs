use std::collections::BTreeMap;

use crate::app::Content;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Row, Table, Widget},
};

#[derive(Debug, Clone)]
pub struct Footer {
    show_menu: bool,
    running_clock: bool,
    selected_content: Content,
    content_labels: BTreeMap<Content, String>,
    edit_mode: bool,
}

pub struct FooterArgs {
    pub show_menu: bool,
    pub running_clock: bool,
    pub selected_content: Content,
    pub edit_mode: bool,
}

impl Footer {
    pub fn new(args: FooterArgs) -> Self {
        let FooterArgs {
            show_menu,
            running_clock,
            selected_content,
            edit_mode,
        } = args;
        Self {
            show_menu,
            running_clock,
            selected_content,
            edit_mode,
            content_labels: BTreeMap::from([
                (Content::Countdown, "[c]ountdown".into()),
                (Content::Timer, "[t]imer".into()),
                (Content::Pomodoro, "[p]omodoro".into()),
            ]),
        }
    }
}

impl Widget for Footer {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [border_area, menu_area] =
            Layout::vertical([Constraint::Length(2), Constraint::Percentage(100)]).areas(area);
        Block::new()
            .borders(Borders::TOP)
            .title(format! {"[m]enu {:} ", if self.show_menu {"↓"} else {"↑"}})
            .border_set(symbols::border::DOUBLE)
            .render(border_area, buf);
        // show menu
        if self.show_menu {
            let content_labels: Vec<Span> = self
                .content_labels
                .iter()
                .enumerate()
                .map(|(index, (content, label))| {
                    let mut style = Style::default();
                    // Add space for all except last
                    let label = if index < self.content_labels.len() - 1 {
                        format!("{}  ", label)
                    } else {
                        label.into()
                    };
                    if *content == self.selected_content {
                        style = style.add_modifier(Modifier::BOLD);
                    }
                    Span::styled(label, style)
                })
                .collect();

            const SPACE: &str = "  ";
            let widths = [Constraint::Length(12), Constraint::Percentage(100)];
            Table::new(
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
                                    Span::from("[← →]edit selection"),
                                    Span::from(SPACE),
                                    Span::from("[↑]edit up"),
                                    Span::from(SPACE),
                                    Span::from("[↓]edit down"),
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
            .render(menu_area, buf);
        }
    }
}

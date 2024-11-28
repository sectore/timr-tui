use std::collections::BTreeMap;

use crate::app::Content;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

#[derive(Debug, Clone)]
pub struct Footer {
    show_menu: bool,
    selected_content: Content,
    content_labels: BTreeMap<Content, String>,
}

impl Footer {
    pub fn new(show_menu: bool, selected_content: Content) -> Self {
        Self {
            show_menu,
            selected_content,
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
            Layout::vertical([Constraint::Length(1), Constraint::Fill(0)]).areas(area);
        Block::new()
            .borders(Borders::TOP)
            .title(format! {"[m]enu {:} ", if self.show_menu {"↓"} else {"↑"}})
            .border_set(symbols::border::DOUBLE)
            .render(border_area, buf);
        // show menu
        if self.show_menu {
            let [title_area, labels_area] =
                Layout::horizontal([Constraint::Length(12), Constraint::Fill(0)]).areas(menu_area);

            Span::styled("screens", Style::default().add_modifier(Modifier::BOLD))
                .render(title_area, buf);
            let spans: Vec<Span> = self
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
            Line::from(spans).render(labels_area, buf);
        }
    }
}

use ratatui::layout::{Constraint, Flex, Layout, Rect};

/// Helper to center an area by given `Constraint`'s
/// based on [Center a Rect](https://ratatui.rs/recipes/layout/center-a-rect)
pub fn center(base_area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(base_area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

pub fn format_ms(ms: u128, show_tenths: bool) -> String {
    // let hours = ms / 3600000;
    let minutes = (ms % 3600000) / 60000;
    let seconds = (ms % 60000) / 1000;
    let tenths = (ms % 1000) / 100;

    if show_tenths {
        format!("{:02}:{:02}.{}", minutes, seconds, tenths)
    } else {
        format!("{:02}:{:02}", minutes, seconds)
    }
}

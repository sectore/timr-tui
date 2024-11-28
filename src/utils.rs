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

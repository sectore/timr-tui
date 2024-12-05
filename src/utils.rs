use ratatui::layout::{Constraint, Flex, Layout, Rect};

/// Helper to center an area horizontally by given `Constraint`
/// based on [Center a Rect](https://ratatui.rs/recipes/layout/center-a-rect)
pub fn center_horizontal(base_area: Rect, horizontal: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(base_area);
    area
}

/// Helper to center an area vertically by given `Constraint`
/// based on [Center a Rect](https://ratatui.rs/recipes/layout/center-a-rect)
pub fn center_vertical(base_area: Rect, vertical: Constraint) -> Rect {
    let [area] = Layout::vertical([vertical])
        .flex(Flex::Center)
        .areas(base_area);
    area
}
/// Helper to center an area by given `Constraint`'s
/// based on [Center a Rect](https://ratatui.rs/recipes/layout/center-a-rect)
pub fn center(base_area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let area = center_horizontal(base_area, horizontal);
    center_vertical(area, vertical)
}

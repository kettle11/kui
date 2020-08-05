use super::interpolation::*;
use crate::ui::{ElementHandle, UIBuilder, Widget};

pub const DEFAULT_COLOR: (f32, f32, f32, f32) = (0.8, 0.8, 0.8, 1.0);

pub struct HorizontalDivider {
    first_section_height: f32,
    element: Option<ElementHandle>,
    handle: Option<ElementHandle>,
    handle_dragging: bool,
}

impl HorizontalDivider {
    fn new() -> Self {
        Self {
            first_section_height: 200.,
            element: None,
            handle: None,
            handle_dragging: false,
        }
    }

    /// Returns true if pressed
    fn build<'a>(&mut self, parent: &UIBuilder<'a>) -> (UIBuilder<'a>, UIBuilder<'a>) {
        // Input
        if let Some(element) = self.handle {
            let pointer_in_element = parent.pointer_in_element(element);
            if parent.pointer_down() && pointer_in_element {
                self.handle_dragging = true;
            }

            if parent.pointer_up() {
                self.handle_dragging = false;
            }

            if self.handle_dragging {
                let element_rectangle = parent.element_rectangle(self.element.unwrap());
                let pointer_position = parent.pointer_position();
                self.first_section_height = pointer_position.1 - element_rectangle.y;
            }
        }

        // Rendering
        let column = parent.column();
        let first_section = column.height(self.first_section_height);
        let top = column.height(10.).horizontal_expander().fill(DEFAULT_COLOR);
        let second_section = column.expander();
        self.handle = Some(top.handle());
        self.element = Some(column.handle());
        (first_section, second_section)
    }
}

impl Widget for HorizontalDivider {}

/// Returns the two sections divided by the divider
pub fn horizontal_divider<'a>(parent: &UIBuilder<'a>, id: u64) -> (UIBuilder<'a>, UIBuilder<'a>) {
    let mut item = parent
        .get_widget(id)
        .1
        .unwrap_or(Box::new(HorizontalDivider::new()));
    let sections = item.build(parent);
    parent.add_widget(id, item);
    sections
}

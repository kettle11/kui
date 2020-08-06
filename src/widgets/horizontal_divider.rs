use super::drag::Drag;
use crate::ui::{UIBuilder, Widget};

pub const DEFAULT_COLOR: (f32, f32, f32, f32) = (0.8, 0.8, 0.8, 1.0);

pub struct HorizontalDivider {
    first_section_height: f32,
    drag: Drag,
}

impl HorizontalDivider {
    fn new() -> Self {
        Self {
            first_section_height: 200.,
            drag: Drag::new((0., 200.)),
        }
    }

    /// Returns true if pressed
    fn build<'a>(&mut self, parent: &UIBuilder<'a>) -> (UIBuilder<'a>, UIBuilder<'a>) {
        // Move the handle
        self.first_section_height = self.drag.update(parent).1;

        // Rendering
        let column = parent.column();
        let first_section = column.height(self.first_section_height);
        let top = column.height(10.).horizontal_expander().fill(DEFAULT_COLOR);
        let second_section = column.expander();

        self.drag.root_and_element = Some((column.handle(), top.handle()));

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

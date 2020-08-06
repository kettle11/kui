use super::drag::Drag;
use crate::ui::{UIBuilder, Widget};

pub const DEFAULT_COLOR: (f32, f32, f32, f32) = (0.25, 0.25, 0.25, 1.0);

pub struct VerticalDivider {
    first_section_width: f32,
    drag: Drag,
}

impl VerticalDivider {
    fn new() -> Self {
        Self {
            first_section_width: 400.,
            drag: Drag::new((400., 0.)),
        }
    }

    /// Returns true if pressed
    fn build<'a>(&mut self, parent: &UIBuilder<'a>) -> (UIBuilder<'a>, UIBuilder<'a>) {
        // Move the handle
        self.first_section_width = self.drag.update(parent).0;

        // Rendering
        let row = parent.row();
        let first_section = row.width(self.first_section_width);
        let handle = row.fit();
        let handle_inner = handle.padding_horizontal(20.).vertical_expander();
        handle_inner.width(2.).fill(DEFAULT_COLOR);
        let second_section = row.expander();

        self.drag.root_and_element = Some((row.handle(), handle.handle()));

        (first_section, second_section)
    }
}

impl Widget for VerticalDivider {}

/// Returns the two sections divided by the divider
pub fn vertical_divider<'a>(parent: &UIBuilder<'a>, id: u64) -> (UIBuilder<'a>, UIBuilder<'a>) {
    let mut item = parent
        .get_widget(id)
        .1
        .unwrap_or(Box::new(VerticalDivider::new()));
    let sections = item.build(parent);
    parent.add_widget(id, item);
    sections
}

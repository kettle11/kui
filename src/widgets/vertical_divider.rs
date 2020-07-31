use super::interpolation::*;
use crate::ui::{ElementHandle, UIBuilder, UIEvent, UI};
use crate::widget::Widget;

pub struct VerticalDivider<A: Widget, B: Widget> {
    pub element_a: A,
    pub element_b: B,
    pointer_down: bool,
    first_column_width: f32,
    element: Option<ElementHandle>,
    handle: Option<ElementHandle>,
}

impl<A: Widget, B: Widget> VerticalDivider<A, B> {
    pub fn new(element_a: A, element_b: B, first_column_width: f32) -> Self {
        Self {
            element_a,
            element_b,
            pointer_down: false,
            element: None,
            handle: None,
            first_column_width,
        }
    }
}

impl<A: Widget, B: Widget> Widget for VerticalDivider<A, B> {
    fn build(&mut self, parent: &UIBuilder) {
        let handle_width = 4.;
        let handle_padding = 30.;

        let row = parent.row();
        self.element_a.build(&row.width(self.first_column_width));
        let handle = row
            .width(handle_padding * 2. + handle_width)
            .padding_horizontal(handle_padding);
        handle.width(handle_width).fill((0.0, 0.0, 0.0, 1.0));
        self.element_b.build(&row);
        self.handle = Some(handle.handle());
        self.element = Some(row.handle());
    }

    fn event(&mut self, ui: &mut UI, event: UIEvent) {
        match event {
            UIEvent::PointerDown => {
                if ui.pointer_in_element(self.handle.unwrap()) {
                    self.pointer_down = true
                }
            }
            UIEvent::PointerUp => {
                self.pointer_down = false;
            }
            UIEvent::PointerMoved => {
                if self.pointer_down {
                    let slider_rectangle = ui.element_rectangle(self.element.unwrap());
                    let pointer_position = ui.pointer_position();
                    let x_difference = pointer_position.0 - slider_rectangle.x;
                    self.first_column_width = x_difference
                }
            }
            _ => {}
        }
        self.element_a.event(ui, event);
        self.element_b.event(ui, event);
    }
}

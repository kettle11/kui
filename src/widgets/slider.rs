use super::interpolation::*;
use crate::ui::{ElementHandle, UIBuilder, UIEvent, UI};
use crate::widget::Widget;

pub const COLOR_DEPRESSED: (f32, f32, f32, f32) = (0.3, 0.3, 0.3, 1.0);
pub const DEFAULT_COLOR: (f32, f32, f32, f32) = (0.8, 0.0, 0.8, 1.0);
pub const HOVER_COLOR: (f32, f32, f32, f32) = (0.7, 0.7, 0.7, 1.0);

pub struct Slider {
    dragging_handle: bool,
    pressed: bool,
    handle_position: f32,
    handle: Option<ElementHandle>,
    element: Option<ElementHandle>,
}

impl Slider {
    pub fn new() -> Self {
        Self {
            // Handle position is from 0.0 to 1.0
            handle_position: 0.0,
            dragging_handle: false,
            pressed: false,
            handle: None,
            element: None,
        }
    }

    /// This function resets when it is called.
    pub fn pressed(&mut self) -> bool {
        let b = self.pressed;
        self.pressed = false;
        b
    }
}

impl Widget for Slider {
    fn build(&mut self, parent: &UIBuilder) {
        // Just using the  parent here is probably incorrect.
        let top = parent;
        let body = top.expander().padding(20.).height(20.).fill(DEFAULT_COLOR);

        let handle_size = 40.;
        let handle = body
            .center_vertical()
            .position_horizontal_percentage(self.handle_position)
            .position_horizontal_pixels(-handle_size / 2.)
            .width(handle_size)
            .height(handle_size)
            .fill((0.0, 0.0, 1.0, 1.0));

        self.handle = Some(handle.handle());
        self.element = Some(top.handle());
    }

    fn event(&mut self, ui: &mut UI, event: UIEvent) {
        match event {
            UIEvent::PointerDown => {
                if ui.pointer_in_element(self.element.unwrap()) {
                    let slider_rectangle = ui.element_rectangle(self.element.unwrap());
                    let pointer_position = ui.pointer_position();
                    let x_difference = ((pointer_position.0 - slider_rectangle.x)
                        / slider_rectangle.width)
                        .min(1.0)
                        .max(0.0);
                    self.handle_position = x_difference;
                    self.dragging_handle = true;
                }
            }
            UIEvent::PointerUp => self.dragging_handle = false,
            UIEvent::PointerMoved => {
                if self.dragging_handle {
                    let slider_rectangle = ui.element_rectangle(self.element.unwrap());
                    let pointer_position = ui.pointer_position();
                    let x_difference = ((pointer_position.0 - slider_rectangle.x)
                        / slider_rectangle.width)
                        .min(1.0)
                        .max(0.0);
                    self.handle_position = x_difference;
                    println!("X Difference: {:?}", x_difference);
                }
            }
            _ => {}
        }
    }
}

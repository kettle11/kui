use crate::ui::{ElementHandle, UIBuilder, UIEvent, UI};
use crate::widget::Widget;

pub const HANDLE_COLOR: (f32, f32, f32, f32) = (0.32, 0.32, 0.32, 1.0);
pub const SLIDER_COLOR: (f32, f32, f32, f32) = (0.74, 0.74, 0.74, 1.0);

pub struct Slider {
    dragging_handle: bool,
    pressed: bool,
    handle_position: f32,
    handle: Option<ElementHandle>,
    bar: Option<ElementHandle>,
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
            bar: None,
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
        let top = parent.fit();
        let bar = top
            .horizontal_expander()
            .padding(20.)
            .height(20.)
            .rounded_fill(SLIDER_COLOR, 10.);

        let _filled_bar = bar
            .width(self.handle_position * 400.)
            .height(20.)
            .rounded_fill((0.0, 0.0, 1.0, 1.0), 10.);

        let handle_size = 40.;
        let handle = bar
            .center_vertical()
            .position_horizontal_percentage(self.handle_position)
            .position_horizontal_pixels(-handle_size / 2.)
            .width(handle_size)
            .height(handle_size)
            .rounded_fill(HANDLE_COLOR, handle_size / 2.);

        self.bar = Some(bar.handle());
        self.handle = Some(handle.handle());
        self.element = Some(top.handle());
    }

    fn event(&mut self, ui: &mut UI, event: UIEvent) {
        match event {
            UIEvent::PointerDown => {
                if ui.pointer_in_element(self.element.unwrap()) {
                    let slider_rectangle = ui.element_rectangle(self.bar.unwrap());
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
                    let slider_rectangle = ui.element_rectangle(self.bar.unwrap());
                    let pointer_position = ui.pointer_position();
                    let x_difference = ((pointer_position.0 - slider_rectangle.x)
                        / slider_rectangle.width)
                        .min(1.0)
                        .max(0.0);

                    self.handle_position = x_difference;
                }
            }
            _ => {}
        }
    }
}

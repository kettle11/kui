use super::interpolation::*;
use crate::ui::{ElementHandle, UIBuilder, Widget};

pub const COLOR_DEPRESSED: (f32, f32, f32, f32) = (0.3, 0.3, 0.3, 1.0);
pub const DEFAULT_COLOR: (f32, f32, f32, f32) = (0.8, 0.8, 0.8, 1.0);
pub const HOVER_COLOR: (f32, f32, f32, f32) = (0.7, 0.7, 0.7, 1.0);

pub struct Button {
    /// The button is currently held down.
    pointer_held_down: bool,
    /// 0. to 1. The current blend to the hover color
    hover_animate: Interpolate,
    element: Option<ElementHandle>,
}

impl Button {
    fn new() -> Self {
        Self {
            pointer_held_down: false,
            hover_animate: Interpolate::new(InterpolationCurve::Ease),
            element: None,
        }
    }

    /// Returns true if pressed
    fn build(&mut self, parent: &UIBuilder, text: &'static str) -> bool {
        // Input
        let mut pressed = false;
        let mut pointer_in_element = false;
        if let Some(element) = self.element {
            pointer_in_element = parent.pointer_in_element(element);
            if parent.pointer_down() && pointer_in_element {
                self.pointer_held_down = true;
            }
            // Perform a button press when the mouse is up within the button.
            if self.pointer_held_down && parent.pointer_up() && pointer_in_element {
                pressed = true;
            }

            if parent.pointer_up() {
                self.pointer_held_down = false;
            }
        }

        let depressed = self.pointer_held_down && pointer_in_element;

        // Animation
        /*
        let animation_speed = 100.;

        if self.pointer_inside {
            self.hover_animate.add(delta / animation_speed);
        } else {
            self.hover_animate.subtract(delta / animation_speed);
        }
        */

        // Rendering
        let color = if depressed {
            COLOR_DEPRESSED
        } else {
            interpolate_color(DEFAULT_COLOR, HOVER_COLOR, self.hover_animate.get())
        };

        let top = parent.fit();
        top.rounded_fill(color, 15.)
            .padding(20.)
            .center_vertical()
            .text(text);
        self.element = Some(top.handle());
        pressed
    }
}

impl Widget for Button {}
pub fn button(parent: &UIBuilder, id: u64, text: &'static str) -> bool {
    // Create or get an existing button.
    let mut button = parent.get_widget(id).1.unwrap_or(Box::new(Button::new()));
    let pressed = button.build(parent, text);
    parent.add_widget(id, button);
    pressed
}

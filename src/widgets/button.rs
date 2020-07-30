use super::interpolation::*;
use crate::ui::{ElementHandle, UIBuilder, UIEvent, UI};
use crate::widget::Widget;

pub const COLOR_DEPRESSED: (f32, f32, f32, f32) = (0.3, 0.3, 0.3, 1.0);
pub const DEFAULT_COLOR: (f32, f32, f32, f32) = (0.8, 0.8, 0.8, 1.0);
pub const HOVER_COLOR: (f32, f32, f32, f32) = (0.7, 0.7, 0.7, 1.0);

pub struct Button {
    pub text: String,
    pressed: bool,
    /// The button is currently held down.
    pointer_held_down: bool,
    pointer_inside: bool,
    /// 0. to 1. The current blend to the hover color
    hover_animate: Interpolate,
    element: Option<ElementHandle>,
}

pub fn interpolate_color(
    c0: (f32, f32, f32, f32),
    c1: (f32, f32, f32, f32),
    t: f32,
) -> (f32, f32, f32, f32) {
    (
        (c1.0 - c0.0) * t + c0.0,
        (c1.1 - c0.1) * t + c0.1,
        (c1.2 - c0.2) * t + c0.2,
        (c1.3 - c0.3) * t + c0.3,
    )
}

impl Button {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.to_owned(),
            pointer_inside: false,
            pointer_held_down: false,
            pressed: false,
            hover_animate: Interpolate::new(InterpolationCurve::Ease),
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

impl Widget for Button {
    fn build(&mut self, parent: &UIBuilder) {
        let color = if self.pointer_held_down && self.pointer_inside {
            COLOR_DEPRESSED
        } else {
            interpolate_color(DEFAULT_COLOR, HOVER_COLOR, self.hover_animate.get())
        };

        let top = parent.fit().handle_events();
        top.fill(color)
            .padding(20.)
            .center_vertical()
            .text(&self.text);
        self.element = Some(top.handle());
    }

    fn event(&mut self, ui: &mut UI, event: UIEvent) {
        match event {
            UIEvent::AnimationFrame(delta) => {
                if self.pointer_inside {
                    self.hover_animate.add(delta / 200.);
                } else {
                    self.hover_animate.subtract(delta / 200.);
                }
                if self.hover_animate.not_one_or_zero() {
                    ui.request_animation_frame();
                }
            }
            UIEvent::PointerMoved => {
                if ui.pointer_in_element(self.element.unwrap()) {
                    self.pointer_inside = true;
                } else {
                    self.pointer_inside = false;
                }
            }
            UIEvent::PointerDown => {
                if ui.pointer_in_element(self.element.unwrap()) {
                    self.pointer_held_down = true
                }
            }
            UIEvent::PointerUp => {
                self.pressed = true;
                self.pointer_held_down = false;
            }
            _ => {}
        }
    }
}

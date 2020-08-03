use super::interpolation::*;
use crate::ui::{ElementHandle, UIBuilder, UIEvent};
use crate::widget::Widget;

pub const COLOR_DEPRESSED: (f32, f32, f32, f32) = (0.3, 0.3, 0.3, 1.0);
pub const DEFAULT_COLOR: (f32, f32, f32, f32) = (0.8, 0.8, 0.8, 1.0);
pub const HOVER_COLOR: (f32, f32, f32, f32) = (0.7, 0.7, 0.7, 1.0);

pub struct Button {
    pressed: bool,
    /// The button is currently held down.
    pointer_held_down: bool,
    pointer_inside: bool,
    /// 0. to 1. The current blend to the hover color
    hover_animate: Interpolate,
    element: Option<ElementHandle>,
}

impl Button {
    fn build(&mut self, parent: &UIBuilder, text: &'static str) -> ElementHandle {
        let color = if self.pointer_held_down && self.pointer_inside {
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
        top.handle()
    }
}

impl Widget for Button {
    fn new() -> Self {
        Self {
            pointer_inside: false,
            pointer_held_down: false,
            pressed: false,
            hover_animate: Interpolate::new(InterpolationCurve::Ease),
            element: None,
        }
    }

    fn event(&mut self, event: UIEvent) {
        match event {
            UIEvent::AnimationFrame(delta) => {
                let animation_speed = 100.;
                if self.pointer_inside {
                    self.hover_animate.add(delta / animation_speed);
                } else {
                    self.hover_animate.subtract(delta / animation_speed);
                }
                if self.hover_animate.not_one_or_zero() {
                    // ui.request_animation_frame();
                }
            }
            UIEvent::PointerDown => {
                self.pointer_held_down = true;
                println!("BUTTON PRESSED");
            }
            UIEvent::PointerUp => {
                self.pressed = true;
                self.pointer_held_down = false;
            }
            _ => {}
        }
    }
}

pub fn button(parent: &UIBuilder, id: u64, text: &'static str) -> bool {
    let mut button: Box<Button> = parent.get_widget(id);
    let pressed = button.pressed;
    button.pressed = false; // Reset the pressed value when we're constructed.
    let element = button.build(parent, text);
    let widget_handle = parent.add_widget(id, button);
    parent.add_widget_event_handler(element, widget_handle);
    pressed
}

use crate::ui::{ElementHandle, UIBuilder, UIEvent};
use crate::widget::{EventSubscriptions, Widget};

pub const COLOR_DEPRESSED: (f32, f32, f32, f32) = (0.3, 0.3, 0.3, 1.0);
pub const DEFAULT_COLOR: (f32, f32, f32, f32) = (0.8, 0.8, 0.8, 1.0);
pub const HOVER_COLOR: (f32, f32, f32, f32) = (0.7, 0.7, 0.7, 1.0);

pub enum InterpolationCurve {
    Linear,
    /// Smoother step function from Wikipedia:
    /// https://en.wikipedia.org/wiki/Smoothstep
    Ease,
}

pub struct Interpolate {
    t: f32,
    curve: InterpolationCurve,
}

impl Interpolate {
    pub fn new(curve: InterpolationCurve) -> Self {
        Self { t: 0., curve }
    }

    pub fn add(&mut self, delta: f32) {
        self.t += delta;
        self.t = self.t.min(1.);
    }

    pub fn subtract(&mut self, delta: f32) {
        self.t -= delta;
        self.t = self.t.max(0.);
    }

    pub fn set(&mut self, t: f32) {
        self.t = t;
        self.t = self.t.min(1.).max(0.);
    }

    pub fn not_one_or_zero(&self) -> bool {
        self.t != 0.0 && self.t != 1.0
    }

    pub fn get(&self) -> f32 {
        match self.curve {
            InterpolationCurve::Linear => self.t,
            InterpolationCurve::Ease => {
                let x = ((self.t - 0.0) / 1.0).max(0.).min(1.);
                x * x * x * (x * (x * 6. - 15.) + 10.)
            }
        }
    }
}

pub struct Button {
    pub text: String,
    pressed: bool,
    /// The button is currently held down.
    pointer_held_down: bool,
    pointer_inside: bool,
    /// 0. to 1. The current blend to the hover color
    hover_animate: Interpolate,
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
        }
    }

    /// This function resets when it is called.
    pub fn pressed(&mut self) -> bool {
        let b = self.pressed;
        self.pressed = false;
        b
    }
}

impl<T> Widget<T> for Button {
    fn build(&self, parent: &UIBuilder<T>) -> (ElementHandle, EventSubscriptions) {
        let color = if self.pointer_held_down && self.pointer_inside {
            COLOR_DEPRESSED
        } else {
            interpolate_color(DEFAULT_COLOR, HOVER_COLOR, self.hover_animate.get())
        };

        let top = parent.fit();
        top.fill(color)
            .padding(20. + self.hover_animate.get() * 80.)
            .center_vertical()
            .text(&self.text);
        (
            top.handle(),
            EventSubscriptions {
                // If the pointer is held down listen for a global pointer up to cancel the button press.
                // This ensures that if the button is pressed and then the cursor moves
                // outside the element and back in the click will still occur.
                // But if the pointer moves outside the element and then releases the click does not
                // occur.
                // This behavior matches MacOS native buttons, and it seems like good behavior.
                global_pointer_up: self.pointer_held_down,
                animation_frame: self.hover_animate.not_one_or_zero(),
                ..Default::default()
            },
        )
    }

    fn event(&mut self, event: UIEvent) {
        match event {
            UIEvent::AnimationFrame(delta) => {
                //println!("Animating");
                if self.pointer_inside {
                    self.hover_animate.add(delta / 200.);
                } else {
                    self.hover_animate.subtract(delta / 200.);
                }
            }
            UIEvent::PointerHover => {
                self.pointer_inside = true;
                self.hover_animate.add(0.0000001); // Kick off an animation frame
            }
            UIEvent::PointerExited => {
                self.pointer_inside = false;
                self.hover_animate.subtract(0.000001);
            }
            UIEvent::PointerDown => self.pointer_held_down = true,
            UIEvent::PointerUp => self.pressed = true,
            UIEvent::GlobalPointerUp => self.pointer_held_down = false,
            _ => {}
        }
    }
}

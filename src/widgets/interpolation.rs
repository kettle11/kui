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

#[derive(Copy, Clone, Debug)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn zero() -> Self {
        Rectangle {
            x: 0.,
            y: 0.,
            width: 0.,
            height: 0.,
        }
    }

    /// Calculates a rectangle that fits both rectangles
    pub fn join(&self, other: Rectangle) -> Self {
        let x = self.x.min(other.x);
        let y = self.y.min(other.y);
        let width = (other.x + other.width).max(self.x + self.width) - x;
        let height = (other.y + other.height).max(self.y + self.height) - y;

        Self {
            x,
            y,
            width,
            height,
        }
    }
}

#[derive(Debug)]
pub struct Drawable {
    pub rectangle: (f32, f32, f32, f32),
    pub texture_rectangle: (f32, f32, f32, f32),
    pub color: (f32, f32, f32, f32),
    pub radiuses: Option<(f32, f32, f32, f32)>,
}

pub struct DrawingInfo {
    pub canvas_width: f32,
    pub canvas_height: f32,
    pub texture: crate::texture::Texture,
    pub drawables: Vec<Drawable>,
}

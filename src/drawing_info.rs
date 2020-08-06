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
    pub(crate) characters: Vec<(usize, fontdue::layout::GlyphRasterConfig)>,
}

impl DrawingInfo {
    /// Update character rectangles to the latest character
    pub(crate) fn fix_character_rectangles(&mut self) {
        let texture = &self.texture;
        for (i, c) in &self.characters {
            if let Some(texture_rectangle) = texture.get_character_no_rasterize(*c) {
                self.drawables[*i].texture_rectangle = (
                    texture_rectangle.x as f32 / texture.width as f32,
                    texture_rectangle.y as f32 / texture.height as f32,
                    texture_rectangle.width as f32 / texture.width as f32,
                    texture_rectangle.height as f32 / texture.height as f32,
                );
            }
        }
    }
}

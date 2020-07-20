use crate::rectangle::RectangleU32;
use std::collections::HashMap;
pub struct Texture {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    packer: rect_packer::Packer,
    characters: HashMap<fontdue::layout::GlyphRasterConfig, RectangleU32>,
}

impl Texture {
    /// Size should be power of 2
    pub fn new(size: u32) -> Self {
        let config = rect_packer::Config {
            width: size as i32,
            height: size as i32,
            border_padding: 5,
            rectangle_padding: 10,
        };
        Self {
            data: vec![0; (size * size) as usize],
            width: size,
            height: size,
            packer: rect_packer::Packer::new(config),
            characters: HashMap::new(),
        }
    }

    pub fn get_character(
        &mut self,
        font: &fontdue::Font,
        c: fontdue::layout::GlyphRasterConfig,
        width: u32,
        height: u32,
    ) -> RectangleU32 {
        if let Some(rectangle) = self.characters.get(&c) {
            *rectangle
        } else {
            let (metrics, new_data) = font.rasterize_config(c);

            let rectangle = self.pack_character(c, metrics.width as u32, metrics.height as u32);

            /*
            use std::fs::File;
            use std::io::Write;

            let mut o = std::fs::File::create(format!("{:?}.pgm", c.c)).unwrap();
            let _ = o.write(format!("P5\n{} {}\n255\n", metrics.width, metrics.height).as_bytes());
            let _ = o.write(&new_data);
            */
            let mut new_data_index = 0;
            for j in rectangle.y..(rectangle.y + rectangle.height) {
                for i in rectangle.x..(rectangle.x + rectangle.width) {
                    self.data[(j * self.width + i) as usize] = new_data[new_data_index];
                    new_data_index += 1;
                }
            }

            rectangle
        }
    }

    fn pack_character(
        &mut self,
        c: fontdue::layout::GlyphRasterConfig,
        width: u32,
        height: u32,
    ) -> RectangleU32 {
        // Just crash for now if there's not space for character.
        let rect = self
            .packer
            .pack(width as i32, height as i32, false)
            .unwrap();
        let rectangle = RectangleU32::new(
            rect.x as u32,
            rect.y as u32,
            rect.width as u32,
            rect.height as u32,
        );
        self.characters.insert(c, rectangle);
        rectangle
    }
}

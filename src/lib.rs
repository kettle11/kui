mod drawing_info;
mod layout;
mod rectangle;
mod render;
mod texture;
mod tree;
mod ui;
pub mod widgets;
pub use drawing_info::{Drawable, DrawingInfo};
pub use render::Render;
pub use ui::*;

#[macro_export]
macro_rules! id {
    ($s:expr) => {{
        generate_id($s)
    }};
    () => {{
        let id = concat!(file!(), line!(), column!());
        id!(id)
    }};
}

pub fn generate_id(id: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut s = DefaultHasher::new();
    id.hash(&mut s);
    s.finish()
}

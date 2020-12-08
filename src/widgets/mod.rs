mod button;
mod drag;
mod interpolation;
mod scroll_view;
mod text_field;
//mod slider;
mod horizontal_divider;
mod vertical_divider;
pub use button::*;
pub use horizontal_divider::*;
pub use scroll_view::*;
//pub use slider::*;
pub use text_field::*;
pub use vertical_divider::*;

#[track_caller]
/// Pass in extra data that will be hashed together with the location the widget function is called from
fn calculate_id<T: std::hash::Hash>(t: T) -> u64 {
    use core::panic::Location;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;

    let mut s = DefaultHasher::new();

    let location = Location::caller();
    (location.file(), location.line(), location.column(), t).hash(&mut s);
    s.finish()
}

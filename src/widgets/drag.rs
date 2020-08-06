use crate::ui::UIBuilder;
use crate::ElementHandle;

/// A helper for dragging elements
pub struct Drag {
    pub root_and_element: Option<(ElementHandle, ElementHandle)>,
    dragging: bool,
    offset: (f32, f32),
    position: (f32, f32),
}

impl Drag {
    pub fn new(position: (f32, f32)) -> Self {
        Self {
            root_and_element: None,
            dragging: false,
            offset: (0., 0.),
            position,
        }
    }

    pub fn update(&mut self, parent: &UIBuilder) -> (f32, f32) {
        if let Some((root, element)) = self.root_and_element {
            if parent.pointer_down() && parent.pointer_in_element(element) {
                let element_rectangle = parent.element_rectangle(element);
                let pointer_position = parent.pointer_position();

                self.offset = (
                    pointer_position.0 - element_rectangle.x,
                    pointer_position.1 - element_rectangle.y,
                );
                self.dragging = true;
            }

            if parent.pointer_up() {
                self.dragging = false;
            }

            if self.dragging {
                let root_rectangle = parent.element_rectangle(root);
                let pointer_position = parent.pointer_position();
                self.position = (
                    pointer_position.0 - root_rectangle.x - self.offset.0,
                    pointer_position.1 - root_rectangle.x - self.offset.1,
                );
            }
        }
        self.position
    }
}

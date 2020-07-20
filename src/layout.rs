use crate::rectangle::Rectangle;
use crate::tree::{NodeHandle, Tree};
use crate::ui::{Element, ElementType};

pub const FONT_SIZE: f32 = 32.0;
/// Layout borrows things from the UI
pub(crate) struct Layout<'a> {
    pub(crate) fonts: &'a Vec<fontdue::Font>,
    pub(crate) tree: &'a Tree,
    pub(crate) elements: &'a mut Vec<Element>,
}

impl<'a> Layout<'a> {
    fn independent_layout(&mut self, node: NodeHandle) {
        for child in self.tree.child_iter(node) {
            self.layout(child);
        }
    }
    pub fn layout(&mut self, node: NodeHandle) -> (f32, f32) {
        let element = &self.elements[node.0];
        let size: (f32, f32) = match &element.element_type {
            // A row walks through all summing up their widths and taking the max of their heights
            ElementType::Row | ElementType::ReverseRow => {
                self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = self.layout(n);
                    (s.0 + child_size.0, s.1.max(child_size.1))
                })
            }
            // Takes up maximum available horizontal space. Fits to child height.
            ElementType::EvenlySpacedRow => self
                .tree
                .child_iter(node)
                .fold((f32::MAX, 0.), |s, n| (f32::MAX, s.1.max(self.layout(n).1))),
            // A row walks through all summing up their widths and taking the max of their heights
            ElementType::Column => self.tree.child_iter(node).fold((0., 0.), |s, n| {
                let child_size = self.layout(n);
                (s.0.max(child_size.0), s.1 + child_size.1)
            }),
            ElementType::Width(width) => {
                let width = *width;
                let child_size: (f32, f32) = self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = self.layout(n);
                    (s.0.max(child_size.0), s.1.max(child_size.1))
                });
                (width, child_size.1)
            }
            ElementType::Height(height) => {
                let height = *height;
                let child_size: (f32, f32) = self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = self.layout(n);
                    (s.0.max(child_size.0), s.1.max(child_size.1))
                });
                (child_size.0, height)
            }
            ElementType::Padding(padding) => {
                let padding = *padding;
                // Padding ensures that the space is requested is at least padding.
                // Probably padding shouldn't have to walk the tree and should just assume one child.
                let child_size: (f32, f32) = self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = self.layout(n);
                    (s.0.max(child_size.0), s.1.max(child_size.1))
                });
                (child_size.0 + padding * 2., child_size.1 + padding * 2.)
            }

            ElementType::Text(text) => {
                let font_size = FONT_SIZE;
                let text_style = fontdue::layout::TextStyle {
                    text: &text,
                    px: font_size,
                    font: &self.fonts[0],
                };

                let layout_settings = fontdue::layout::LayoutSettings {
                    ..fontdue::layout::LayoutSettings::default()
                };

                let mut layout = Vec::new();
                fontdue::layout::layout_horizontal(&text_style, &layout_settings, &mut layout);

                if let Some(c) = layout.get(0) {
                    let rectangle = Rectangle::new(c.x, c.y, c.width as f32, c.height as f32);
                    let total_rectangle = layout.iter().fold(rectangle, |r, c| {
                        let c_rectangle = Rectangle::new(c.x, c.y, c.width as f32, c.height as f32);
                        r.join(c_rectangle)
                    });
                    (total_rectangle.width, font_size)
                } else {
                    (0., 0.)
                }
            }
            ElementType::Fill(..) => self.tree.child_iter(node).fold((0., 0.), |s, n| {
                let child_size = self.layout(n);
                (s.0.max(child_size.0), s.1.max(child_size.1))
            }),
            ElementType::Expander => {
                self.independent_layout(node);
                (f32::MAX, f32::MAX)
            }
            _ => unimplemented!(),
        };
        self.elements[node.0].size = size;
        size
    }
}

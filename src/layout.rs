use crate::rectangle::Rectangle;
use crate::tree::{NodeHandle, Tree};
use crate::ui::{Element, ElementType, TextProperties};

/// Layout borrows things from the UI
pub(crate) struct Layout<'a> {
    pub(crate) fonts: &'a Vec<fontdue::Font>,
    pub(crate) tree: &'a Tree,
    pub(crate) elements: &'a mut Vec<Element>,
}

impl<'a> Layout<'a> {
    fn independent_layout(&mut self, text_properties: &TextProperties, node: NodeHandle) {
        for child in self.tree.child_iter(node) {
            self.layout(text_properties, child);
        }
    }
    pub fn layout(&mut self, text_properties: &TextProperties, node: NodeHandle) -> (f32, f32) {
        let element = &self.elements[node.0];
        let size: (f32, f32) = match &element.element_type {
            // A row walks through all summing up their widths and taking the max of their heights
            ElementType::Row(spacing) | ElementType::ReverseRow(spacing) => {
                let spacing = *spacing;
                self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = self.layout(text_properties, n);
                    (s.0 + child_size.0 + spacing, s.1.max(child_size.1))
                })
            }
            // Takes up maximum available horizontal space. Fits to child height.
            ElementType::EvenlySpacedRow => {
                self.tree.child_iter(node).fold((f32::MAX, 0.), |s, n| {
                    (f32::MAX, s.1.max(self.layout(text_properties, n).1))
                })
            }
            // A row walks through all summing up their widths and taking the max of their heights
            ElementType::Column(spacing) => {
                let spacing = *spacing;
                self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = self.layout(text_properties, n);
                    (s.0.max(child_size.0), s.1 + child_size.1 + spacing)
                })
            }
            ElementType::Width(width) => {
                let width = *width;
                let child_size: (f32, f32) = self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = self.layout(text_properties, n);
                    (s.0.max(child_size.0), s.1.max(child_size.1))
                });
                (width, child_size.1)
            }
            ElementType::Height(height) => {
                let height = *height;
                let child_size: (f32, f32) = self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = self.layout(text_properties, n);
                    (s.0.max(child_size.0), s.1.max(child_size.1))
                });
                (child_size.0, height)
            }
            ElementType::Padding(padding) => {
                let padding = *padding;
                // Padding ensures that the space is requested is at least padding.
                // Probably padding shouldn't have to walk the tree and should just assume one child.
                let child_size: (f32, f32) = self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = self.layout(text_properties, n);
                    (s.0.max(child_size.0), s.1.max(child_size.1))
                });
                (child_size.0 + padding * 2., child_size.1 + padding * 2.)
            }

            ElementType::Text(text) => {
                let text_style = fontdue::layout::TextStyle {
                    text: &text,
                    px: text_properties.size,
                    font: &self.fonts[0],
                };

                let text_height = self.fonts[0]
                    .horizontal_line_metrics(text_properties.size)
                    .unwrap()
                    .new_line_size;
                let layout_settings = fontdue::layout::LayoutSettings {
                    ..fontdue::layout::LayoutSettings::default()
                };

                /*
                println!(
                    "Metrics: {:?}",
                    self.fonts[0]
                        .horizontal_line_metrics(text_properties.size)
                        .unwrap()
                );

                println!("Text height: {:?}", text_height);
                */
                let mut layout = Vec::new();
                fontdue::layout::layout_horizontal(&text_style, &layout_settings, &mut layout);

                if let Some(c) = layout.get(0) {
                    let rectangle = Rectangle::new(c.x, c.y, c.width as f32, c.height as f32);
                    let total_rectangle = layout.iter().fold(rectangle, |r, c| {
                        //    println!("c: {:?}", c);

                        let c_rectangle = Rectangle::new(c.x, c.y, c.width as f32, c.height as f32);
                        r.join(c_rectangle)
                    });

                    // println!("width: {:?}", total_rectangle.width);
                    // For now add the first character's x to the width
                    // More thoughtful layout should be used instead.
                    (total_rectangle.width + rectangle.x, text_height)
                } else {
                    (0., 0.)
                }
            }
            ElementType::TextSize(size) => {
                let text_properties = TextProperties { size: *size };
                self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = self.layout(&text_properties, n);
                    (s.0.max(child_size.0), s.1.max(child_size.1))
                })
            }
            ElementType::Fill(..) | ElementType::CenterVertical => {
                self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = self.layout(text_properties, n);
                    (s.0.max(child_size.0), s.1.max(child_size.1))
                })
            }
            ElementType::Expander => {
                self.independent_layout(text_properties, node);
                (f32::MAX, f32::MAX)
            }
            _ => unimplemented!(),
        };
        self.elements[node.0].size = size;
        size
    }
}

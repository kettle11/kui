use crate::rectangle::Rectangle;
use crate::tree::{NodeHandle, Tree};
use crate::ui::{Drawable, DrawingInfo, Element, ElementType};
/// Render borrows things from the UI
pub(crate) struct Render<'a> {
    pub(crate) fonts: &'a Vec<fontdue::Font>,
    pub(crate) tree: &'a Tree,
    pub(crate) elements: &'a Vec<Element>,
    pub(crate) drawing_info: &'a mut DrawingInfo,
}

impl<'a> Render<'a> {
    /// Max space passes in the available space to the element
    pub(crate) fn render_element(
        &mut self,
        rectangle: (f32, f32, f32, f32),
        node: NodeHandle,
    ) -> (f32, f32) {
        let element = &self.elements[node.0];
        let (element_width, element_height) = element.size;
        match element.element_type {
            ElementType::Row => {
                let mut x = rectangle.0;
                let mut width = rectangle.2;
                self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let used_size = self.render_element((x, rectangle.1, width, rectangle.3), n);
                    x += used_size.0;
                    width -= used_size.0;
                    (s.0 + used_size.0, s.1.max(used_size.1))
                })
            }
            ElementType::ReverseRow => {
                let mut x = rectangle.0 + rectangle.2; // Of the upper right corner
                for child in self.tree.child_iter(node) {
                    let child_size = self.elements[child.0].size;
                    self.render_element(
                        (x - child_size.0, rectangle.1, child_size.1, rectangle.3),
                        child,
                    );
                    x -= child_size.0;
                }
                (element_width, element_height)
            }
            ElementType::EvenlySpacedRow => {
                let mut x = rectangle.0;
                let child_width = rectangle.2 / self.tree.child_iter(node).count() as f32;
                let mut child_max_height = 0.0;
                for child in self.tree.child_iter(node) {
                    let used_size =
                        self.render_element((x, rectangle.1, child_width, rectangle.3), child);
                    x += child_width;
                    child_max_height = used_size.1.max(child_max_height);
                }
                (element_width, child_max_height)
            }
            ElementType::Column => {
                let mut y = rectangle.1;
                let mut height = rectangle.3;
                self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let used_size = self.render_element((rectangle.0, y, rectangle.2, height), n);
                    y += used_size.1;
                    height -= used_size.1;
                    (s.0.max(used_size.0), s.1 + used_size.1)
                })
            }
            ElementType::Width(width) => {
                let rectangle = (rectangle.0, rectangle.1, width, rectangle.3);
                for child in self.tree.child_iter(node) {
                    self.render_element(rectangle, child);
                }
                (width, rectangle.3)
            }
            ElementType::Height(height) => {
                let rectangle = (rectangle.0, rectangle.1, rectangle.2, height);
                for child in self.tree.child_iter(node) {
                    self.render_element(rectangle, child);
                }
                (rectangle.2, height)
            }
            ElementType::Fill(color) => {
                let fill_rectangle = (
                    rectangle.0,
                    rectangle.1,
                    element_width.min(rectangle.2),
                    element_height.min(rectangle.3),
                );
                self.drawing_info.drawables.push(Drawable {
                    rectangle: fill_rectangle,
                    texture_rectangle: (0., 0., 0., 0.),
                    color,
                });
                // Render all children with the full size of the space.
                // Although technically should only contain a single child
                for child in self.tree.child_iter(node) {
                    self.render_element(rectangle, child);
                }
                (element_width, element_height) // Just use size calculated during layout
            }
            ElementType::Padding(padding) => {
                let padded_rectangle = (
                    rectangle.0 + padding,
                    rectangle.1 + padding,
                    rectangle.2 - padding * 2.,
                    rectangle.3 - padding * 2.,
                );
                for child in self.tree.child_iter(node) {
                    self.render_element(padded_rectangle, child);
                }
                (element_width, element_height) // Just use size calculated during layout
            }
            ElementType::Expander => {
                for child in self.tree.child_iter(node) {
                    self.render_element(rectangle, child);
                }
                (rectangle.2, rectangle.3)
            }
            ElementType::CenterVertical => {
                let center = rectangle.3 / 2.0;
                for child in self.tree.child_iter(node) {
                    let child_size = self.elements[child.0].size;
                    let y = center - child_size.1 / 2.0;
                    let render_rect = (rectangle.0, y, rectangle.2, child_size.1);
                    self.render_element(render_rect, child);
                }
                (element_width, element_height)
            }
            ElementType::Text(ref text) => {
                for child in self.tree.child_iter(node) {
                    self.render_element(rectangle, child);
                }

                let font_size = crate::layout::FONT_SIZE;
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
                for c in layout {
                    let texture_rectangle = self.drawing_info.texture.get_character(
                        &self.fonts[0],
                        c.key,
                        c.width as u32,
                        c.height as u32,
                    );

                    // Fontdue lays out relative to the upper left corner.
                    // Fontdue's coordinate system is with 0, 0 in the lower left.
                    let c_rectangle = (
                        rectangle.0 + c.x as f32,
                        rectangle.1 + -c.y - texture_rectangle.height as f32, // Why is this shifting like this?
                        texture_rectangle.width as f32,
                        texture_rectangle.height as f32,
                    );

                    /*
                    self.drawing_info.drawables.push(Drawable {
                        texture_rectangle: (0.0, 0.0, 0.0, 0.0),
                        rectangle: c_rectangle,
                        color: (1.0, 0.1, 0.8, 1.0),
                    });
                    */

                    self.drawing_info.drawables.push(Drawable {
                        texture_rectangle: (
                            texture_rectangle.x as f32 / self.drawing_info.texture.width as f32,
                            texture_rectangle.y as f32 / self.drawing_info.texture.height as f32,
                            texture_rectangle.width as f32 / self.drawing_info.texture.width as f32,
                            texture_rectangle.height as f32
                                / self.drawing_info.texture.height as f32,
                        ),
                        rectangle: c_rectangle,
                        color: (1.0, 1.0, 1.0, 1.0),
                    })
                }

                (element_width, element_height)
            }
            _ => unimplemented!("Unimplemented element: {:?}", element.element_type),
        }
    }
}

//! The render pass uses the sizes calculated in the layout pass to determine the positioning of all elements.
//! A parent element makes available to the child a rectangle of space.
use crate::rectangle::Rectangle;
use crate::tree::{NodeHandle, Tree};
use crate::ui::{Drawable, DrawingInfo, Element, ElementType, TextProperties};

/// Render borrows things from the UI
pub(crate) struct Render<'a> {
    pub(crate) fonts: &'a Vec<fontdue::Font>,
    pub(crate) tree: &'a Tree,
    pub(crate) elements: &'a mut Vec<Element>,
    pub(crate) drawing_info: &'a mut DrawingInfo,
}

impl<'a> Render<'a> {
    pub(crate) fn render_element(
        &mut self,
        text_properties: &TextProperties,
        rectangle: Rectangle,
        node: NodeHandle,
    ) {
        let element = &self.elements[node.0];
        let element_rectangle = element.rectangle;
        match element.element_type {
            ElementType::Fit => {
                let element_rectangle = Rectangle::new(
                    rectangle.x,
                    rectangle.y,
                    element_rectangle.width.min(rectangle.width),
                    element_rectangle.height.min(rectangle.height),
                );
                for child in self.tree.child_iter(node) {
                    self.render_element(text_properties, element_rectangle, child);
                }
                self.elements[node.0].rectangle = element_rectangle;
                return;
            }
            ElementType::Row(spacing) => {
                self.tree.child_iter(node).fold(
                    (0. as f32, 0. as f32),
                    |(width, height), child| {
                        let (child_width, child_height) =
                            self.elements[child.0].rectangle.width_height();

                        self.render_element(
                            text_properties,
                            Rectangle::new(
                                rectangle.x + width,
                                rectangle.y,
                                rectangle.width - width,
                                rectangle.height,
                            ),
                            child,
                        );
                        (width + child_width + spacing, height.max(child_height))
                    },
                );
            }
            ElementType::ReverseRow(spacing) => {
                let mut x = rectangle.x + rectangle.width; // Of the upper right corner
                for child in self.tree.child_iter(node) {
                    let child_width = self.elements[child.0].rectangle.width;
                    let child_height = self.elements[child.0].rectangle.height;

                    self.render_element(
                        text_properties,
                        Rectangle::new(
                            x - child_width - spacing,
                            rectangle.y,
                            rectangle.x + x,
                            child_height,
                        ),
                        child,
                    );
                    x -= child_width + spacing;
                }
            }
            ElementType::Column(spacing) => {
                self.tree.child_iter(node).fold(
                    (0. as f32, 0. as f32),
                    |(width, height), child| {
                        let (child_width, child_height) =
                            self.elements[child.0].rectangle.width_height();

                        self.render_element(
                            text_properties,
                            Rectangle::new(
                                rectangle.x,
                                rectangle.y + height,
                                rectangle.width,
                                rectangle.height - height,
                            ),
                            child,
                        );
                        (width.max(child_width), height + child_height + spacing)
                    },
                );
            }
            ElementType::Width(width) => {
                let rectangle = Rectangle::new(rectangle.x, rectangle.y, width, rectangle.height);
                for child in self.tree.child_iter(node) {
                    self.render_element(text_properties, rectangle, child);
                }
            }
            ElementType::Height(height) => {
                let rectangle = Rectangle::new(rectangle.x, rectangle.y, rectangle.width, height);
                for child in self.tree.child_iter(node) {
                    self.render_element(text_properties, rectangle, child);
                }
            }
            ElementType::Fill(color) => {
                let fill_rectangle = (rectangle.x, rectangle.y, rectangle.width, rectangle.height);
                self.drawing_info.drawables.push(Drawable {
                    rectangle: fill_rectangle,
                    texture_rectangle: (0., 0., 0., 0.),
                    color,
                    radiuses: None,
                });
                // Render all children with the full size of the space.
                for child in self.tree.child_iter(node) {
                    self.render_element(text_properties, rectangle, child);
                }
            }
            ElementType::RoundedFill(r, color) => {
                let fill_rectangle = (rectangle.x, rectangle.y, rectangle.width, rectangle.height);
                self.drawing_info.drawables.push(Drawable {
                    rectangle: fill_rectangle,
                    texture_rectangle: (0., 0., 0., 0.),
                    color,
                    radiuses: Some(r),
                });
                // Render all children with the full size of the space.
                for child in self.tree.child_iter(node) {
                    self.render_element(text_properties, rectangle, child);
                }
            }
            ElementType::Padding(padding) => {
                let padded_rectangle = Rectangle::new(
                    rectangle.x + padding,
                    rectangle.y + padding,
                    rectangle.width - padding * 2.,
                    rectangle.height - padding * 2.,
                );
                for child in self.tree.child_iter(node) {
                    self.render_element(text_properties, padded_rectangle, child);
                }
            }
            ElementType::Expander
            | ElementType::ExpanderVertical
            | ElementType::ExpanderHorizontal => {
                for child in self.tree.child_iter(node) {
                    self.render_element(text_properties, rectangle, child);
                }
            }
            ElementType::CenterVertical => {
                let center = rectangle.y + rectangle.height / 2.0;
                for child in self.tree.child_iter(node) {
                    let child_height = self.elements[child.0].rectangle.height;

                    let y = center - child_height / 2.0;
                    self.render_element(
                        text_properties,
                        Rectangle::new(rectangle.x, y, rectangle.width, rectangle.height),
                        child,
                    );
                }
            }
            ElementType::TextSize(size) => {
                let text_properties = TextProperties {
                    size,
                    font: text_properties.font,
                };
                for child in self.tree.child_iter(node) {
                    self.render_element(&text_properties, rectangle, child);
                }
            }
            ElementType::Font(font) => {
                let text_properties = TextProperties {
                    size: text_properties.size,
                    font: Some(font),
                };
                for child in self.tree.child_iter(node) {
                    self.render_element(&text_properties, rectangle, child);
                }
            }
            ElementType::PositionHorizontalPercentage(percentage) => {
                let rectangle = Rectangle::new(
                    rectangle.x + rectangle.width * percentage,
                    rectangle.y,
                    rectangle.width,
                    rectangle.height,
                );
                for child in self.tree.child_iter(node) {
                    self.render_element(&text_properties, rectangle, child);
                }
            }
            ElementType::PositionHorizontalPixels(pixels) => {
                let rectangle = Rectangle::new(
                    rectangle.x + pixels,
                    rectangle.y,
                    rectangle.width,
                    rectangle.height,
                );
                for child in self.tree.child_iter(node) {
                    self.render_element(&text_properties, rectangle, child);
                }
            }
            ElementType::Text(ref text) => {
                if let Some(font) = text_properties.font {
                    let text_style = fontdue::layout::TextStyle {
                        text: &text,
                        px: text_properties.size,
                        font: &self.fonts[font.0],
                    };

                    let layout_settings = fontdue::layout::LayoutSettings {
                        ..fontdue::layout::LayoutSettings::default()
                    };

                    /*
                    self.drawing_info.drawables.push(Drawable {
                        texture_rectangle: (0.0, 0.0, 0.0, 0.0),
                        rectangle: (rectangle.0, rectangle.1, element_width, element_height),
                        color: (1.0, 0.8, 0.8, 1.0),
                    });
                    */
                    let mut layout = Vec::new();
                    fontdue::layout::layout_horizontal(&text_style, &layout_settings, &mut layout);
                    for c in layout {
                        let texture_rectangle = self.drawing_info.texture.get_character(
                            &self.fonts[font.0],
                            c.key,
                            c.width as u32,
                            c.height as u32,
                        );

                        // Fontdue lays out relative to the upper left corner.
                        // Fontdue's coordinate system is with 0, 0 in the lower left.
                        let c_rectangle = (
                            rectangle.x + c.x as f32,
                            rectangle.y + -c.y - texture_rectangle.height as f32, // Why is this shifting like this?
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
                                texture_rectangle.y as f32
                                    / self.drawing_info.texture.height as f32,
                                texture_rectangle.width as f32
                                    / self.drawing_info.texture.width as f32,
                                texture_rectangle.height as f32
                                    / self.drawing_info.texture.height as f32,
                            ),
                            rectangle: c_rectangle,
                            color: (1.0, 1.0, 1.0, 1.0),
                            radiuses: None,
                        })
                    }
                }

                for child in self.tree.child_iter(node) {
                    self.render_element(text_properties, rectangle, child);
                }
            } //     _ => unimplemented!("Unimplemented element: {:?}", element.element_type),
        }
        self.elements[node.0].rectangle = rectangle;
    }
}

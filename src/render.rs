//! The render pass uses the sizes calculated in the layout pass to determine the positioning of all elements.
//! A parent element makes available to the child a rectangle of space.
use crate::drawing_info::*;
use crate::rectangle::Rectangle;
use crate::tree::{NodeHandle, Tree};
use crate::ui::{Element, ElementType, TextProperties, Widget};

/// Render borrows things from the UI
pub struct Render<'a> {
    pub(crate) fonts: &'a Vec<fontdue::Font>,
    pub tree: &'a Tree,
    pub elements: &'a mut Vec<Element>,
    pub drawing_info: &'a mut DrawingInfo,
    pub(crate) widgets: &'a mut Vec<Option<Box<dyn Widget>>>,
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
        self.elements[node.0].rectangle = rectangle;
        let element = &self.elements[node.0];

        match element.element_type {
            ElementType::Fit => {
                let element_rectangle = Rectangle::new(
                    rectangle.x,
                    rectangle.y,
                    element_rectangle.width,
                    element_rectangle.height,
                );
                self.elements[node.0].rectangle = element_rectangle;

                for child in self.tree.child_iter(node) {
                    self.render_element(text_properties, element_rectangle, child);
                }
                return;
            }
            ElementType::Flexible => {
                let element_rectangle = Rectangle::new(
                    rectangle.x,
                    rectangle.y,
                    element_rectangle.width.min(rectangle.width),
                    element_rectangle.height.min(rectangle.height),
                );
                self.elements[node.0].rectangle = element_rectangle;

                for child in self.tree.child_iter(node) {
                    self.render_element(text_properties, element_rectangle, child);
                }
                return;
            }
            ElementType::ScaleToFit => {
                for child in self.tree.child_iter(node) {
                    let child_rect = self.elements[child.0].rectangle;
                    let scale = (rectangle.width / child_rect.width)
                        .min(rectangle.height / child_rect.height);

                    let child_rectangle = Rectangle::new(
                        rectangle.x,
                        rectangle.y,
                        scale * child_rect.width,
                        scale * child_rect.height,
                    );

                    self.render_element(text_properties, child_rectangle, child);
                }
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
            ElementType::WidthPercentage(width) => {
                let rectangle = Rectangle::new(
                    rectangle.x,
                    rectangle.y,
                    rectangle.width * width,
                    rectangle.height,
                );
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
            ElementType::Padding(padding_width, padding_height) => {
                let padded_rectangle = Rectangle::new(
                    rectangle.x + padding_width,
                    rectangle.y + padding_height,
                    rectangle.width - padding_width * 2.,
                    rectangle.height - padding_height * 2.,
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

            ElementType::Center(horizontal, vertical) => {
                let center_x = if horizontal {
                    rectangle.x + rectangle.width / 2.0
                } else {
                    rectangle.x
                };
                let center_y = if vertical {
                    rectangle.y + rectangle.height / 2.0
                } else {
                    rectangle.y
                };

                for child in self.tree.child_iter(node) {
                    let child_width = self.elements[child.0].rectangle.width;
                    let child_height = self.elements[child.0].rectangle.height;
                    let x = if horizontal {
                        center_x - child_width / 2.0
                    } else {
                        rectangle.x
                    };
                    let y = if vertical {
                        center_y - child_height / 2.0
                    } else {
                        rectangle.y
                    };

                    self.render_element(
                        text_properties,
                        Rectangle::new(x, y, rectangle.width, rectangle.height),
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
            ElementType::PositionVerticalPixels(pixels) => {
                let rectangle = Rectangle::new(
                    rectangle.x,
                    rectangle.y + pixels,
                    rectangle.width,
                    rectangle.height,
                );
                for child in self.tree.child_iter(node) {
                    self.render_element(&text_properties, rectangle, child);
                }
            }
            ElementType::Text(ref text) => {
                // If no size is specified then scale the text to fit within the available space.
                let text_size = text_properties.size;

                if let Some(font) = text_properties.font {
                    let fonts = [&self.fonts[font.0]];
                    let text_style = fontdue::layout::TextStyle {
                        text: &text,
                        px: text_size,
                        font_index: 0,
                    };

                    let layout_settings = fontdue::layout::LayoutSettings {
                        ..fontdue::layout::LayoutSettings::default()
                    };

                    let mut layout_output = Vec::new();
                    let mut layout = fontdue::layout::Layout::new();

                    layout.layout_horizontal(
                        &fonts,
                        &[&text_style],
                        &layout_settings,
                        &mut layout_output,
                    );

                    // It'd be good to have an option to trim overflow text if it's too long for the container.

                    for c in layout_output {
                        let texture_rectangle = self.drawing_info.texture.get_character(
                            &self.fonts[font.0],
                            c.key,
                            c.width as u32,
                            c.height as u32,
                        );

                        // If the character cannot be packed (it is too large or there's not space) then don't render it
                        if let Some(texture_rectangle) = texture_rectangle {
                            self.drawing_info
                                .characters
                                .push((self.drawing_info.drawables.len(), c.key));
                            // Fontdue lays out relative to the upper left corner.
                            // Fontdue's coordinate system is with 0, 0 in the lower left.
                            let c_rectangle = (
                                rectangle.x as f32 + c.x,
                                rectangle.y + -c.y - texture_rectangle.height as f32, // Why is this shifting like this?
                                texture_rectangle.width as f32,
                                texture_rectangle.height as f32,
                            );

                            /*
                            self.drawing_info.drawables.push(Drawable {
                                texture_rectangle: (0., 0., 0., 0.), // This will be replaced later in the texture rectangle fixup step
                                rectangle: c_rectangle,
                                color: (1.0, 0.0, 0.0, 1.0),
                                radiuses: None,
                            });*/
                            self.drawing_info.drawables.push(Drawable {
                                texture_rectangle: (0., 0., 0., 0.), // This will be replaced later in the texture rectangle fixup step
                                rectangle: c_rectangle,
                                color: (1.0, 1.0, 1.0, 1.0),
                                radiuses: None,
                            });
                        /*
                        self.drawing_info.drawables.push(Drawable {
                            texture_rectangle: (0., 0., 0., 0.), // This will be replaced later in the texture rectangle fixup step
                            rectangle: c_rectangle,
                            color: (1.0, 0.0, 0.0, 1.0),
                            radiuses: None,
                        });*/
                        } else {
                            println!("Text unrendered because texture atlas is full");
                        }
                    }
                }

                for child in self.tree.child_iter(node) {
                    self.render_element(text_properties, rectangle, child);
                }
            }
            ElementType::CustomRender(handle) => {
                let mut widget = self.widgets[handle.0].take();
                if let Some(widget) = widget.as_mut() {
                    widget.draw(self, node, rectangle, text_properties);
                }
                self.widgets[handle.0] = widget;
            }
        }
    }
}

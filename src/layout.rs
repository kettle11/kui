//! Layout is responsible for determining the sizing of each element.
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
    /// Lays out children and returns their total width and height.
    fn layout_children(
        &mut self,
        parent_size: (f32, f32),
        text_properties: &TextProperties,
        node: NodeHandle,
    ) -> (f32, f32) {
        self.tree.child_iter(node).fold((0., 0.), |s, n| {
            let (child_width, child_height) = self.layout(parent_size, &text_properties, n);
            (s.0.max(child_width), s.1.max(child_height))
        })
    }

    pub fn layout(
        &mut self,
        parent_size: (f32, f32),
        text_properties: &TextProperties,
        node: NodeHandle,
    ) -> (f32, f32) {
        let element = &self.elements[node.0];

        let size: (f32, f32) = match element.element_type {
            ElementType::Fit
            | ElementType::CustomRender(..)
            | ElementType::Flexible
            | ElementType::Fill(..)
            | ElementType::RoundedFill(..)
            | ElementType::Center(..)
            | ElementType::PositionHorizontalPercentage(_)
            | ElementType::PositionHorizontalPixels(_)
            | ElementType::PositionVerticalPixels(_) => {
                self.layout_children(parent_size, text_properties, node)
            }
            ElementType::WidthPercentage(percentage) => {
                let parent_size = (parent_size.0 * percentage, parent_size.1);
                self.layout_children(parent_size, text_properties, node)
            }
            ElementType::ScaleToFit => {
                let (children_width, children_height): (f32, f32) =
                    self.layout_children(parent_size, &text_properties, node);

                let scale = (parent_size.0 / children_width).min(parent_size.1 / children_height);
                (children_width * scale, children_height * scale)
            }
            // A row walks through all summing up their widths and taking the max of their heights
            ElementType::Row(spacing) | ElementType::ReverseRow(spacing) => {
                self.tree.child_iter(node).fold((0., 0.), |s, n| {
                    let (child_width, child_height) = self.layout(parent_size, text_properties, n);
                    (s.0 + child_width + spacing, s.1.max(child_height))
                })
            }
            // A row walks through all summing up their widths and taking the max of their heights
            ElementType::Column(spacing) => self.tree.child_iter(node).fold((0., 0.), |s, n| {
                let (child_width, child_height) = self.layout(parent_size, text_properties, n);
                (s.0.max(child_width), s.1 + child_height + spacing)
            }),
            ElementType::Padding(padding_width, padding_height) => {
                // Padding ensures that the space is requested is at least padding.
                // Probably padding shouldn't have to walk the tree and should just assume one child.
                let (children_width, children_height): (f32, f32) =
                    self.layout_children(parent_size, &text_properties, node);

                (
                    children_width + padding_width * 2.,
                    children_height + padding_height * 2.,
                )
            }
            // The following elements do not rearrange children.
            ElementType::Width(width) => {
                let children_height: f32 = self
                    .layout_children((width, parent_size.1), &text_properties, node)
                    .1;
                (width, children_height)
            }
            ElementType::Height(height) => {
                let children_width: f32 = self
                    .layout_children((parent_size.0, height), &text_properties, node)
                    .0;
                (children_width, height)
            }
            ElementType::TextSize(size) => {
                let text_properties = TextProperties {
                    size,
                    font: text_properties.font,
                };
                self.layout_children(parent_size, &text_properties, node)
            }
            ElementType::Font(font) => {
                let text_properties = TextProperties {
                    size: text_properties.size,
                    font: Some(font),
                };
                self.layout_children(parent_size, &text_properties, node)
            }
            ElementType::Expander => {
                self.layout_children(parent_size, text_properties, node);
                parent_size
            }
            ElementType::ExpanderHorizontal => {
                let (_, height) = self.layout_children(parent_size, text_properties, node);
                (parent_size.0, height)
            }
            ElementType::ExpanderVertical => {
                let (width, _) = self.layout_children(parent_size, text_properties, node);
                (width, parent_size.1)
            }

            ElementType::Text(ref text) => {
                // Choose an arbitrary size if known is specified.
                let text_size = text_properties.size;
                if let Some(font) = text_properties.font {
                    let fonts = [&self.fonts[font.0]];
                    let text_style = fontdue::layout::TextStyle {
                        text: &text,
                        px: text_size,
                        font_index: 0,
                    };

                    // Should this be used if a text size is specified?
                    let text_height = self.fonts[font.0]
                        .horizontal_line_metrics(text_size)
                        .unwrap()
                        .new_line_size;
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

                    if let Some(c) = layout_output.get(0) {
                        let rectangle = Rectangle::new(c.x, c.y, c.width as f32, c.height as f32);
                        let total_rectangle = layout_output.iter().fold(rectangle, |r, c| {
                            //   println!("c.y: {:?} c.height: {:?}", c.y, c.height);
                            let c_rectangle =
                                Rectangle::new(c.x, c.y, c.width as f32, c.height as f32);
                            r.join(c_rectangle)
                        });

                        (total_rectangle.width, total_rectangle.height)
                    } else {
                        (0., 0.)
                    }
                } else {
                    (0., 0.)
                }
            }
        };
        // x and y are unassigned until render pass
        self.elements[node.0].rectangle = Rectangle::new(0., 0., size.0, size.1);
        size
    }
}

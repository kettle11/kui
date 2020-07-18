use crate::rectangle::Rectangle;
use crate::tree::{NodeHandle, Tree};
use fontdue;
pub type ElementHandle = NodeHandle;

#[derive(Debug)]
pub enum ElementType {
    /// Draw a rectangle equivalent to the area of its children
    Fill((f32, f32, f32, f32)),
    /// A container that accepts a single element and constrains its width
    Width(f32),
    /// A container that accepts a single element and constrains its height
    Height(f32),
    /// A container that accepts a single element and pads its width and height
    Padding(f32),
    /// Rows lay out multiple elements in a row.
    Row,
    /// Reverse row lay out multiple elements in a row with the opposite alignment.
    ReverseRow,
    /// Lays out multiple elements evenly spaced
    EvenlySpacedRow,
    /// Columns lay out multiple elements in a column.
    Column,
    /// Unstyled text
    Text(String),
    /// Always takes up maximum available space
    Expander,
}
pub struct Element {
    element_type: ElementType,
    size: (f32, f32),
}

pub struct UI {
    tree: Tree,
    root: NodeHandle,
    elements: Vec<Element>,
    width: f32,
    height: f32,
    drawables: Vec<Drawable>,
    fonts: Vec<fontdue::Font>,
}

/// Layout borrows things from the UI
struct Layout<'a> {
    fonts: &'a Vec<fontdue::Font>,
    tree: &'a Tree,
    elements: &'a mut Vec<Element>,
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
                let text_style = fontdue::layout::TextStyle {
                    text: &text,
                    px: 60.,
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
                    (total_rectangle.width, total_rectangle.height)
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

impl UI {
    pub fn new() -> Self {
        let mut ui = Self {
            tree: Tree::new(),
            elements: Vec::new(),
            root: NodeHandle(0),
            width: 0.0,
            height: 0.0,
            drawables: Vec::new(),
            fonts: Vec::new(),
        };
        ui.add(ElementType::Expander, None);
        ui
    }

    pub fn font_from_bytes(&mut self, bytes: &[u8]) {
        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).unwrap();
        self.fonts.push(font);
    }

    /// Directly add an element to the UI tree.
    pub fn add(
        &mut self,
        element_type: ElementType,
        parent: Option<ElementHandle>,
    ) -> ElementHandle {
        let new_handle = self.tree.add(parent);
        let element = Element {
            element_type,
            size: (0., 0.),
        };
        // If the tree has allocated a new index, push the element there.
        if new_handle.0 >= self.elements.len() {
            self.elements.push(element)
        } else {
            self.elements[new_handle.0] = element;
        }
        new_handle
    }

    pub fn edit(&mut self) -> UIBuilder {
        let root = self.root;
        UIBuilder {
            ui: Rc::new(RefCell::new(self)),
            current_container: Some(root),
        }
    }

    /// Max space passes in the available space to the element
    fn render_element(
        tree: &Tree,
        elements: &mut Vec<Element>,
        drawables: &mut Vec<Drawable>,
        rectangle: (f32, f32, f32, f32),
        node: NodeHandle,
    ) -> (f32, f32) {
        let element = &elements[node.0];
        let (element_width, element_height) = element.size;
        match element.element_type {
            ElementType::Row => {
                let mut x = rectangle.0;
                let mut width = rectangle.2;
                tree.child_iter(node).fold((0., 0.), |s, n| {
                    let used_size = Self::render_element(
                        tree,
                        elements,
                        drawables,
                        (x, rectangle.1, width, rectangle.3),
                        n,
                    );
                    x += used_size.0;
                    width -= used_size.0;
                    (s.0 + used_size.0, s.1.max(used_size.1))
                })
            }
            ElementType::ReverseRow => {
                let mut x = rectangle.0 + rectangle.2; // Of the upper right corner
                for child in tree.child_iter(node) {
                    let child_size = elements[child.0].size;
                    Self::render_element(
                        tree,
                        elements,
                        drawables,
                        (x - child_size.0, rectangle.1, child_size.1, rectangle.3),
                        child,
                    );
                    x -= child_size.0;
                }
                (element_width, element_height)
            }
            ElementType::EvenlySpacedRow => {
                let mut x = rectangle.0;
                let child_width = rectangle.2 / tree.child_iter(node).count() as f32;
                let mut child_max_height = 0.0;
                for child in tree.child_iter(node) {
                    let used_size = Self::render_element(
                        tree,
                        elements,
                        drawables,
                        (x, rectangle.1, child_width, rectangle.3),
                        child,
                    );
                    x += child_width;
                    child_max_height = used_size.1.max(child_max_height);
                }
                (element_width, child_max_height)
            }
            ElementType::Column => {
                let mut y = rectangle.1;
                let mut height = rectangle.3;
                tree.child_iter(node).fold((0., 0.), |s, n| {
                    let used_size = Self::render_element(
                        tree,
                        elements,
                        drawables,
                        (rectangle.0, y, rectangle.2, height),
                        n,
                    );
                    y += used_size.1;
                    height -= used_size.1;
                    (s.0.max(used_size.0), s.1 + used_size.1)
                })
            }
            ElementType::Width(width) => {
                let rectangle = (rectangle.0, rectangle.1, width, rectangle.3);
                for child in tree.child_iter(node) {
                    Self::render_element(tree, elements, drawables, rectangle, child);
                }
                (width, rectangle.3)
            }
            ElementType::Height(height) => {
                let rectangle = (rectangle.0, rectangle.1, rectangle.2, height);
                for child in tree.child_iter(node) {
                    Self::render_element(tree, elements, drawables, rectangle, child);
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
                drawables.push(Drawable {
                    rectangle: fill_rectangle,
                    color,
                });
                // Render all children with the full size of the space.
                // Although technically should only contain a single child
                for child in tree.child_iter(node) {
                    Self::render_element(tree, elements, drawables, rectangle, child);
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
                for child in tree.child_iter(node) {
                    Self::render_element(tree, elements, drawables, padded_rectangle, child);
                }
                (element_width, element_height) // Just use size calculated during layout
            }
            ElementType::Expander => {
                // A stack simply renders all children with the full size of the space.
                for child in tree.child_iter(node) {
                    Self::render_element(tree, elements, drawables, rectangle, child);
                }
                (rectangle.2, rectangle.3)
            }
            ElementType::Text(_) => {
                // A stack simply renders all children with the full size of the space.
                for child in tree.child_iter(node) {
                    Self::render_element(tree, elements, drawables, rectangle, child);
                }
                (rectangle.2, rectangle.3)
            }
            _ => unimplemented!("Unimplemented element: {:?}", element.element_type),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn render(&mut self) -> &Vec<Drawable> {
        let now = std::time::Instant::now();

        let mut layout = Layout {
            fonts: &self.fonts,
            tree: &self.tree,
            elements: &mut self.elements,
        };
        layout.layout(self.root);

        self.drawables.clear();
        Self::render_element(
            &self.tree,
            &mut self.elements,
            &mut self.drawables,
            (0., 0., self.width, self.height),
            self.root,
        );

        println!("Time: {:?}", now.elapsed().as_secs_f32());

        &self.drawables
    }
}

#[derive(Debug)]
pub struct Drawable {
    pub rectangle: (f32, f32, f32, f32),
    pub color: (f32, f32, f32, f32),
}

use std::cell::RefCell;
use std::rc::Rc;
pub struct UIBuilder<'a> {
    ui: Rc<RefCell<&'a mut UI>>,
    current_container: Option<NodeHandle>,
}

impl<'a> UIBuilder<'a> {
    pub fn add(&self, element_type: ElementType) -> Self {
        let new_container = self
            .ui
            .borrow_mut()
            .add(element_type, self.current_container);
        UIBuilder {
            ui: self.ui.clone(),
            current_container: Some(new_container),
        }
    }

    pub fn row(&self) -> Self {
        self.add(ElementType::Row)
    }

    pub fn reverse_row(&self) -> Self {
        self.add(ElementType::ReverseRow)
    }

    pub fn evenly_spaced_row(&self) -> Self {
        self.add(ElementType::EvenlySpacedRow)
    }

    pub fn column(&self) -> Self {
        self.add(ElementType::Column)
    }

    pub fn expander(&self) -> Self {
        self.add(ElementType::Expander)
    }

    pub fn width(&self, width_pixels: f32) -> Self {
        self.add(ElementType::Width(width_pixels))
    }

    pub fn padding(&self, padding: f32) -> Self {
        self.add(ElementType::Padding(padding))
    }

    pub fn height(&self, height_pixels: f32) -> Self {
        self.add(ElementType::Height(height_pixels))
    }

    pub fn fill(&self, color: (f32, f32, f32, f32)) -> Self {
        self.add(ElementType::Fill(color))
    }

    pub fn text(&self, text: &str) -> Self {
        self.add(ElementType::Text(text.to_owned()))
    }
}

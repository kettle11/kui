use crate::tree::{NodeHandle, Tree};

pub type ElementHandle = NodeHandle;

#[derive(Debug)]
pub enum ElementType {
    /// A fill is a container that accepts a single element
    Fill((f32, f32, f32, f32)),
    /// A container that defines a minimum width
    MinimumWidth(f32),
    /// A container that accepts a single element and constrains its width
    Width(f32),
    /// A container that accepts a single element and constrains its height
    Height(f32),
    /// A container that accepts a single element and pads its width and height
    Padding(f32),
    /// Stacks accept multiple elements and put them on top of eachother.
    Stack,
    /// Rows lay out multiple elements in a row.
    Row,
    /// Reverse row lay out multiple elements in a row with the opposite alignment.
    ReverseRow,
    /// Lays out multiple elements evenly spaced
    EvenlySpacedRow,
    /// Columns lay out multiple elements in a column.
    Column,
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
}

impl UI {
    pub fn new() -> Self {
        let mut ui = Self {
            tree: Tree::new(),
            elements: Vec::new(),
            root: NodeHandle(0),
            width: 0.0,
            height: 0.0,
        };
        ui.add(ElementType::Stack, None);
        ui
    }

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
            ui: self,
            current_container: Some(root),
        }
    }

    /// Walk through the layout tree.
    /// Each element requests the maximum space possible according to their rules.
    fn layout(tree: &Tree, elements: &mut Vec<Element>, node: NodeHandle) -> (f32, f32) {
        fn independent_layout(tree: &Tree, elements: &mut Vec<Element>, node: NodeHandle) {
            for child in tree.child_iter(node) {
                UI::layout(tree, elements, child);
            }
        }
        let element = &elements[node.0];
        let size: (f32, f32) = match element.element_type {
            // A row walks through all summing up their widths and taking the max of their heights
            ElementType::Row | ElementType::ReverseRow => {
                tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = Self::layout(tree, elements, n);
                    (s.0 + child_size.0, s.1.max(child_size.1))
                })
            }
            // Takes up maximum available horizontal space. Fits to child height.
            ElementType::EvenlySpacedRow => tree.child_iter(node).fold((f32::MAX, 0.), |s, n| {
                (f32::MAX, s.1.max(Self::layout(tree, elements, n).1))
            }),
            // A row walks through all summing up their widths and taking the max of their heights
            ElementType::Column => tree.child_iter(node).fold((0., 0.), |s, n| {
                let child_size = Self::layout(tree, elements, n);
                (s.0.max(child_size.0), s.1 + child_size.1)
            }),
            ElementType::Width(width) => {
                independent_layout(tree, elements, node);
                (width, f32::MAX)
            }
            ElementType::MinimumWidth(width) => {
                independent_layout(tree, elements, node);
                (width, f32::MAX)
            }
            ElementType::Height(height) => {
                independent_layout(tree, elements, node);
                (f32::MAX, height)
            }
            ElementType::Padding(padding) => {
                // Padding ensures that the space is requested is atleast padding.
                // Probably padding shouldn't have to walk the tree and should just assume one child.
                let child_size: (f32, f32) = tree.child_iter(node).fold((0., 0.), |s, n| {
                    let child_size = Self::layout(tree, elements, n);
                    (s.0.max(child_size.0), s.1.max(child_size.1))
                });
                (padding.max(child_size.0), padding.max(child_size.1))
            }
            ElementType::Stack | ElementType::Fill(..) => {
                independent_layout(tree, elements, node);
                (f32::MAX, f32::MAX)
            }
            _ => unimplemented!(),
        };
        elements[node.0].size = size;
        size
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
                drawables.push(Drawable { rectangle, color });
                // Render all children with the full size of the space.
                // Although technically should only contain a single child
                for child in tree.child_iter(node) {
                    Self::render_element(tree, elements, drawables, rectangle, child);
                }
                (rectangle.2, rectangle.3)
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
            ElementType::Stack => {
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

    pub fn render(&mut self) -> Vec<Drawable> {
        Self::layout(&self.tree, &mut self.elements, self.root);
        let mut drawables = Vec::new();
        Self::render_element(
            &self.tree,
            &mut self.elements,
            &mut drawables,
            (0., 0., self.width, self.height),
            self.root,
        );
        drawables
    }
}

#[derive(Debug)]
pub struct Drawable {
    pub rectangle: (f32, f32, f32, f32),
    pub color: (f32, f32, f32, f32),
}

pub struct UIBuilder<'a> {
    ui: &'a mut UI,
    current_container: Option<NodeHandle>,
}

impl<'b, 'a: 'b> UIBuilder<'a> {
    fn add_container(&'b mut self, element_type: ElementType) -> UIBuilder<'b> {
        let new_container = self.ui.add(element_type, self.current_container);
        UIBuilder {
            ui: self.ui,
            current_container: Some(new_container),
        }
    }

    fn add_container_single(&'b mut self, element_type: ElementType) -> UIBuilderSingle<'b> {
        let new_container = self.ui.add(element_type, self.current_container);
        UIBuilderSingle {
            ui: self.ui,
            current_container: Some(new_container),
        }
    }

    fn add_element(&'b mut self, element_type: ElementType) -> &'b mut Self {
        self.ui.add(element_type, self.current_container);
        self
    }

    pub fn row(&'b mut self) -> UIBuilder<'b> {
        self.add_container(ElementType::Row)
    }

    pub fn reverse_row(&'b mut self) -> UIBuilder<'b> {
        self.add_container(ElementType::ReverseRow)
    }

    pub fn evenly_spaced_row(&'b mut self) -> UIBuilder<'b> {
        self.add_container(ElementType::EvenlySpacedRow)
    }

    pub fn column(&'b mut self) -> UIBuilder<'b> {
        self.add_container(ElementType::Column)
    }

    pub fn stack(&'b mut self) -> UIBuilder<'b> {
        self.add_container(ElementType::Stack)
    }

    pub fn width(&'b mut self, width_pixels: f32) -> UIBuilderSingle<'b> {
        self.add_container_single(ElementType::Width(width_pixels))
    }

    pub fn padding(&'b mut self, padding: f32) -> UIBuilderSingle<'b> {
        self.add_container_single(ElementType::Padding(padding))
    }

    pub fn height(&'b mut self, height_pixels: f32) -> UIBuilderSingle<'b> {
        self.add_container_single(ElementType::Height(height_pixels))
    }

    pub fn fill(&'b mut self, color: (f32, f32, f32, f32)) -> UIBuilderSingle<'b> {
        self.add_container_single(ElementType::Fill(color))
    }
}

/// Containers that only accept a single child.
pub struct UIBuilderSingle<'a> {
    ui: &'a mut UI,
    current_container: Option<NodeHandle>,
}

impl<'a> UIBuilderSingle<'a> {
    fn add_container(self, element_type: ElementType) -> UIBuilder<'a> {
        let new_container = self.ui.add(element_type, self.current_container);
        UIBuilder {
            ui: self.ui,
            current_container: Some(new_container),
        }
    }

    fn add_container_single(mut self, element_type: ElementType) -> Self {
        let new_container = self.ui.add(element_type, self.current_container);
        self.current_container = Some(new_container);
        self
    }

    fn add_element(self, element_type: ElementType) -> Self {
        self.ui.add(element_type, self.current_container);
        self
    }

    pub fn stack(self) -> UIBuilder<'a> {
        self.add_container(ElementType::Stack)
    }

    pub fn row(self) -> UIBuilder<'a> {
        self.add_container(ElementType::Row)
    }

    pub fn reverse_row(self) -> UIBuilder<'a> {
        self.add_container(ElementType::ReverseRow)
    }

    pub fn evenly_spaced_row(self) -> UIBuilder<'a> {
        self.add_container(ElementType::EvenlySpacedRow)
    }

    pub fn column(self) -> UIBuilder<'a> {
        self.add_container(ElementType::Column)
    }

    pub fn width(self, width_pixels: f32) -> Self {
        self.add_container_single(ElementType::Width(width_pixels))
    }

    pub fn padding(self, padding: f32) -> Self {
        self.add_container_single(ElementType::Padding(padding))
    }

    pub fn height(self, height_pixels: f32) -> Self {
        self.add_container_single(ElementType::Height(height_pixels))
    }

    pub fn fill(self, color: (f32, f32, f32, f32)) -> Self {
        self.add_container_single(ElementType::Fill(color))
    }
}

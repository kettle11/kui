use fontdue;

use crate::drawing_info::*;
use crate::layout::Layout;
use crate::rectangle::Rectangle;
use crate::render::Render;
use crate::texture::Texture;
use crate::tree::{NodeHandle, Tree};

use std::any::Any;

pub trait Widget: ToAny {
    fn draw(
        &mut self,
        rectangle: Rectangle,
        drawing_info: &mut DrawingInfo,
        elements: &Vec<Element>,
    ) {
    }
}

pub trait ToAny {
    fn to_any(self: Box<Self>) -> Box<dyn Any>;
}

impl<T: Any + Widget> ToAny for T {
    fn to_any(self: Box<Self>) -> Box<dyn Any> {
        self
    }
}

pub type ElementHandle = NodeHandle;

#[derive(Copy, Clone, Debug)]
pub struct FontHandle(pub(crate) usize);

pub(crate) struct TextProperties {
    pub(crate) size: f32,
    pub(crate) font: Option<FontHandle>,
}

impl TextProperties {
    pub fn new() -> Self {
        Self {
            // 17 * 2. When DPI scaling is added change this to 17.
            // 17 is the recommended size for buttons on iOS, so a bit arbitrary.
            size: 34.,
            font: None,
        }
    }
}

#[derive(Debug)]
pub enum ElementType {
    Fill((f32, f32, f32, f32)),
    /// A rounded fill. The first f32s are corner radiuses, the second are colors.
    RoundedFill((f32, f32, f32, f32), (f32, f32, f32, f32)),
    /// A container that accepts a single element and constrains its width
    Width(f32),
    /// A container that accepts a single element and constrains its width by a percentage
    WidthPercentage(f32),
    /// A container that accepts a single element and constrains its height
    Height(f32),
    /// A container that accepts a single element and pads its width and height
    Padding(f32, f32),
    /// Rows lay out multiple elements in a row.
    /// Accepts a single value for spacing between elements
    Row(f32),
    /// Reverse row lay out multiple elements in a row with the opposite alignment.
    ReverseRow(f32),
    /// Moves an element by a percentage of the parent towards the end of the parent.
    PositionHorizontalPercentage(f32),
    /// Moves an element towards the horizontal end of the parent.
    PositionHorizontalPixels(f32),
    /// Moves an element towards the vertical end of the parent.
    PositionVerticalPixels(f32),
    /// Columns lay out multiple elements in a column.
    /// Accepts a single value for spacing between elements
    Column(f32),
    /// Unstyled text
    Text(String),
    /// Specify text size for dependent elements,
    TextSize(f32),
    /// Specifies font to use for children. Defaults to 'None'.
    Font(FontHandle),
    /// Centers children vertically in available space
    CenterVertical,
    /// Always takes up maximum available space
    Expander,
    /// Takes up all horizontal space
    ExpanderHorizontal,
    /// Takes up all vertical space
    ExpanderVertical,
    /// Is the size of its children
    Fit,
    /// Is the size of the children, but not bigger than parent space.
    Flexible,
    /// A bit of a hack to set an element's height to be based on the height of two elements
    /// rendered previously
    ScrollbarVertical(ElementHandle, ElementHandle),
    /// Used in cases where custom rendering logic is needed that depends on a prior render pass
    /// The usize passed can be used for the widget ID.
    CustomRender(WidgetHandle),
}

pub struct Element {
    pub element_type: ElementType,
    pub rectangle: Rectangle,
    pub widget: Option<usize>,
}

use std::collections::HashMap;

pub struct UITree {
    tree: Tree,
    root: NodeHandle,
    elements: Vec<Element>,
}

impl UITree {
    pub fn new() -> Self {
        let mut ui_tree = Self {
            tree: Tree::new(),
            elements: Vec::new(),
            root: NodeHandle(0),
        };
        ui_tree.add(ElementType::Expander, None);
        ui_tree
    }

    pub fn add(
        &mut self,
        element_type: ElementType,
        parent: Option<ElementHandle>,
    ) -> ElementHandle {
        let new_handle = self.tree.add(parent);
        let element = Element {
            element_type,
            rectangle: Rectangle::zero(),
            widget: None,
        };
        // If the tree has allocated a new index, push the element there.
        if new_handle.0 >= self.elements.len() {
            self.elements.push(element)
        } else {
            self.elements[new_handle.0] = element;
        }
        new_handle
    }

    pub fn reset(&mut self) {
        self.tree.remove(self.root);
        self.root = self.add(ElementType::Expander, None);
    }
}

pub struct UI {
    // Two UITrees are double buffered.
    // When a new UITree is being constructed it can query for how user inputs
    // interacted with the previous UITree.
    current_ui_tree: UITree,
    old_ui_tree: UITree,
    width: f32,
    height: f32,
    drawing_info: DrawingInfo,
    fonts: Vec<fontdue::Font>,
    pointer_x: f32,
    pointer_y: f32,
    pointer_down: bool,
    pointer_up: bool,
    scroll_delta: f32,
    last_animation_timestamp: Option<std::time::Instant>,
    animation_frame_requested: bool,
    widgets: Vec<Option<Box<dyn Widget>>>,
    widget_id_to_index: HashMap<u64, usize>,
}

impl UI {
    pub fn new() -> Self {
        let mut ui = Self {
            current_ui_tree: UITree::new(),
            old_ui_tree: UITree::new(),
            width: 0.0,
            height: 0.0,
            drawing_info: DrawingInfo {
                canvas_width: 0.0,
                canvas_height: 0.0,
                drawables: Vec::new(),
                texture: Texture::new(512),
            },
            fonts: Vec::new(),
            pointer_x: 0.0,
            pointer_y: 0.0,
            pointer_down: false,
            pointer_up: false,
            scroll_delta: 0.,
            last_animation_timestamp: None,
            animation_frame_requested: false,
            widgets: Vec::new(),
            widget_id_to_index: HashMap::new(),
        };
        // ui.current_ui_tree.add(ElementType::Expander, None);
        ui
    }

    pub fn font_from_bytes(&mut self, bytes: &[u8]) -> FontHandle {
        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).unwrap();
        self.fonts.push(font);
        FontHandle(self.fonts.len() - 1)
    }

    pub fn edit<'a>(&'a mut self) -> UIBuilder {
        std::mem::swap(&mut self.old_ui_tree, &mut self.current_ui_tree);
        self.current_ui_tree.reset();
        let root = self.current_ui_tree.root;
        UIBuilder {
            ui: Rc::new(RefCell::new(self)),
            parent: Some(root),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    pub fn render(&mut self) -> &DrawingInfo {
        // First layout the elements.
        // Calculate the sizes for various elements.
        let mut layout = Layout {
            fonts: &self.fonts,
            tree: &self.current_ui_tree.tree,
            elements: &mut self.current_ui_tree.elements,
        };
        let text_properties = TextProperties::new();

        layout.layout(&text_properties, self.current_ui_tree.root);

        self.drawing_info.drawables.clear();

        // Then render the final outputs based on the previously calculated sizes.
        let mut render = Render {
            fonts: &self.fonts,
            tree: &self.current_ui_tree.tree,
            elements: &mut self.current_ui_tree.elements,
            drawing_info: &mut self.drawing_info,
            widgets: &mut self.widgets,
        };

        render.render_element(
            &text_properties,
            Rectangle::new(0., 0., self.width, self.height),
            self.current_ui_tree.root,
        );

        //println!("Time: {:?}", now.elapsed().as_secs_f32());
        self.drawing_info.canvas_width = self.width;
        self.drawing_info.canvas_height = self.height;

        self.pointer_down = false;
        self.pointer_up = false;
        self.scroll_delta = 0.;

        &self.drawing_info
    }

    /// Move the pointer (mouse or touch) and trigger any potential events.
    pub fn pointer_move(&mut self, x: f32, y: f32) {
        // let old_x = self.pointer_x;
        // let old_y = self.pointer_y;
        self.pointer_x = x;
        self.pointer_y = y;
    }

    pub fn pointer_down(&mut self, x: f32, y: f32) {
        self.pointer_x = x;
        self.pointer_y = y;
        self.pointer_down = true;
    }

    pub fn pointer_up(&mut self, x: f32, y: f32) {
        self.pointer_x = x;
        self.pointer_y = y;
        self.pointer_up = true;
    }

    pub fn scroll(&mut self, delta: f32) {
        self.scroll_delta = delta;
    }

    /*
    pub fn animate(&mut self, widget: &mut impl Widget) {
        self.animation_frame_requested = false;
        let elapsed = if let Some(last_time_stamp) = self.last_animation_timestamp {
            last_time_stamp.elapsed().as_secs_f32() * 1000.
        } else {
            0.
        };

        // Cap animation frames at 33 ms (30 fps) deltas.
        // This means that at lower framerates things will animate slower.
        // This also means that after a long period of time the animation will jump forward.
        // This probably requires a better solution.
        let elapsed = elapsed.min(33.);
        widget.event(self, UIEvent::AnimationFrame(elapsed));
        self.last_animation_timestamp = Some(std::time::Instant::now());
    }
    */

    pub fn pointer_position(&self) -> (f32, f32) {
        (self.pointer_x, self.pointer_y)
    }

    pub fn needs_redraw(&self) -> bool {
        self.animation_frame_requested
    }

    pub fn request_animation_frame(&mut self) {
        self.animation_frame_requested = true;
    }
}

use std::cell::RefCell;
use std::rc::Rc;

/// A UIBuilder is used to construct UI and query the UI.
#[derive(Clone)]
pub struct UIBuilder<'a> {
    ui: Rc<RefCell<&'a mut UI>>,
    parent: Option<NodeHandle>,
}

impl<'a> UIBuilder<'a> {
    pub fn add(&self, element_type: ElementType) -> Self {
        let new_container = self
            .ui
            .borrow_mut()
            .current_ui_tree
            .add(element_type, self.parent);
        UIBuilder {
            ui: self.ui.clone(),
            parent: Some(new_container),
        }
    }

    pub fn handle(&self) -> NodeHandle {
        self.parent.unwrap()
    }

    pub fn row(&self) -> Self {
        self.add(ElementType::Row(0.))
    }

    /// The spacing value specifies spacing between elements.
    /// Use an empty width container to spacing at the start or end.
    pub fn spaced_row(&self, spacing: f32) -> Self {
        self.add(ElementType::Row(spacing))
    }

    pub fn column(&self) -> Self {
        self.add(ElementType::Column(0.))
    }

    /// The spacing value specifies spacing between elements.
    /// Use an empty height container to spacing at the start or end.
    pub fn spaced_column(&self, spacing: f32) -> Self {
        self.add(ElementType::Column(spacing))
    }

    /// The spacing value specifies spacing between elements.
    /// Use an empty height container to spacing at the start or end.
    pub fn reverse_row(&self) -> Self {
        self.add(ElementType::ReverseRow(0.))
    }

    /// The spacing value specifies spacing between elements.
    /// Use an empty height container to spacing at the start or end.
    pub fn spaced_reverse_row(&self, spacing: f32) -> Self {
        self.add(ElementType::ReverseRow(spacing))
    }

    pub fn expander(&self) -> Self {
        self.add(ElementType::Expander)
    }

    pub fn horizontal_expander(&self) -> Self {
        self.add(ElementType::ExpanderHorizontal)
    }

    pub fn vertical_expander(&self) -> Self {
        self.add(ElementType::ExpanderVertical)
    }

    pub fn width(&self, width_pixels: f32) -> Self {
        self.add(ElementType::Width(width_pixels))
    }

    /// Percentage of parent
    pub fn width_percentage(&self, width_percentage: f32) -> Self {
        self.add(ElementType::WidthPercentage(width_percentage))
    }

    pub fn padding(&self, padding: f32) -> Self {
        self.add(ElementType::Padding(padding, padding))
    }

    pub fn padding_horizontal(&self, padding: f32) -> Self {
        self.add(ElementType::Padding(padding, 0.))
    }

    pub fn padding_vertical(&self, padding: f32) -> Self {
        self.add(ElementType::Padding(0., padding))
    }

    pub fn height(&self, height_pixels: f32) -> Self {
        self.add(ElementType::Height(height_pixels))
    }

    /// Draw a rectangle that fills the entire available space
    pub fn fill(&self, color: (f32, f32, f32, f32)) -> Self {
        self.add(ElementType::Fill(color))
    }

    /// Draw a rectangle that fills the entire available space
    pub fn rounded_fill(&self, color: (f32, f32, f32, f32), radius: f32) -> Self {
        self.add(ElementType::RoundedFill(
            (radius, radius, radius, radius),
            color,
        ))
    }

    pub fn center_vertical(&self) -> Self {
        self.add(ElementType::CenterVertical)
    }

    pub fn text(&self, text: &str) -> Self {
        self.add(ElementType::Text(text.to_owned()))
    }

    pub fn text_size(&self, size: f32) -> Self {
        self.add(ElementType::TextSize(size))
    }

    pub fn font(&self, font: FontHandle) -> Self {
        self.add(ElementType::Font(font))
    }

    /// Fits to children but can grow larger than the parent.
    pub fn fit(&self) -> Self {
        self.add(ElementType::Fit)
    }

    /// Fits to children without growing larger than the parent.
    pub fn flexible(&self) -> Self {
        self.add(ElementType::Flexible)
    }

    pub fn position_horizontal_percentage(&self, percentage: f32) -> Self {
        self.add(ElementType::PositionHorizontalPercentage(percentage))
    }

    pub fn position_horizontal_pixels(&self, pixels: f32) -> Self {
        self.add(ElementType::PositionHorizontalPixels(pixels))
    }

    pub fn position_vertical_pixels(&self, pixels: f32) -> Self {
        self.add(ElementType::PositionVerticalPixels(pixels))
    }

    pub fn scrollbar_vertical(&self, content: ElementHandle, view: ElementHandle) -> Self {
        self.add(ElementType::ScrollbarVertical(content, view))
    }

    pub fn custom_draw(&self, widget_handle: WidgetHandle) -> Self {
        self.add(ElementType::CustomRender(widget_handle))
    }

    /// This gets an existing widget if cached with the right ID.
    /// It must be paired with a call to "add_widget" to add it back to the tree.
    /// This will never deallocate an item for a ID, so if many unique IDs are
    /// produced memory will increase forever.
    pub fn get_widget<T: 'static + Any>(&self, id: u64) -> (WidgetHandle, Option<Box<T>>) {
        let mut ui = self.ui.borrow_mut();
        if let Some(index) = ui.widget_id_to_index.get(&id).copied() {
            if let Some(widget) = ui.widgets[index].take() {
                if let Ok(widget) = widget.to_any().downcast::<T>() {
                    return (WidgetHandle(index), Some(widget));
                }
            }
        }
        (WidgetHandle(ui.widgets.len()), None)
    }

    /// Adds a widget to the UI that is associated with an ID.
    pub fn add_widget<T: 'static + Widget>(&self, id: u64, widget: Box<T>) -> WidgetHandle {
        let mut ui = self.ui.borrow_mut();
        if let Some(index) = ui.widget_id_to_index.get(&id).copied() {
            ui.widgets[index] = Some(widget);
            WidgetHandle(index)
        } else {
            let index = ui.widgets.len();
            ui.widgets.push(Some(widget));
            ui.widget_id_to_index.insert(id, index);
            WidgetHandle(index)
        }
    }

    /// Code to query about input state.
    pub fn pointer_position(&self) -> (f32, f32) {
        let ui = self.ui.borrow();
        (ui.pointer_x, ui.pointer_y)
    }

    pub fn scroll_delta(&self) -> f32 {
        self.ui.borrow().scroll_delta
    }

    pub fn pointer_in_element(&self, element: ElementHandle) -> bool {
        let ui = self.ui.borrow();
        ui.old_ui_tree.elements[element.0]
            .rectangle
            .contains(ui.pointer_x, ui.pointer_y)
    }

    pub fn pointer_down(&self) -> bool {
        self.ui.borrow().pointer_down
    }

    pub fn pointer_up(&self) -> bool {
        self.ui.borrow().pointer_up
    }

    pub fn element_rectangle(&self, element: ElementHandle) -> Rectangle {
        self.ui.borrow().old_ui_tree.elements[element.0].rectangle
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WidgetHandle(pub(crate) usize);

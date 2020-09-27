mod gl_drawer;

use crate::UI;
use glow::HasContext;
pub use kapp::*;

pub struct SimpleUI {
    pub ui: UI,
    application: Application,
    events: Events,
    window: Window,
    gl_context: GLContext,
    gl: glow::Context,
    gl_drawer: gl_drawer::GLDrawer,
}

impl SimpleUI {
    pub fn edit(&mut self) -> crate::UIBuilder {
        self.ui.edit()
    }

    pub fn new(app: Application, events: Events) -> Self {
        let window_width = 1200;
        let window_height = 800;

        let window = app
            .new_window()
            .title("Hello")
            .size(window_width, window_height)
            .build()
            .unwrap();

        // Create a GLContext
        let mut gl_context = GLContext::new().samples(4).build().unwrap();
        gl_context.set_window(Some(&window)).unwrap();

        #[cfg(target_arch = "wasm32")]
        let gl =
            unsafe { glow::Context::from_webgl2_context(gl_context.webgl2_context().unwrap()) };
        #[cfg(not(target_arch = "wasm32"))]
        let gl = unsafe { glow::Context::from_loader_function(|s| gl_context.get_proc_address(s)) };

        unsafe {
            gl.viewport(0, 0, window_width as i32, window_height as i32);
        }
        let gl_drawer = gl_drawer::GLDrawer::new(&gl);

        window.request_redraw();

        let mut ui = UI::new();

        ui.resize(window_width as f32, window_height as f32);

        // Move all initial setup to here.
        Self {
            ui,
            application: app,
            events,
            gl_context,
            window,
            gl,
            gl_drawer,
        }
    }

    pub async fn update(&mut self) {
        // Perform drawing here assuming that a break occurred at a draw previously.
        let drawing_info = self.ui.render();
        self.gl_drawer.draw(&self.gl, &drawing_info);

        if self.ui.needs_redraw() {
            self.window.request_redraw();
        }
        self.gl_context.swap_buffers();

        loop {
            match self.events.next().await {
                Event::WindowCloseRequested { .. } => self.application.quit(),
                Event::WindowResized { width, height, .. } => {
                    self.gl_context.resize();
                    unsafe {
                        self.gl_context.resize(); // Resizes the window buffer
                        self.gl.viewport(0, 0, width as i32, height as i32);
                    }
                    self.ui.resize(width as f32, height as f32);
                    self.window.request_redraw();
                }
                Event::PointerMoved { x, y, .. } => {
                    self.ui.pointer_move(x as f32, y as f32);
                    self.window.request_redraw();
                }
                Event::PointerDown { x, y, .. } => {
                    self.ui.pointer_down(x as f32, y as f32);
                    self.window.request_redraw();
                }
                Event::PointerUp { x, y, .. } => {
                    self.ui.pointer_up(x as f32, y as f32);
                    self.window.request_redraw();
                }
                Event::Scroll { delta_y, .. } => {
                    self.ui.scroll(delta_y as f32);
                    self.window.request_redraw();
                }
                Event::Draw { .. } => {
                    return;
                }
                _ => {}
            }
        }
    }
}

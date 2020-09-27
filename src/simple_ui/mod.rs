mod gl_drawer;

use crate::UI;
use glow::HasContext;
use kapp::*;
use std::cell::{RefCell, RefMut};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};

pub fn run<F>(title: &str, run_function: impl Fn(UIContext) -> F)
where
    F: 'static + Future<Output = ()>,
{
    let (app, event_loop) = initialize();
    let mut window_width = 1200;
    let mut window_height = 800;
    let window = app
        .new_window()
        .title(title)
        .size(window_width, window_height)
        .build()
        .unwrap();

    // Create a GLContext
    let mut gl_context = GLContext::new().samples(4).build().unwrap();
    gl_context.set_window(Some(&window)).unwrap();

    #[cfg(target_arch = "wasm32")]
    let gl = glow::Context::from_webgl2_context(gl_context.webgl2_context().unwrap());
    #[cfg(not(target_arch = "wasm32"))]
    let gl = glow::Context::from_loader_function(|s| gl_context.get_proc_address(s));

    unsafe {
        gl.viewport(0, 0, window_width as i32, window_height as i32);
    }

    let ui_context = UIContext::new();

    // Run user function for the first time
    let mut user_future = Box::pin(run_function(ui_context.clone()));

    /*
    let inter_medium = ui_context
        .get_ui()
        .font_from_bytes(include_bytes!("../../resources/Inter-Medium.ttf"));
        */
    ui_context
        .get_ui()
        .resize(window_width as f32, window_height as f32);

    let mut gl_drawer = gl_drawer::GLDrawer::new(&gl);

    event_loop.run(move |event| match event {
        Event::WindowCloseRequested { .. } => app.quit(),
        Event::WindowResized { width, height, .. } => {
            gl_context.resize();
            unsafe {
                gl_context.resize(); // Resizes the window buffer
                gl.viewport(0, 0, width as i32, height as i32);
            }
            ui_context.get_ui().resize(width as f32, height as f32);
            window.request_redraw();
        }
        Event::PointerMoved { x, y, .. } => {
            ui_context.get_ui().pointer_move(x as f32, y as f32);
            window.request_redraw();
        }
        Event::PointerDown { x, y, .. } => {
            ui_context.get_ui().pointer_down(x as f32, y as f32);
            window.request_redraw();
        }
        Event::PointerUp { x, y, .. } => {
            ui_context.get_ui().pointer_up(x as f32, y as f32);
            window.request_redraw();
        }
        Event::Scroll { delta_y, .. } => {
            ui_context.get_ui().scroll(delta_y as f32);
            window.request_redraw();
        }
        Event::Draw { .. } => {
            // ----------- Build UI ---------------

            // Call into user async function giving it a chance to update.
            let waker = empty_waker::create();
            let mut context = Context::from_waker(&waker);
            *ui_context.ready.borrow_mut() = true;
            let _ = user_future.as_mut().poll(&mut context);

            {
                let mut ui_borrow = ui_context.get_ui();
                let drawables = ui_borrow.render();
                gl_drawer.draw(&gl, &drawables);
            }
            if ui_context.get_ui().needs_redraw() {
                window.request_redraw();
            }

            gl_context.swap_buffers();
        }
        _ => {}
    });
}

#[derive(Clone)]
pub struct UIContext {
    ui: Rc<RefCell<UI>>,
    ready: Rc<RefCell<bool>>,
}

impl UIContext {
    pub fn new() -> Self {
        Self {
            ui: Rc::new(RefCell::new(UI::new())),
            ready: Rc::new(RefCell::new(false)),
        }
    }

    pub fn next(&self) -> self::NextFrameFuture {
        self::NextFrameFuture {
            ui_context: self,
            phantom: std::marker::PhantomData,
        }
    }

    pub(crate) fn get_ui(&self) -> RefMut<UI> {
        (*self.ui).borrow_mut()
    }
}

pub struct NextFrameFuture<'a, 'b> {
    ui_context: &'a UIContext,
    phantom: std::marker::PhantomData<&'b ()>,
}

impl<'a: 'b, 'b> Future for NextFrameFuture<'a, 'b> {
    type Output = RefMut<'b, UI>;

    fn poll(mut self: Pin<&mut Self>, _ctx: &mut Context) -> Poll<Self::Output> {
        if *self.ui_context.ready.borrow() {
            *self.ui_context.ready.borrow_mut() = false;
            Poll::Ready((*self.ui_context.ui).borrow_mut())
        } else {
            Poll::Pending
        }
    }
}

/// This should be expanded in the future in case the user wants to wait on other
/// async events.
mod empty_waker {
    use std::task::{RawWaker, RawWakerVTable, Waker};

    pub fn create() -> Waker {
        unsafe { Waker::from_raw(RAW_WAKER) }
    }

    const RAW_WAKER: RawWaker = RawWaker::new(std::ptr::null(), &VTABLE);
    const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

    unsafe fn clone(_: *const ()) -> RawWaker {
        RAW_WAKER
    }
    unsafe fn wake(_: *const ()) {}
    unsafe fn wake_by_ref(_: *const ()) {}
    unsafe fn drop(_: *const ()) {}
}

use glow::*;
use kapp::*;
use kui::*;

fn main() {
    let (app, event_loop) = initialize();
    let mut window_width = 800;
    let mut window_height = 800;
    let window = app
        .new_window()
        .dimensions(window_width, window_height)
        .build()
        .unwrap();
    println!("Hello, world!");

    // Create a GLContext
    let mut gl_context = GLContext::new().build().unwrap();
    gl_context.set_window(Some(&window)).unwrap();

    #[cfg(target_arch = "wasm32")]
    let gl = glow::Context::from_webgl2_context(gl_context.webgl2_context().unwrap());
    #[cfg(not(target_arch = "wasm32"))]
    let gl = glow::Context::from_loader_function(|s| gl_context.get_proc_address(s));

    unsafe {
        gl.enable(SCISSOR_TEST);
        gl.viewport(0, 0, 800, 800);
    }

    let mut ui = UI::new();
    ui.resize(window_width as f32, window_height as f32);

    let red = (1.0, 0.0, 0.0, 1.0);
    let blue = (0.0, 0.0, 1.0, 1.0);
    let light_gray0 = (0.6, 0.6, 0.6, 1.0);
    let light_gray1 = (0.5, 0.5, 0.5, 1.0);

    let gray = (0.28, 0.28, 0.28, 1.0);

    let body = ui.edit();
    // body.row().fill(red).padding(20.);
    let column = body.column();

    let nav = column.height(100.).fill(gray); //.evenly_spaced_row();

    let nav_left = nav.row();
    for _ in 0..3 {
        let button = nav_left.padding(20.).row();
        button.width(60.).fill(red); // Icon
        button.width(200.).fill(blue);
    }

    let nav_right = nav.reverse_row();
    for _ in 0..3 {
        let button = nav_right.padding(20.).row();
        button.width(60.).fill(red); // Icon
        button.width(200.).fill(blue); // Icon
    }

    let content = column.column();
    for _ in 0..30 {
        content.height(50.).padding(5.).fill(light_gray0);
        content.height(50.).padding(5.).fill(light_gray1);
    }

    event_loop.run(move |event| match event {
        Event::WindowCloseRequested { .. } => app.quit(),
        Event::WindowResized { width, height, .. } => {
            window_width = width;
            window_height = height;
            gl_context.resize();
            unsafe {
                gl_context.resize(); // Resizes the window buffer
                gl.viewport(0, 0, width as i32, height as i32);
            }
            ui.resize(width as f32, height as f32);
            window.request_redraw();
        }
        Event::MouseMoved { x, y, .. } => {}
        Event::Draw { .. } => unsafe {
            gl.scissor(0, 0, window_width as i32, window_height as i32);
            gl.clear_color(0.945, 0.945, 0.945, 1.0);
            gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);

            let drawables = ui.render();
            for drawable in drawables {
                let width = (drawable.rectangle.2) as i32;
                let height = (drawable.rectangle.3) as i32;

                let x = (drawable.rectangle.0) as i32;
                let y = (window_height as f32 - drawable.rectangle.1 - drawable.rectangle.3) as i32;

                gl.scissor(x, y, width, height);

                gl.clear_color(
                    drawable.color.0,
                    drawable.color.1,
                    drawable.color.2,
                    drawable.color.3,
                );

                gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
            }

            gl_context.swap_buffers();
        },
        _ => {}
    });
}

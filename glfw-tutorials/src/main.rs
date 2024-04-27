fn main() {
    use glfw::{Context, SwapInterval, WindowHint, WindowMode};

    let mut glfw = glfw::init(glfw::fail_on_errors).unwrap();
    #[cfg(feature = "opengl")]
    glfw.window_hint(WindowHint::ContextVersion(3, 3));
    let (mut window, event_queue) = glfw
        .create_window(480, 320, "glfw tutorial", WindowMode::Windowed)
        .unwrap();

    window.make_current();
    #[cfg(feature = "opengl")]
    gl::load_with(|symbol| window.get_proc_address(symbol).cast());
    glfw.set_swap_interval(SwapInterval::Sync(1));
    if glfw.extension_supported("GL_ARB_gl_spirv") {
        println!("Extension 'GL_ARB_gl_spirv' is supported.");
    } else {
        println!("Extension 'GL_ARB_gl_spirv' is not supported.");
    }

    let mut i = 0;
    window.set_all_polling(true);
    while !window.should_close() {
        glfw.poll_events();
        for (_, event) in glfw::flush_messages(&event_queue) {
            use glfw::{Action, Key, WindowEvent};

            match event {
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    window.set_should_close(true);
                }
                _ => {
                    println!("{:?}", event);
                }
            }
        }

        // my_app.update();
        // my_app.render();
        #[cfg(feature = "opengl")]
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
        window.swap_buffers();

        if i % 60 == 0 {
            println!("loop {}", i as f32 / 60.0);
        }
        i += 1;
    }
}

use std::time::Duration;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    #[cfg(not(feature = "opengl"))]
    let window = video_subsystem
        .window("sdl2 tutorial", 480, 320)
        .position_centered()
        .build()
        .unwrap();
    #[cfg(feature = "opengl")]
    {
        use sdl2::video::GLProfile;

        let gl_attr = video_subsystem.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 3);
    }
    #[cfg(feature = "opengl")]
    let window = video_subsystem
        .window("sdl2 tutorial", 480, 320)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    #[cfg(feature = "opengl")]
    let gl_context = {
        use sdl2::video::SwapInterval;

        let gl_context = window.gl_create_context().unwrap();
        window.gl_make_current(&gl_context).unwrap();
        video_subsystem
            .gl_set_swap_interval(SwapInterval::VSync)
            .unwrap();
        gl::load_with(|symbol| video_subsystem.gl_get_proc_address(symbol) as *const _);
        gl_context
    };
    // debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
    // debug_assert_eq!(gl_attr.context_version(), (3, 3));
    let mut event_queue = sdl_context.event_pump().unwrap();

    let mut my_app = MyApp::new();
    #[cfg(not(feature = "opengl"))]
    let mut canvas = window.into_canvas().build().unwrap();
    #[cfg(feature = "gfx")]
    let mut fps_manager = {
        use sdl2::gfx::framerate::FPSManager;

        let mut fps_manager = FPSManager::new();
        fps_manager.set_framerate(60).unwrap();
        fps_manager
    };
    let mut i = 0;
    'running: loop {
        for event in event_queue.poll_iter() {
            use sdl2::{event::Event, keyboard::Keycode};

            match event {
                Event::Window { win_event, .. } => match win_event {
                    _ => {
                        println!("{:?}", event);
                    }
                },
                Event::MouseMotion { .. }
                | Event::MouseButtonDown { .. }
                | Event::MouseButtonUp { .. } => {
                    println!("{:?}", event);
                }
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        #[cfg(not(feature = "opengl"))]
        {
            my_app.update();
            my_app.render(&mut canvas);
            #[cfg(not(feature = "gfx"))]
            ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            #[cfg(feature = "gfx")]
            fps_manager.delay();
        }
        #[cfg(feature = "opengl")]
        {
            my_app.update();
            my_app.render();
            window.gl_swap_window();
        }

        if i % 60 == 0 {
            println!("loop {}", i as f32 / 60.0);
        }
        i += 1;
    }
}

struct MyApp {}

use sdl2::render::WindowCanvas;

impl MyApp {
    fn new() -> Self {
        MyApp {}
    }

    fn update(&mut self) {}

    #[cfg(feature = "opengl")]
    fn render(&mut self) {
        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    #[cfg(not(feature = "opengl"))]
    fn render(&mut self, canvas: &mut WindowCanvas) {
        use sdl2::gfx::primitives::DrawRenderer;
        use sdl2::{pixels::Color, rect::Rect};

        canvas.set_draw_color(Color::RGB(48, 48, 48));
        canvas.clear();
        let colors = [Color::RED, Color::GREEN, Color::BLUE];
        [100, 300, 500]
            .iter()
            .zip(colors.iter())
            .for_each(|(x, c)| {
                #[cfg(not(feature = "gfx"))]
                {
                    canvas.set_draw_color(*c);
                    canvas
                        .fill_rect(Rect::new(*x - 90, 100 - 90, 180, 180))
                        .unwrap();
                }
                #[cfg(feature = "gfx")]
                canvas.filled_circle(*x, 100, 100, *c).unwrap();
            });

        #[cfg(feature = "gfx")]
        canvas.string(1, 1, "SDL2 Tutorial", Color::WHITE).unwrap();

        canvas.present();
    }
}

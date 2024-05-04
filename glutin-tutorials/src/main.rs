use std::ffi::CString;
use std::num::NonZeroU32;

use raw_window_handle::HasRawWindowHandle;

fn main() {
    use glutin::config::ConfigTemplateBuilder;
    use glutin::prelude::*;
    use glutin_winit::DisplayBuilder;
    use winit::dpi::PhysicalSize;
    use winit::event_loop::EventLoop;
    use winit::window::WindowBuilder;

    let event_queue = EventLoop::new().unwrap();
    let window_builder = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(480.0, 320.0))
        .with_title("glutin tutorial");
    let template = ConfigTemplateBuilder::new().prefer_hardware_accelerated(Some(true));
    let (window, gl_config) = DisplayBuilder::new()
        .with_window_builder(Some(window_builder))
        .build(&event_queue, template, |configs| {
            configs
                .reduce(|prev_config, config| {
                    let transparency_check = config.supports_transparency().unwrap_or(false)
                        & !prev_config.supports_transparency().unwrap_or(false);
                    if transparency_check || config.num_samples() > prev_config.num_samples() {
                        config
                    } else {
                        prev_config
                    }
                })
                .unwrap()
        })
        .unwrap();
    let mut window = window.unwrap();
    println!("gl_config: {:?}", gl_config);
    println!("gl_config.api(): {:?}", gl_config.api());

    use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
    use glutin::display::GetGlDisplay;

    let gl_display = gl_config.display();
    let raw_window_handle = Some(window.raw_window_handle());
    let context_attributes = ContextAttributesBuilder::new().build(raw_window_handle);
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(raw_window_handle);
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
        .build(raw_window_handle);
    let mut gl_context_not_current = unsafe {
        gl_display
            .create_context(&gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(&gl_config, &fallback_context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(&gl_config, &legacy_context_attributes)
                            .expect("failed to create context")
                    })
            })
    };
    println!("gl_display: {:?}", gl_display);
    println!(
        "gl_display.version_string(): {:?}",
        gl_display.version_string()
    );

    use glutin::surface::SwapInterval;
    use glutin_winit::GlWindow;

    let attrs = window.build_surface_attributes(Default::default());
    let mut gl_surface = unsafe {
        gl_display
            .create_window_surface(&gl_config, &attrs)
            .unwrap()
    };
    let mut gl_context = gl_context_not_current.make_current(&gl_surface).unwrap();
    gl_surface
        .set_swap_interval(&gl_context, SwapInterval::Wait(NonZeroU32::new(1).unwrap()))
        .unwrap();
    gl::load_with(|symbol| {
        let symbol = CString::new(symbol).unwrap();
        gl_display.get_proc_address(symbol.as_c_str()).cast()
    });
    println!("gl_surface: {:?}", gl_surface);
    println!("gl_context: {:?}", gl_context);
    println!("gl_context.context_api(): {:?}", gl_context.context_api());

    use winit::event::{Event, KeyEvent, WindowEvent};
    use winit::keyboard::{Key, NamedKey};

    let mut i = 0;
    _ = event_queue.run(move |event, window_target| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                window_target.exit();
            }
            WindowEvent::Resized(size) => {
                if size.width != 0 && size.height != 0 {
                    gl_surface.resize(
                        &gl_context,
                        NonZeroU32::new(size.width).unwrap(),
                        NonZeroU32::new(size.height).unwrap(),
                    );
                    // my_app.resize(size.width as i32, size.height as i32);
                }
            }
            WindowEvent::RedrawRequested => {
                // my_app.update();
                // my_app.render();
                // unsafe {
                //     gl::ClearColor(0.2, 0.2, 0.2, 1.0);
                //     gl::Clear(gl::COLOR_BUFFER_BIT);
                // }
                gl_surface.swap_buffers(&gl_context).unwrap();

                if i % 60 == 0 {
                    println!("loop {}", i as f32 / 60.0);
                }
                i += 1;
            }
            _ => {
                println!("{:?}", event);
            }
        },
        Event::Resumed | Event::Suspended => {
            println!("{:?}", event);
        }
        Event::AboutToWait => {
            window.request_redraw();
        }
        Event::LoopExiting => return,
        Event::NewEvents(_) => {}
        _ => {
            println!("{:?}", event);
        }
    });
}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    use winit::dpi::PhysicalSize;
    use winit::event_loop::{ControlFlow, EventLoop};
    use winit::window::WindowBuilder;

    let event_queue = EventLoop::new().unwrap();
    event_queue.set_control_flow(ControlFlow::Poll);
    let window = WindowBuilder::new()
        .with_title("winit tutorial")
        .with_inner_size(PhysicalSize {
            width: 480,
            height: 320,
        })
        .build(&event_queue)
        .unwrap();

    #[cfg(target_arch = "wasm32")]
    {
        // use winit::dpi::PhysicalSize;
        // let _ = window.request_inner_size(PhysicalSize::new(480, 320));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    use winit::event::{DeviceEvent, ElementState, Event, KeyEvent, WindowEvent};
    use winit::keyboard::{KeyCode, PhysicalKey};

    _ = event_queue.run(move |event, window_target| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => {
                window_target.exit();
            }
            WindowEvent::CursorMoved { .. } | WindowEvent::MouseInput { .. } => {
                println!("{:?}", event);
            }
            WindowEvent::RedrawRequested => {
                // my_app.update();
                // my_app.render();
            }
            _ => {
                println!("{:?}", event);
            }
        },
        Event::DeviceEvent { event, .. } => match event {
            DeviceEvent::Added | DeviceEvent::Removed => {
                println!("{:?}", event);
            }
            _ => {}
        },
        Event::AboutToWait => {
            window.request_redraw();
        }
        Event::Suspended | Event::Resumed => {
            println!("{:?}", event);
        }
        _ => {}
    });
}

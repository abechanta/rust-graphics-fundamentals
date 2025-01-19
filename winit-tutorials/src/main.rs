use std::rc::Rc;
use winit::window::{Window, WindowAttributes};

fn main() {
    use winit::dpi::PhysicalSize;
    use winit::event_loop::{ControlFlow, EventLoop};

    let event_queue = EventLoop::new().unwrap();
    event_queue.set_control_flow(ControlFlow::Poll);
    let window_attributes = WindowAttributes::default()
        .with_title("winit tutorial")
        .with_inner_size(PhysicalSize {
            width: 480,
            height: 320,
        });

    let mut my_app = MyApp {
        window_attributes,
        ..Default::default()
    };
    _ = event_queue.run_app(&mut my_app);
}

#[derive(Default)]
struct MyApp {
    window_attributes: WindowAttributes,
    window: Option<Rc<Window>>,
}

use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowId;

impl ApplicationHandler for MyApp {
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(window) = &self.window {
            window.request_redraw();
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = {
            let window = event_loop
                .create_window(self.window_attributes.clone())
                .unwrap();
            Rc::new(window)
        };
        self.window = Some(window);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        println!("suspended");
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::Added | DeviceEvent::Removed => {
                println!("{:?}", event);
            }
            _ => {}
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        use winit::event::{ElementState, KeyEvent};
        use winit::keyboard::{KeyCode, PhysicalKey};

        match event {
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
                event_loop.exit();
            }
            WindowEvent::CursorMoved { .. } | WindowEvent::MouseInput { .. } => {
                println!("{:?}", event);
            }
            WindowEvent::Resized(_) => {
                println!("{:?}", event);
            }
            WindowEvent::RedrawRequested => {
                // self.update();
                // self.render();
            }
            _ => {
                println!("{:?}", event);
            }
        }
    }
}

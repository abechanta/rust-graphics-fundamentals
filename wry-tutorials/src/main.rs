use std::rc::Rc;
use tao::event::{ElementState, KeyEvent, MouseButton, RawKeyEvent};

fn main() {
    use tao::dpi::PhysicalSize;
    use tao::event_loop::EventLoop;
    use tao::window::WindowBuilder;

    let event_queue = EventLoop::new();

    let window = WindowBuilder::new()
        .with_title("wry tutorial")
        .with_inner_size(PhysicalSize::new(480, 320))
        .build(&event_queue)
        .unwrap();

    use wry::WebViewBuilder;

    let webview = WebViewBuilder::new()
        .with_html(MY_HTML_SOURCE)
        // .with_url("https://tauri.app/")
        .build(&window)
        .unwrap();

    let mut window = Some(window);
    let mut webview = Some(webview);
    let mut visible = true;
    event_queue.run(move |event, _, control_flow| {
        use tao::event::{DeviceEvent, Event, StartCause, WindowEvent};
        use tao::event_loop::ControlFlow;
        use tao::keyboard::KeyCode;

        *control_flow = ControlFlow::Wait;

        match event {
            Event::NewEvents(StartCause::Init) => {
                println!("Wry has started!");
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | Event::DeviceEvent {
                event:
                    DeviceEvent::Key(
                        RawKeyEvent {
                            physical_key: KeyCode::Escape,
                            ..
                        },
                        ..,
                    ),
                ..
            } => {
                println!("{:?}", event);
                // drop the window to fire the `Destroyed` event
                window = None;
            }
            Event::WindowEvent {
                event: WindowEvent::Destroyed,
                ..
            } => {
                println!("{:?}", event);
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                if let Some(window) = &window {
                    window.request_redraw();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::DecorationsClick,
                ..
            } => {
                println!("{:?}", event);
                if let Some(webview) = &webview {
                    visible = !visible;
                    webview.set_visible(visible).unwrap();
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { .. },
                ..
            }
            | Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                println!("{:?}", event);
            }
            Event::Suspended | Event::Resumed => {
                println!("{:?}", event);
            }
            Event::RedrawRequested(_) => {
                // self.update();
                // self.render();
            }
            _ => {
                println!("{:?}", event);
            }
        }
    });
}

static MY_HTML_SOURCE: &str = "
<html>
    <body>
        <h1>It works!</h1>
    </body>
</html>
";

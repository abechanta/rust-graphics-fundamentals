use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

fn main() {
    let event_queue = EventLoop::new().unwrap();
    event_queue.set_control_flow(ControlFlow::Poll);
    let window = WindowBuilder::new()
        .with_title("pixels tutorial")
        .with_inner_size(PhysicalSize {
            width: 480,
            height: 320,
        })
        .build(&event_queue)
        .unwrap();
    let mut input = WinitInputHelper::new();

    use pixels::wgpu::{BlendState, Color};
    use pixels::{PixelsBuilder, SurfaceTexture};

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        PixelsBuilder::new(window_size.width, window_size.height, surface_texture)
            .blend_state(BlendState::ALPHA_BLENDING)
            .clear_color(Color {
                r: 0.2,
                g: 0.2,
                b: 0.2,
                a: 1.0,
            })
            .build()
            .unwrap()
    };

    // let mut pause = false;
    let mut my_app = MyApp::new();
    _ = event_queue.run(move |event, window_target| {
        if input.update(&event) {
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                window_target.exit();
                return;
            }

            if let Some(size) = input.window_resized() {
                println!("window resized: {:?}", size);
                if let Err(_err) = pixels.resize_surface(size.width, size.height) {
                    window_target.exit();
                    return;
                }
            }

            my_app.update();
            // if input.key_pressed(KeyCode::Enter) {
            //     pause = !pause;
            // }
            // if !pause {
            //     my_app.update();
            // }
            window.request_redraw();
        }

        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            my_app.render(pixels.frame_mut());
            if let Err(_err) = pixels.render() {
                window_target.exit();
                return;
            }
        }
    });
}

const WIDTH: u32 = 480;
const HEIGHT: u32 = 320;

struct MyApp {
    box_size: i16,
    box_x: i16,
    box_y: i16,
    velocity_x: i16,
    velocity_y: i16,
}

impl MyApp {
    fn new() -> Self {
        Self {
            box_size: 32,
            box_x: 16,
            box_y: 16,
            velocity_x: 1,
            velocity_y: 1,
        }
    }

    fn update(&mut self) {
        if self.box_x <= 0 || self.box_x + self.box_size >= WIDTH as i16 {
            self.velocity_x *= -1;
        }
        if self.box_y <= 0 || self.box_y + self.box_size >= HEIGHT as i16 {
            self.velocity_y *= -1;
        }

        self.box_x += self.velocity_x;
        self.box_y += self.velocity_y;
    }

    fn render(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = (i % WIDTH as usize) as i16;
            let y = (i / WIDTH as usize) as i16;

            let edge_of_the_box = (x == self.box_x || x == self.box_x + self.box_size - 1)
                && (y >= self.box_y && y <= self.box_y + self.box_size - 1)
                || (x >= self.box_x && x <= self.box_x + self.box_size - 1)
                    && (y == self.box_y || y == self.box_y + self.box_size - 1);
            let inside_the_box = (x > self.box_x && x < self.box_x + self.box_size - 1)
                && (y > self.box_y && y < self.box_y + self.box_size - 1);

            let rgba = if inside_the_box {
                [0x5e, 0x48, 0xe8, 0xff]
            } else {
                [0x48, 0xb2, 0xe8, 0xff]
            };
            if edge_of_the_box || inside_the_box {
                pixel.copy_from_slice(&rgba);
            }
        }
    }
}

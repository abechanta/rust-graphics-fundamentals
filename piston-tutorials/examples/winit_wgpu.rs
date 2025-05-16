// use piston::{*, Events, EventSettings};
use std::{env, f64, path};

#[pollster::main]
async fn main() {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("../assets");
        path
    } else {
        path::PathBuf::from("../assets")
    };

    use piston::WindowSettings;
    use wgpu_graphics::{TextureContext, TextureSettings, Wgpu2d};
    use winit_window::WinitWindow;

    let mut window = WindowSettings::new("piston tutorial / winit+wgpu", [480, 320])
        .exit_on_esc(true)
        .vsync(true)
        .build::<WinitWindow>()
        .unwrap();

    let (device, queue, config, surface) = {
        //
        // size, surface, device, queue, config
        //
        let size = window.get_window().inner_size();
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });
        let surface = unsafe { instance.create_surface(&window.get_window()) }.unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::DEPTH_CLIP_CONTROL,
                    ..Default::default()
                },
                None,
            )
            .await
            .unwrap();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            // desired_maximum_frame_latency: 60,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        (device, queue, config, surface)
    };
    let mut wgpu2d = Wgpu2d::new(&device, &config);

    let glyph_cache = {
        let texture_context = TextureContext::from_parts(&device, &queue);
        let glyph_cache = GlyphCache::new(
            resource_dir.join("LiberationMono-Regular.ttf"),
            texture_context,
            TextureSettings::new(),
        )
        .unwrap();
        glyph_cache
    };

    let mut my_app = MyApp {
        text: "POS: ".into(),
        glyph_cache,
    };

    use piston::{Event, EventSettings, Events, Input, Loop, Motion};

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        match e {
            Event::Input(Input::Move(Motion::MouseCursor(args)), _) => {
                my_app.mouse_moved(&args);
            }
            Event::Loop(Loop::Update(args)) => {
                my_app.update(&args);
            }
            Event::Loop(Loop::Render(args)) => {
                let surface_texture = surface.get_current_texture().unwrap();
                let surface_view = surface_texture
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let command_buffer =
                    wgpu2d.draw(&device, &config, &surface_view, args.viewport(), |c, g| {
                        my_app.render(&args, &c, g);
                    });
                queue.submit(std::iter::once(command_buffer));
                surface_texture.present();
            }
            _ => {}
        }
    }
}

use graphics::{context::Context, Graphics};
use piston::{RenderArgs, UpdateArgs};
use wgpu_graphics::GlyphCache;

struct MyApp<'a> {
    text: String,
    glyph_cache: GlyphCache<'a>,
}

impl MyApp<'_> {
    fn mouse_moved(self: &mut Self, args: &[f64; 2]) {
        self.text = format!("POS: {:?}", args);
    }

    fn update(self: &mut Self, _args: &UpdateArgs) {}

    fn render<G>(self: &mut Self, _args: &RenderArgs, c: &Context, g: &mut G)
    where
        G: Graphics<Texture = wgpu_graphics::Texture>,
    {
        use graphics::{clear, Transformed};

        clear([0.2; 4], g);

        let mut transform = c.transform.trans(100., 100.);
        let colors = [
            [1., 0., 0., 1.],
            [0., 1., 0., 1.],
            [0., 0., 1., 1.],
        ];
        colors.iter().for_each(|color| {
            use graphics::Ellipse;

            Ellipse::new(*color).draw([-200., -200., 400., 400.], &c.draw_state, transform, g);
            transform = transform.trans(200., 0.);
        });

        use graphics::Text;

        let transform = c.transform.trans(0., 32.);
        Text::new_color([1.; 4], 24)
            .draw(
                &self.text,
                &mut self.glyph_cache,
                &c.draw_state,
                transform,
                g,
            )
            .unwrap();
    }
}

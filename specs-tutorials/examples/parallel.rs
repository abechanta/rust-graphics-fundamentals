use ggez::{graphics::Color, Context, GameResult};
use glam::*;
// use specs::prelude::*;
use std::{
    env, f32, path,
    time::{Duration, Instant},
};

pub fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("../assets");
        path
    } else {
        path::PathBuf::from("../assets")
    };

    use ggez::conf::{WindowMode, WindowSetup};

    let context_builder = ggez::ContextBuilder::new("appname", "yourname")
        .window_setup(WindowSetup {
            title: "specs tutorial with ggez".to_owned(),
            ..Default::default()
        })
        .window_mode(WindowMode {
            width: 480.,
            height: 320.,
            ..Default::default()
        })
        .add_resource_path(resource_dir);
    let (mut ggez_context, event_queue) = context_builder.build()?;

    let my_app = MyApp::new(&mut ggez_context)?;
    ggez::event::run(ggez_context, event_queue, my_app)
}

use specs::{Dispatcher, World, WorldExt};

struct MyApp<'a> {
    world: World,
    dispatcher: Dispatcher<'a, 'a>,
}

impl MyApp<'_> {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        use ggez::graphics::FontData;

        ctx.gfx.add_font(
            "LiberationMono",
            FontData::from_path(ctx, "/LiberationMono-Regular.ttf")?,
        );

        use specs::DispatcherBuilder;

        let mut world = World::new();
        world.register::<MyExplosion>();
        world.register::<MyBreakable>();
        world.register::<MyBomb>();
        world.register::<MyTransform>();
        world.insert(MyTime::new());
        world.insert(MyChains(0));
        let mut dispatcher = DispatcherBuilder::new()
            .with(MyExplosionSystem, "explosion_system", &[])
            .with(MyBreakableSystem, "breakable_system", &[])
            .with(MyChainExplosionSystem, "chain_explosion_system", &[])
            .build();
        dispatcher.setup(&mut world);

        let my_app = MyApp { world, dispatcher };
        Ok(my_app)
    }
}

use ggez::{event::EventHandler, GameError};

impl EventHandler<GameError> for MyApp<'_> {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(time) = self.world.get_mut::<MyTime>() {
            time.tick();
        }
        self.dispatcher.dispatch(&self.world);
        self.world.maintain();

        use ggez::event::MouseButton;
        use specs::Builder;

        if ctx.mouse.button_just_pressed(MouseButton::Left) {
            let point = ctx.mouse.position();
            println!("MouseLeft {:?}", point);

            self.world
                .create_entity()
                .with(MyExplosion::new(0))
                .with(MyTransform::new(&point.into(), 0.))
                .build();
        }
        if ctx.mouse.button_just_pressed(MouseButton::Middle) {
            let point = ctx.mouse.position();
            println!("MouseMiddle {:?}", point);

            self.world
                .create_entity()
                .with(MyBomb {})
                .with(MyBreakable::new())
                .with(MyTransform::new(&point.into(), 1.))
                .build();
        }
        if ctx.mouse.button_just_pressed(MouseButton::Right) {
            if let Some(time) = self.world.get_mut::<MyTime>() {
                if time.paused {
                    println!("MouseRight: resume");
                    time.resume();
                } else {
                    println!("MouseRight: pause");
                    time.pause();
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, Drawable, Mesh, Text};

        let mut canvas = Canvas::from_frame(ctx, Color::from([0.2, 0.2, 0.2, 1.]));

        use specs::Join;
        {
            let explosions = self.world.read_storage::<MyExplosion>();
            let transforms = self.world.read_storage::<MyTransform>();
            (&explosions, &transforms)
                .join()
                .for_each(|(explosion, transform)| {
                    Mesh::new_circle(
                        ctx,
                        DrawMode::fill(),
                        Vec2::new(0., 0.),
                        EXPLOSION_RADIUS,
                        1.,
                        MyExplosion::color(explosion.radius),
                    )
                    .unwrap()
                    .draw(
                        &mut canvas,
                        DrawParam::new()
                            .dest(transform.translation)
                            .scale(transform.scaling),
                    );
                });
        }

        {
            let bombs = self.world.read_storage::<MyBomb>();
            let transforms = self.world.read_storage::<MyTransform>();
            (&bombs, &transforms).join().for_each(|(_bomb, transform)| {
                Mesh::new_circle(
                    ctx,
                    DrawMode::fill(),
                    Vec2::new(0., 0.),
                    BOMB_RADIUS,
                    1.,
                    BOMB_COLOR,
                )
                .unwrap()
                .draw(
                    &mut canvas,
                    DrawParam::new()
                        .dest(transform.translation)
                        .scale(transform.scaling),
                );
            });
        }

        {
            let texts = [
                "Mouse L: Spawn Explosion",
                "Mouse M: Spawn Bomb",
                "Mouse R: Pause/Resume",
            ];
            texts.iter().enumerate().for_each(|(i, &t)| {
                Text::new(t.to_string())
                    .set_font("LiberationMono")
                    .set_scale(16.)
                    .draw(&mut canvas, Vec2::new(0., 16. * i as f32));
            });
        }

        {
            let chains = self.world.read_resource::<MyChains>();
            let nbchains = if chains.0 > 0 {
                format!("{} Chain(s)", chains.0)
            } else {
                "".into()
            };
            Text::new(nbchains.to_string())
                .set_font("LiberationMono")
                .set_scale(16.)
                .draw(&mut canvas, Vec2::new(0., 320. - 16.));
        }

        canvas.finish(ctx)?;
        Ok(())
    }
}

struct MyChains(u32);

impl Default for MyChains {
    fn default() -> Self {
        MyChains(0)
    }
}

struct MyTime {
    delta: Duration,
    elapsed: Duration,
    paused: bool,
    timer: Instant,
}

impl Default for MyTime {
    fn default() -> Self {
        MyTime::new()
    }
}

impl MyTime {
    fn new() -> Self {
        Self {
            delta: Duration::ZERO,
            elapsed: Duration::ZERO,
            paused: false,
            timer: Instant::now(),
        }
    }

    fn tick(&mut self) {
        self.delta = if self.paused {
            Duration::ZERO
        } else {
            self.timer.elapsed()
        };
        self.elapsed += self.delta;
        self.timer = Instant::now()
    }

    fn delta(&self) -> Duration {
        self.delta
    }

    fn elapsed(&self) -> Duration {
        self.elapsed
    }

    fn pause(&mut self) {
        self.paused = true;
    }

    fn resume(&mut self) {
        self.paused = false;
    }
}

struct MyTransform {
    translation: Vec2,
    scaling: Vec2,
}

use specs::{Component, VecStorage};

impl Component for MyTransform {
    // This uses `VecStorage`, because all entities have a position.
    type Storage = VecStorage<Self>;
}

impl MyTransform {
    fn new(point: &Vec2, size: f32) -> MyTransform {
        Self {
            translation: point.clone(),
            scaling: Vec2::splat(size),
        }
    }
}

struct MyExplosion {
    timer: Duration,
    radius: f32,
    chain_value: u32,
}

use specs::HashMapStorage;

impl Component for MyExplosion {
    // This uses `HashMapStorage`, because only some entities are explosions.
    type Storage = HashMapStorage<Self>;
}

const EXPLOSION_RADIUS: f32 = 40.;
const EXPLOSION_TIMER: f32 = 1.2;

impl MyExplosion {
    fn new(chain_value: u32) -> MyExplosion {
        Self {
            timer: Duration::from_secs_f32(EXPLOSION_TIMER),
            radius: 0.,
            chain_value,
        }
    }

    fn color(t: f32) -> Color {
        Color::new(1., 0.5 + t, 0.3 + t, 1.)
    }
}

use specs::{Entities, System, Write, WriteStorage};

struct MyExplosionSystem;

impl<'a> System<'a> for MyExplosionSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, MyExplosion>,
        WriteStorage<'a, MyTransform>,
        Write<'a, MyTime>,
        Write<'a, MyChains>,
    );

    fn run(
        &mut self,
        (entities, mut explosions, mut transforms, time, mut chains): Self::SystemData,
    ) {
        use specs::{prelude::ParallelIterator, ParJoin};

        (&entities, &mut explosions, &mut transforms)
            .par_join()
            .for_each(|(entity, explosion, transform)| {
                explosion.timer = explosion.timer.saturating_sub(time.delta());
                if explosion.timer == Duration::ZERO {
                    // 爆発おわり
                    println!("despawn {:?}", entity);
                    _ = entities.delete(entity);
                    return;
                }

                // 爆発している
                let t = Duration::min(
                    explosion.timer,
                    Duration::from_secs_f32(EXPLOSION_TIMER) - explosion.timer,
                )
                .as_secs_f32()
                    / EXPLOSION_TIMER
                    * 2.;
                explosion.radius = t;
                transform.scaling = Vec2::splat(t);
            });

        let chain_value_max = explosions
            .par_join()
            .map(|explosion| explosion.chain_value)
            .reduce(
                || 0,
                |chain_value1, chain_value2| u32::max(chain_value1, chain_value2),
            );
        chains.0 = chain_value_max;
    }
}

#[derive(Default)]
enum MyBreakableEvent {
    #[default]
    None,
    Damaged(u32),
}

struct MyBreakable {
    will_explode: bool,
    incoming: MyBreakableEvent,
}

impl Component for MyBreakable {
    // This uses `HashMapStorage`, because only some entities are breakables.
    type Storage = HashMapStorage<Self>;
}

impl MyBreakable {
    fn new() -> MyBreakable {
        Self {
            will_explode: true,
            incoming: MyBreakableEvent::None,
        }
    }
}

struct MyBomb;

const BOMB_RADIUS: f32 = 4.;
const BOMB_COLOR: Color = Color::CYAN;

impl Component for MyBomb {
    // This uses `HashMapStorage`, because only some entities are bombs.
    type Storage = HashMapStorage<Self>;
}

use specs::{LazyUpdate, Read, ReadStorage};

struct MyBreakableSystem;

impl<'a> System<'a> for MyBreakableSystem {
    type SystemData = (
        Read<'a, LazyUpdate>,
        Entities<'a>,
        ReadStorage<'a, MyBreakable>,
        ReadStorage<'a, MyTransform>,
    );

    fn run(&mut self, (updater, entities, breakables, transforms): Self::SystemData) {
        use specs::{prelude::ParallelIterator, ParJoin};

        (&entities, &breakables, &transforms).par_join().for_each(
            |(entity, breakable, transform)| {
                match breakable.incoming {
                    MyBreakableEvent::Damaged(chain_value) => {
                        println!("despawn {:?}", entity);
                        _ = entities.delete(entity);
                        if breakable.will_explode {
                            // 誘爆する
                            let entity = entities.create();
                            updater.insert(entity, MyExplosion::new(chain_value + 1));
                            updater.insert(entity, MyTransform::new(&transform.translation, 0.));
                            println!("spawn {:?}", entity);
                        }
                    }
                    _ => {}
                }
            },
        );
    }
}

struct MyChainExplosionSystem;

impl<'a> System<'a> for MyChainExplosionSystem {
    type SystemData = (
        WriteStorage<'a, MyBreakable>,
        ReadStorage<'a, MyExplosion>,
        ReadStorage<'a, MyTransform>,
    );

    fn run(&mut self, (mut breakables, explosions, transforms): Self::SystemData) {
        use specs::{prelude::ParallelIterator, ParJoin};

        (&mut breakables, &transforms)
            .par_join()
            .for_each(|(breakable, transform)| {
                let point = transform.translation;
                let query_explosions = (&explosions, &transforms).par_join();
                let collided_opponent =
                    query_explosions.find_map_any(|(explosion, explosion_transform)| {
                        let point2 = explosion_transform.translation;
                        if get_collision(
                            (&point, explosion.radius * EXPLOSION_RADIUS),
                            (&point2, BOMB_RADIUS),
                        ) {
                            Some(explosion.chain_value)
                        } else {
                            None
                        }
                    });
                if let Some(chain_value) = collided_opponent {
                    breakable.incoming = MyBreakableEvent::Damaged(chain_value);
                }
            });
    }
}

fn get_collision(c1: (&Vec2, f32), c2: (&Vec2, f32)) -> bool {
    let distance_squared = c1.0.distance_squared(*c2.0);
    distance_squared < (c1.1 + c2.1) * (c1.1 + c2.1)
}

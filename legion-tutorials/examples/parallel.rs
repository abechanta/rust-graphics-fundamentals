use ggez::{graphics::Color, Context, GameResult};
use glam::*;
// use legion::*;
use std::{
    collections::HashMap,
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
            title: "legion tutorial with ggez".to_owned(),
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

use legion::{Resources, Schedule, World};

struct MyApp {
    world: World,
    resources: Resources,
    scheduler: Schedule,
}

impl MyApp {
    fn new(ctx: &mut Context) -> GameResult<Self> {
        use ggez::graphics::FontData;

        ctx.gfx.add_font(
            "LiberationMono",
            FontData::from_path(ctx, "/LiberationMono-Regular.ttf")?,
        );

        let world = World::default();
        let mut resources = Resources::default();
        resources.insert::<MyTime>(MyTime::new());
        resources.insert::<MyChains>(MyChains(0));
        let scheduler = Schedule::builder()
            .add_system(my_time_system())
            .add_system(my_chains_system())
            .add_system(my_explosion_system())
            .add_system(my_breakable_system())
            .add_system(my_chain_explosion_system())
            .build();

        let my_app = MyApp {
            world,
            resources,
            scheduler,
        };
        Ok(my_app)
    }
}

use ggez::{event::EventHandler, GameError};

impl EventHandler<GameError> for MyApp {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        self.scheduler.execute(&mut self.world, &mut self.resources);

        use ggez::event::MouseButton;

        if ctx.mouse.button_just_pressed(MouseButton::Left) {
            let point = ctx.mouse.position();
            println!("MouseLeft {:?}", point);

            self.world
                .push((MyExplosion::new(0), MyTransform::new(&point.into(), 0.)));
        }
        if ctx.mouse.button_just_pressed(MouseButton::Middle) {
            let point = ctx.mouse.position();
            println!("MouseMiddle {:?}", point);

            self.world.push((
                MyBomb {},
                MyBreakable::new(),
                MyTransform::new(&point.into(), 1.),
            ));
        }
        if ctx.mouse.button_just_pressed(MouseButton::Right) {
            if let Some(mut time) = self.resources.get_mut::<MyTime>() {
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

        use legion::{component, IntoQuery};
        {
            let mut query = <(&MyExplosion, &MyTransform)>::query();

            query.iter(&self.world).for_each(|(explosion, transform)| {
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
            let mut query = <&MyTransform>::query().filter(component::<MyBomb>());

            query.iter(&self.world).for_each(|transform| {
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
            let chains = self.resources.get::<MyChains>();
            if let Some(chains) = chains {
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
        }

        canvas.finish(ctx)?;
        Ok(())
    }
}

struct MyTime {
    delta: Duration,
    elapsed: Duration,
    paused: bool,
    timer: Instant,
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

use legion::system;

#[system]
fn my_time(#[resource] time: &mut MyTime) {
    time.tick();
}

struct MyChains(u32);

impl Default for MyChains {
    fn default() -> Self {
        MyChains(0)
    }
}

use legion::{world::SubWorld, IntoQuery};

#[system]
#[read_component(MyExplosion)]
fn my_chains(world: &SubWorld, #[resource] chains: &mut MyChains) {
    use rayon::iter::ParallelIterator;

    let mut explosions = <&MyExplosion>::query();
    let chain_value_max = explosions
        .par_iter(world)
        .map(|explosion| explosion.chain_value)
        .reduce(
            || 0,
            |chain_value1, chain_value2| u32::max(chain_value1, chain_value2),
        );
    chains.0 = chain_value_max;
}

struct MyTransform {
    translation: Vec2,
    scaling: Vec2,
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

use legion::{component, systems::CommandBuffer, Entity};

#[system(for_each)]
fn my_explosion(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    explosion: &mut MyExplosion,
    transform: &mut MyTransform,
    #[resource] time: &mut MyTime,
) {
    explosion.timer = explosion.timer.saturating_sub(time.delta());
    if explosion.timer == Duration::ZERO {
        // 爆発おわり
        println!("despawn {:?}", entity);
        cmd.remove(*entity);
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

#[system(for_each)]
fn my_breakable(
    cmd: &mut CommandBuffer,
    entity: &Entity,
    breakable: &MyBreakable,
    transform: &MyTransform,
) {
    match breakable.incoming {
        MyBreakableEvent::Damaged(chain_value) => {
            println!("despawn {:?}", entity);
            cmd.remove(*entity);
            if breakable.will_explode {
                // 誘爆する
                let entity = cmd.push((
                    MyExplosion::new(chain_value + 1),
                    MyTransform::new(&transform.translation, 0.),
                ));
                println!("spawn {:?}", entity);
            }
        }
        _ => {}
    }
}

#[system]
#[read_component(Entity)]
#[write_component(MyBreakable)]
#[read_component(MyTransform)]
#[read_component(MyExplosion)]
fn my_chain_explosion(world: &mut SubWorld) {
    use rayon::iter::ParallelIterator;

    let mut query_breakables = <(Entity, &MyTransform)>::query().filter(component::<MyBreakable>());

    let collided = query_breakables
        .par_iter(world)
        .filter_map(|(entity, transform)| {
            let mut query_explosions = <(&MyExplosion, &MyTransform)>::query();
            let point = transform.translation;
            let collided_opponent = query_explosions.par_iter(world).find_map_any(
                |(explosion, explosion_transform)| {
                    let point2 = explosion_transform.translation;
                    if get_collision(
                        (&point, explosion.radius * EXPLOSION_RADIUS),
                        (&point2, BOMB_RADIUS),
                    ) {
                        Some(explosion.chain_value)
                    } else {
                        None
                    }
                },
            );
            if let Some(chain_value) = collided_opponent {
                Some((*entity, chain_value))
            } else {
                None
            }
        })
        .collect::<HashMap<Entity, u32>>();

    let mut query_breakables = <(Entity, &mut MyBreakable)>::query();
    query_breakables
        .iter_mut(world)
        .for_each(|(entity, breakable)| {
            if let Some(chain_value) = collided.get(entity) {
                breakable.incoming = MyBreakableEvent::Damaged(*chain_value);
            }
        });
}

fn get_collision(c1: (&Vec2, f32), c2: (&Vec2, f32)) -> bool {
    let distance_squared = c1.0.distance_squared(*c2.0);
    distance_squared < (c1.1 + c2.1) * (c1.1 + c2.1)
}

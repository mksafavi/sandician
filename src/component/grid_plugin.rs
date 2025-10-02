use bevy::{
    app::{App, FixedUpdate, Plugin, PostStartup, Startup, Update},
    asset::{Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        observer::On,
        query::With,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    image::Image,
    picking::{
        Pickable,
        events::{Move, Out, Pointer, Press, Release},
    },
    prelude::Vec3,
    sprite::Sprite,
    time::{Fixed, Time},
    transform::components::Transform,
    window::Window,
};

use super::{grid::Grid, particles::particle::Particle};

#[derive(Resource)]
struct OutputFrameHandle(Handle<Image>);

#[derive(Component)]
pub struct ParticleBrush {
    pub spawning: bool,
    pub position: (usize, usize),
    pub particle: Particle,
    pub size: usize,
}

impl Default for ParticleBrush {
    fn default() -> Self {
        Self::new()
    }
}

impl ParticleBrush {
    pub fn new() -> Self {
        Self {
            spawning: false,
            position: (0, 0),
            particle: Particle::Sand,
            size: 1,
        }
    }

    fn start_spawning(&mut self) {
        self.spawning = true;
    }

    fn stop_spawning(&mut self) {
        self.spawning = false;
    }

    fn move_brush(&mut self, position: Vec3, grid_size: (usize, usize), grid_scale: Vec3) {
        self.position = (
            ((position.x / grid_scale.x) + (grid_size.0 as f32 / 2.)) as usize,
            ((-position.y / grid_scale.y) + (grid_size.1 as f32 / 2.)) as usize,
        );
    }
}

#[derive(Resource, Clone)]
pub struct ConfigResource {
    pub width: usize,
    pub height: usize,
    update_rate: f64,
}
impl ConfigResource {
    pub fn new(width: usize, height: usize, update_rate: f64) -> Self {
        Self {
            width,
            height,
            update_rate,
        }
    }
}

pub struct GridPlugin {
    pub config: ConfigResource,
}
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .insert_resource(Time::<Fixed>::from_hz(self.config.update_rate))
            .add_systems(Startup, GridPlugin::init_grid_system)
            .add_systems(FixedUpdate, GridPlugin::update_grid_system)
            .add_systems(Update, GridPlugin::draw_grid_system)
            .add_systems(Update, GridPlugin::scale_output_frame_to_window_system)
            .add_systems(PostStartup, GridPlugin::init_inputs_system)
            .add_systems(Update, GridPlugin::spawn_brush_system);
    }
}

impl GridPlugin {
    fn init_grid_system(
        mut commands: Commands,
        config: Res<ConfigResource>,
        mut images: ResMut<Assets<Image>>,
    ) {
        commands.spawn(Grid::new(config.width, config.height));
        let handle = images.add(Grid::create_output_frame(config.width, config.height));
        commands.spawn((
            Sprite::from_image(handle.clone()),
            Transform::from_scale(Vec3::new(1., 1., 1.)),
            Pickable::default(),
        ));
        commands.insert_resource(OutputFrameHandle(handle));
        commands.spawn(ParticleBrush::new());
    }

    fn scale_output_frame_to_window_system(
        config: Res<ConfigResource>,
        windows: Query<&Window>,
        mut sprite_transform: Query<&mut Transform, With<Sprite>>,
    ) {
        if let Ok(mut s) = sprite_transform.single_mut() {
            let sprite_scale = match windows.single() {
                Ok(window) => {
                    let scale = (window.resolution.width() / config.width as f32)
                        .min(window.resolution.height() / config.height as f32);
                    Vec3::new(scale, scale, 1.)
                }
                Err(_) => Vec3::new(1., 1., 1.),
            };
            s.scale = sprite_scale;
        };
    }

    fn update_grid_system(mut grid: Query<&mut Grid>) {
        if let Ok(mut g) = grid.single_mut() {
            g.update_grid();
        }
    }

    fn draw_grid_system(
        mut grid: Query<&mut Grid>,
        output_frame_handle: Res<OutputFrameHandle>,
        mut images: ResMut<Assets<Image>>,
    ) {
        if let Ok(mut g) = grid.single_mut() {
            if let Some(image) = images.get_mut(&output_frame_handle.0) {
                g.draw_grid(image);
            }
        }
    }

    fn spawn_brush_system(particle_brush: Query<&ParticleBrush>, mut grid: Query<&mut Grid>) {
        if let Ok(mut g) = grid.single_mut() {
            if let Ok(pb) = particle_brush.single() {
                if pb.spawning {
                    g.spawn_brush(pb.position, pb.size, &pb.particle)
                }
            }
        }
    }

    fn init_inputs_system(mut commands: Commands, sprite_query: Query<Entity, With<Sprite>>) {
        if let Ok(sprite_entity) = sprite_query.single() {
            commands
                .entity(sprite_entity)
                .observe(
                    |m: On<Pointer<Press>>,
                     mut particle_brush: Query<&mut ParticleBrush>,
                     config: Res<ConfigResource>,
                     sprite_transform: Query<&Transform, With<Sprite>>| {
                        if let Ok(mut pb) = particle_brush.single_mut() {
                            pb.start_spawning();
                            if let Ok(s) = sprite_transform.single() {
                                if let Some(p) = m.hit.position {
                                    pb.move_brush(p, (config.width, config.height), s.scale);
                                }
                            }
                        }
                    },
                )
                .observe(
                    |_: On<Pointer<Release>>, mut particle_brush: Query<&mut ParticleBrush>| {
                        if let Ok(mut pb) = particle_brush.single_mut() {
                            pb.stop_spawning();
                        }
                    },
                )
                .observe(
                    |_: On<Pointer<Out>>, mut particle_brush: Query<&mut ParticleBrush>| {
                        if let Ok(mut pb) = particle_brush.single_mut() {
                            pb.stop_spawning();
                        }
                    },
                )
                .observe(
                    |m: On<Pointer<Move>>,
                     mut particle_brush: Query<&mut ParticleBrush>,
                     config: Res<ConfigResource>,
                     sprite_transform: Query<&Transform, With<Sprite>>| {
                        if let Ok(mut pb) = particle_brush.single_mut() {
                            if let Ok(s) = sprite_transform.single() {
                                if let Some(p) = m.hit.position {
                                    pb.move_brush(p, (config.width, config.height), s.scale);
                                }
                            }
                        }
                    },
                );
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::component::grid::{Cell, GridAccess};
    use crate::component::particles::particle::Particle;
    use crate::component::{grid::BACKGROUND_COLOR, macros::assert_color_srgb_eq};

    use super::*;
    use bevy::camera::NormalizedRenderTarget;
    use bevy::input::InputPlugin;
    use bevy::math::{Vec2, vec3};
    use bevy::picking::DefaultPickingPlugins;
    use bevy::picking::backend::HitData;
    use bevy::picking::pointer::{Location, PointerButton, PointerId};
    use bevy::prelude::default;
    use bevy::{
        ecs::query::With,
        window::{WindowPlugin, WindowRef, WindowResolution},
    };

    #[test]
    fn test_init_grid_system_creates_a_grid() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.add_plugins(GridPlugin {
            config: ConfigResource::new(2, 3, 100.),
        });

        app.update();

        assert_eq!(1, app.world_mut().query::<&Grid>().iter(app.world()).len());
    }

    #[test]
    fn test_scale_output_frame_to_window_system_scales_and_keeps_aspect_ratio_when_width_is_bigger()
    {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.add_plugins(GridPlugin {
            config: ConfigResource::new(200, 100, 100.),
        });

        app.add_systems(Startup, |mut c: Commands| {
            c.spawn(Transform::default()); /* Insert any transform asset that might be present at app runtime */
        });
        app.add_plugins(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(400, 400),
                ..default()
            }),
            ..default()
        });

        app.update();

        let mut t = app.world_mut().query_filtered::<&Transform, With<Sprite>>();
        if let Ok(t) = t.single(app.world()) {
            assert_eq!(Transform::from_scale(Vec3::new(2., 2., 1.)), *t);
        } else {
            panic!("missing the transform component of output frame sprite")
        }
    }

    #[test]
    fn test_scale_output_frame_to_window_system_scales_and_keeps_aspect_ratio_when_height_is_bigger()
     {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.add_plugins(GridPlugin {
            config: ConfigResource::new(100, 200, 100.),
        });

        app.add_systems(Startup, |mut c: Commands| {
            c.spawn(Transform::default()); /* Insert any transform asset that might be present at app runtime */
        });
        app.add_plugins(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(400, 400),
                ..default()
            }),
            ..default()
        });

        app.update();

        let mut t = app.world_mut().query_filtered::<&Transform, With<Sprite>>();
        if let Ok(t) = t.single(app.world()) {
            assert_eq!(Transform::from_scale(Vec3::new(2., 2., 1.)), *t);
        } else {
            panic!("missing the transform component of output frame sprite")
        }
    }

    #[test]
    fn test_draw_grid_system() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.add_plugins(GridPlugin {
            config: ConfigResource::new(2, 2, 100.),
        });

        app.update();

        let mut grid = app.world_mut().query::<&mut Grid>();
        if let Ok(mut g) = grid.single_mut(app.world_mut()) {
            g.spawn_particle(0, 1, Particle::new_water());
            g.spawn_particle(1, 1, Particle::Sand);
        } else {
            panic!("grid not found");
        }

        app.update();

        let frame = app.world().resource::<OutputFrameHandle>();
        if let Some(images) = app.world().get_resource::<Assets<Image>>() {
            let image = images.get(&frame.0).expect("image not found");
            assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 0).unwrap());
            assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 0).unwrap());
            assert_color_srgb_eq!(
                Particle::new_water().color(),
                image.get_color_at(0, 1).unwrap()
            );
            assert_color_srgb_eq!(Particle::Sand.color(), image.get_color_at(1, 1).unwrap());
        } else {
            panic!("image not found");
        }
    }

    #[test]
    fn test_particle_brush() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.add_plugins(GridPlugin {
            config: ConfigResource::new(2, 2, 100.),
        });

        app.update();

        let mut grid = app.world_mut().query::<&Grid>();
        if let Ok(g) = grid.single(app.world()) {
            assert_eq!(
                &vec![
                    Cell::new(None, 0),
                    Cell::new(None, 0),
                    Cell::new(None, 0),
                    Cell::new(None, 0),
                ],
                g.get_cells()
            );
        } else {
            panic!("grid not found");
        }

        let mut s = app.world_mut().query::<&mut ParticleBrush>();
        if let Ok(mut s) = s.single_mut(app.world_mut()) {
            s.spawning = true;
            s.position = (1, 1);
        } else {
            panic!("ParticleBrush not found");
        }
        app.update();

        let mut grid = app.world_mut().query::<&Grid>();
        if let Ok(g) = grid.single(app.world()) {
            assert_eq!(
                &vec![
                    Cell::new(None, 0),
                    Cell::new(None, 0),
                    Cell::new(None, 0),
                    Cell::new(Some(Particle::Sand), 0)
                ],
                g.get_cells()
            );
        } else {
            panic!("grid not found");
        }

        let mut s = app.world_mut().query::<&mut ParticleBrush>();
        if let Ok(mut s) = s.single_mut(app.world_mut()) {
            s.particle = Particle::Salt;
            s.position = (0, 1);
        } else {
            panic!("ParticleBrush not found");
        }
        app.update();

        let mut grid = app.world_mut().query::<&Grid>();
        if let Ok(g) = grid.single(app.world()) {
            assert_eq!(
                &vec![
                    Cell::new(None, 0),
                    Cell::new(None, 0),
                    Cell::new(Some(Particle::Salt), 0),
                    Cell::new(Some(Particle::Sand), 0)
                ],
                g.get_cells()
            );
        } else {
            panic!("grid not found");
        }
    }

    #[test]
    fn test_particle_brush_size() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.add_plugins(GridPlugin {
            config: ConfigResource::new(2, 2, 100.),
        });

        app.update();

        let mut grid = app.world_mut().query::<&Grid>();
        if let Ok(g) = grid.single(app.world()) {
            assert_eq!(
                &vec![
                    Cell::new(None, 0),
                    Cell::new(None, 0),
                    Cell::new(None, 0),
                    Cell::new(None, 0),
                ],
                g.get_cells()
            );
        } else {
            panic!("grid not found");
        }

        let mut s = app.world_mut().query::<&mut ParticleBrush>();
        if let Ok(mut s) = s.single_mut(app.world_mut()) {
            s.spawning = true;
            s.size = 2;
        } else {
            panic!("ParticleBrush not found");
        }
        app.update();

        let mut grid = app.world_mut().query::<&Grid>();
        if let Ok(g) = grid.single(app.world()) {
            assert_eq!(
                &vec![
                    Cell::new(Some(Particle::Sand), 0),
                    Cell::new(Some(Particle::Sand), 0),
                    Cell::new(Some(Particle::Sand), 0),
                    Cell::new(Some(Particle::Sand), 0),
                ],
                g.get_cells()
            );
        } else {
            panic!("grid not found");
        }
    }

    #[test]
    fn test_particle_brush_start_and_stop_spawning() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.add_plugins(InputPlugin);
        app.add_plugins(DefaultPickingPlugins);
        app.add_plugins(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(300, 200),
                ..default()
            }),
            ..default()
        });
        app.add_plugins(GridPlugin {
            config: ConfigResource::new(300, 200, 100.),
        });

        app.update();

        trigger_pressed_event(&mut app, Vec3::ZERO);
        assert_particle_brush_spawning(&mut app, true);

        trigger_released_event(&mut app);
        assert_particle_brush_spawning(&mut app, false);

        trigger_pressed_event(&mut app, Vec3::ZERO);
        assert_particle_brush_spawning(&mut app, true);

        trigger_out_event(&mut app);
        assert_particle_brush_spawning(&mut app, false);
    }

    #[test]
    fn test_particle_brush_pressed_event_sets_brush_position() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.add_plugins(InputPlugin);
        app.add_plugins(DefaultPickingPlugins);
        app.add_plugins(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(300, 200),
                ..default()
            }),
            ..default()
        });

        app.add_plugins(GridPlugin {
            config: ConfigResource::new(300, 200, 100.),
        });

        app.update();

        trigger_pressed_event(&mut app, vec3(-150., 100., 0.));
        assert_particle_brush_position(&mut app, (0, 0));

        trigger_pressed_event(&mut app, vec3(150., -100., 0.));
        assert_particle_brush_position(&mut app, (300, 200));
    }

    #[test]
    fn test_particle_brush_move_brush() {
        let mut app = trigger_move_event(vec3(-150., 100., 0.), (300, 200), (300, 200));
        assert_particle_brush_position(&mut app, (0, 0));

        let mut app = trigger_move_event(vec3(150., -100., 0.), (300, 200), (300, 200));
        assert_particle_brush_position(&mut app, (300, 200));

        let mut app = trigger_move_event(vec3(-450., 300., 0.), (300, 200), (900, 600));
        assert_particle_brush_position(&mut app, (0, 0));

        let mut app = trigger_move_event(vec3(450., -300., 0.), (300, 200), (900, 600));
        assert_particle_brush_position(&mut app, (300, 200));
    }

    fn trigger_pressed_event(app: &mut App, position: Vec3) {
        let mut entity_query = app.world_mut().query_filtered::<Entity, With<Sprite>>();
        if let Ok(entity) = entity_query.single(app.world()) {
            let event = Pointer::new(
                PointerId::Mouse,
                Location {
                    target: NormalizedRenderTarget::Window(
                        WindowRef::Entity(Entity::from_raw_u32(0).unwrap())
                            .normalize(Some(Entity::from_raw_u32(0).unwrap()))
                            .unwrap(),
                    ),
                    position: Vec2::ZERO,
                },
                Press {
                    button: PointerButton::Primary,
                    hit: HitData {
                        camera: Entity::from_raw_u32(0).unwrap(),
                        depth: 0.,
                        position: Some(position),
                        normal: None,
                    },
                },
                entity,
            );
            app.world_mut().trigger(event);
        } else {
            panic!("sprite not found");
        }
    }

    fn trigger_released_event(app: &mut App) {
        let mut entity_query = app.world_mut().query_filtered::<Entity, With<Sprite>>();
        if let Ok(entity) = entity_query.single(app.world()) {
            let event = Pointer::new(
                PointerId::Mouse,
                Location {
                    target: NormalizedRenderTarget::Window(
                        WindowRef::Entity(Entity::from_raw_u32(0).unwrap())
                            .normalize(Some(Entity::from_raw_u32(0).unwrap()))
                            .unwrap(),
                    ),
                    position: Vec2::ZERO,
                },
                Release {
                    button: PointerButton::Primary,
                    hit: HitData {
                        camera: Entity::from_raw_u32(0).unwrap(),
                        depth: 0.,
                        position: None,
                        normal: None,
                    },
                },
                entity,
            );
            app.world_mut().trigger(event);
        } else {
            panic!("sprite not found");
        }
    }

    fn trigger_out_event(app: &mut App) {
        let mut entity_query = app.world_mut().query_filtered::<Entity, With<Sprite>>();
        if let Ok(entity) = entity_query.single(app.world()) {
            let event = Pointer::new(
                PointerId::Mouse,
                Location {
                    target: NormalizedRenderTarget::Window(
                        WindowRef::Entity(Entity::from_raw_u32(0).unwrap())
                            .normalize(Some(Entity::from_raw_u32(0).unwrap()))
                            .unwrap(),
                    ),
                    position: Vec2::ZERO,
                },
                Out {
                    hit: HitData {
                        camera: Entity::from_raw_u32(0).unwrap(),
                        depth: 0.,
                        position: None,
                        normal: None,
                    },
                },
                entity,
            );
            app.world_mut().trigger(event);
        } else {
            panic!("sprite not found");
        }
    }

    fn trigger_move_event(
        position: Vec3,
        grid_size: (usize, usize),
        window_size: (u32, u32),
    ) -> App {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.add_plugins(InputPlugin);
        app.add_plugins(DefaultPickingPlugins);
        app.add_plugins(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(window_size.0, window_size.1),
                ..default()
            }),
            ..default()
        });
        app.add_plugins(GridPlugin {
            config: ConfigResource::new(grid_size.0, grid_size.1, 100.),
        });

        app.update();

        let mut entity_query = app.world_mut().query_filtered::<Entity, With<Sprite>>();
        if let Ok(entity) = entity_query.single(app.world()) {
            let event = Pointer::new(
                PointerId::Mouse,
                Location {
                    target: NormalizedRenderTarget::Window(
                        WindowRef::Entity(Entity::from_raw_u32(0).unwrap())
                            .normalize(Some(Entity::from_raw_u32(0).unwrap()))
                            .unwrap(),
                    ),
                    position: Vec2::ZERO,
                },
                Move {
                    hit: HitData {
                        camera: Entity::from_raw_u32(0).unwrap(),
                        depth: 0.,
                        position: Some(position),
                        normal: None,
                    },
                    delta: Vec2::ZERO,
                },
                entity,
            );
            app.world_mut().trigger(event);
        } else {
            panic!("sprite not found");
        }
        app
    }

    fn assert_particle_brush_position(app: &mut App, position: (usize, usize)) {
        let mut s = app.world_mut().query::<&ParticleBrush>();
        if let Ok(s) = s.single(app.world()) {
            assert_eq!(position, s.position);
        } else {
            panic!("ParticleBrush not found");
        }
    }

    fn assert_particle_brush_spawning(app: &mut App, expected: bool) {
        let mut s = app.world_mut().query::<&ParticleBrush>();
        if let Ok(s) = s.single(app.world()) {
            assert_eq!(expected, s.spawning);
        } else {
            panic!("ParticleBrush not found");
        }
    }
}

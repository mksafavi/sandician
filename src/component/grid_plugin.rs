use bevy::{
    app::{App, FixedUpdate, Plugin, PostStartup, Startup, Update},
    asset::{Assets, Handle},
    color::Alpha,
    ecs::{
        bundle::Bundle,
        children,
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
        events::{Click, Move, Out, Pointer, Press, Release},
    },
    prelude::{SpawnRelated, Vec3},
    text::TextFont,
    time::{Fixed, Time},
    ui::{
        AlignItems, BackgroundColor, BorderColor, BorderRadius, Display, FlexDirection, FlexWrap,
        Node, UiRect, Val, px,
        widget::{Button, ImageNode, Text},
    },
    utils::default,
};

use super::{grid::Grid, particles::particle::Particle};

#[derive(Resource)]
struct OutputFrameHandle(Handle<Image>);

#[derive(Component, Debug)]
struct ParticleRadio(Particle);

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

    fn move_brush(&mut self, position: Vec3, grid_size: (usize, usize)) {
        self.position = (
            ((position.x + 0.5) * grid_size.0 as f32) as usize,
            ((position.y + 0.5) * grid_size.1 as f32) as usize,
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
            .add_systems(Startup, init_grid_system)
            .add_systems(FixedUpdate, update_grid_system)
            .add_systems(Update, draw_grid_system)
            .add_systems(PostStartup, init_inputs_system)
            .add_systems(PostStartup, observe_particle_button_particle_brush_system)
            .add_systems(Update, spawn_brush_system);
    }
}

fn init_grid_system(
    mut commands: Commands,
    config: Res<ConfigResource>,
    mut images: ResMut<Assets<Image>>,
) {
    commands.spawn(Grid::new(config.width, config.height));
    let handle = images.add(Grid::create_output_frame(config.width, config.height));
    commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![brush_node(), grid_node(&handle)],
    ));

    commands.insert_resource(OutputFrameHandle(handle));
    commands.spawn(ParticleBrush::new());
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

fn observe_particle_button_particle_brush_system(
    mut commands: Commands,
    particle_buttons: Query<Entity, With<Button>>,
) {
    particle_buttons.iter().for_each(|e| {
        commands.entity(e).observe(
            |m: On<Pointer<Click>>,
             mut particle_brush: Query<&mut ParticleBrush>,
             particle_buttons: Query<(Entity, &ParticleRadio), With<Button>>| {
                particle_buttons
                    .iter()
                    .filter(|(e, _)| m.entity == *e)
                    .for_each(|(_, pr)| {
                        if let Ok(mut pb) = particle_brush.single_mut() {
                            pb.particle = pr.0.clone();
                        }
                    });
            },
        );
    });
}

fn init_inputs_system(mut commands: Commands, image_node_query: Query<Entity, With<ImageNode>>) {
    if let Ok(image_node_entity) = image_node_query.single() {
        commands
            .entity(image_node_entity)
            .observe(
                |m: On<Pointer<Press>>,
                 mut particle_brush: Query<&mut ParticleBrush>,
                 config: Res<ConfigResource>| {
                    if let Ok(mut pb) = particle_brush.single_mut() {
                        pb.start_spawning();
                        if let Some(p) = m.hit.position {
                            pb.move_brush(p, (config.width, config.height));
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
                 config: Res<ConfigResource>| {
                    if let Ok(mut pb) = particle_brush.single_mut() {
                        if let Some(p) = m.hit.position {
                            pb.move_brush(p, (config.width, config.height));
                        }
                    }
                },
            );
    }
}

fn brush_node() -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(4),
            ..default()
        },
        children![
            radio(Particle::Sand),
            radio(Particle::Salt),
            radio(Particle::new_water()),
            radio(Particle::Rock),
        ],
    )
}

fn radio(particle: Particle) -> impl Bundle {
    (
        Node {
            height: px(26),
            padding: UiRect::all(px(2)),
            margin: UiRect::all(px(2)),
            border: UiRect::all(px(3)),
            align_items: AlignItems::Center,
            ..default()
        },
        BorderColor::all(particle.color()),
        BorderRadius::all(px(3)),
        ParticleRadio(particle.clone()),
        BackgroundColor(particle.color().with_alpha(0.3)),
        Button,
        children![(
            Text::new(particle.to_string()),
            TextFont {
                font_size: 12.,
                ..default()
            }
        )],
    )
}

fn grid_node(handle: &Handle<Image>) -> impl Bundle {
    (
        Node {
            width: Val::Auto,
            height: Val::Auto,
            ..default()
        },
        ImageNode::new(handle.clone()),
        Pickable::default(),
    )
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use crate::component::grid::{Cell, GridAccess};
    use crate::component::particles::particle::Particle;
    use crate::component::{grid::BACKGROUND_COLOR, macros::assert_color_srgb_eq};

    use super::*;
    use bevy::camera::NormalizedRenderTarget;
    use bevy::input::InputPlugin;
    use bevy::math::{Vec2, vec3};
    use bevy::picking::DefaultPickingPlugins;
    use bevy::picking::backend::HitData;
    use bevy::picking::events::Click;
    use bevy::picking::pointer::{Location, PointerButton, PointerId};
    use bevy::prelude::default;
    use bevy::window::Window;
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

        trigger_pressed_event(&mut app, vec3(-0.5, -0.5, 0.));
        assert_particle_brush_position(&mut app, (0, 0));

        trigger_pressed_event(&mut app, vec3(0.5, 0.5, 0.));
        assert_particle_brush_position(&mut app, (300, 200));
    }

    #[test]
    fn test_particle_brush_move_brush() {
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

        trigger_move_event(&mut app, vec3(-0.5, -0.5, 0.));
        assert_particle_brush_position(&mut app, (0, 0));

        trigger_move_event(&mut app, vec3(0.5, 0.5, 0.));
        assert_particle_brush_position(&mut app, (300, 200));
    }

    #[test]
    fn test_particle_buttons_set_the_particle_type_in_particle_brush() {
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

        trigger_particle_button_click_event(&mut app, Particle::Salt);
        assert_particle_brush_particle(&mut app, Particle::Salt);

        trigger_particle_button_click_event(&mut app, Particle::Sand);
        assert_particle_brush_particle(&mut app, Particle::Sand);

        trigger_particle_button_click_event(&mut app, Particle::new_water());
        assert_particle_brush_particle(&mut app, Particle::new_water());

        trigger_particle_button_click_event(&mut app, Particle::Rock);
        assert_particle_brush_particle(&mut app, Particle::Rock);
    }

    fn trigger_pressed_event(app: &mut App, position: Vec3) {
        let mut entity_query = app.world_mut().query_filtered::<Entity, With<ImageNode>>();
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
            panic!("image node not found");
        }
    }

    fn trigger_released_event(app: &mut App) {
        let mut entity_query = app.world_mut().query_filtered::<Entity, With<ImageNode>>();
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
            panic!("image node not found");
        }
    }

    fn trigger_out_event(app: &mut App) {
        let mut entity_query = app.world_mut().query_filtered::<Entity, With<ImageNode>>();
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
            panic!("image node not found");
        }
    }

    fn trigger_move_event(app: &mut App, position: Vec3) {
        let mut entity_query = app.world_mut().query_filtered::<Entity, With<ImageNode>>();
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
            panic!("image node not found");
        }
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

    fn assert_particle_brush_particle(app: &mut App, particle: Particle) {
        let mut s = app.world_mut().query::<&ParticleBrush>();
        if let Ok(s) = s.single(app.world()) {
            assert_eq!(particle, s.particle);
        } else {
            panic!("ParticleBrush not found");
        }
    }

    fn trigger_particle_button_click_event(app: &mut App, particle: Particle) {
        let radio_button = app
            .world_mut()
            .query::<(Entity, &ParticleRadio)>()
            .iter(app.world())
            .filter(|(_, p)| p.0 == particle)
            .collect::<Vec<_>>();

        let (entity, _) = radio_button[0];
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
            Click {
                hit: HitData {
                    camera: Entity::from_raw_u32(0).unwrap(),
                    depth: 0.,
                    position: None,
                    normal: None,
                },
                button: PointerButton::Primary,
                duration: Duration::from_secs(1),
            },
            entity,
        );
        app.world_mut().trigger(event);
    }
}

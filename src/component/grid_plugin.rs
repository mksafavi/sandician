use std::collections::VecDeque;

use bevy::{
    app::{App, FixedUpdate, Plugin, PostStartup, Startup, Update},
    asset::{AssetServer, Assets, Handle},
    color::{Alpha, Color},
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
    text::{Font, TextFont},
    time::{Fixed, Time},
    ui::{
        AlignItems, BackgroundColor, BorderColor, Display, FlexDirection, FlexWrap, JustifyContent,
        Node, UiRect, Val, px,
        widget::{Button, ImageNode, Text},
    },
    utils::default,
};

use crate::component::{grid::BACKGROUND_COLOR, particles::rock::Rock};

use super::{
    grid::Grid,
    particles::{
        drain::Drain, particle::Particle, particle::ParticleKind, salt::Salt, sand::Sand, tap::Tap,
        water::Water,
    },
};

const ASSET_FONT_PATH: &str = "fonts/Adventurer.ttf";

#[derive(Resource)]
struct OutputFrameHandle(Handle<Image>);

#[derive(Component, Debug)]
struct ParticleRadio(Option<ParticleKind>);

#[derive(Component)]
pub struct ParticleBrush {
    pub spawning: bool,
    pub positions: VecDeque<(usize, usize)>,
    pub particle_kind: Option<ParticleKind>,
    pub size: usize,
    last_position: Option<Vec3>,
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
            positions: VecDeque::new(),
            particle_kind: Some(ParticleKind::from(Sand::new())),
            size: 8,
            last_position: None,
        }
    }

    fn start_spawning(&mut self) {
        self.spawning = true;
    }

    fn stop_spawning(&mut self) {
        self.spawning = false;
    }

    fn set_position(&mut self, position: Vec3, grid_size: (usize, usize)) {
        self.positions.push_back((
            ((position.x + 0.5) * grid_size.0 as f32) as usize,
            ((position.y + 0.5) * grid_size.1 as f32) as usize,
        ));
    }

    fn set_position_linear(&mut self, position: Vec3, grid_size: (usize, usize)) {
        if let Some(last_position) = self.last_position {
            let steps = 10;
            for s in 1..=steps {
                let n = last_position.lerp(position, s as f32 / steps as f32);
                self.set_position(n, grid_size);
            }
        } else {
            self.set_position(position, grid_size);
        }
        self.last_position = Some(position);
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
    asset_server: Option<Res<AssetServer>>,
) {
    commands.spawn(Grid::new(config.width, config.height));
    let handle = images.add(Grid::create_output_frame(config.width, config.height));
    let font = match asset_server {
        Some(a) => a.load(ASSET_FONT_PATH),
        None => default(),
    };
    commands.spawn((
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        children![brush_node(font), grid_node(&handle)],
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
    if let Ok(mut g) = grid.single_mut()
        && let Some(image) = images.get_mut(&output_frame_handle.0)
    {
        g.draw_grid(image);
    }
}

fn spawn_brush_system(mut particle_brush: Query<&mut ParticleBrush>, mut grid: Query<&mut Grid>) {
    if let Ok(mut g) = grid.single_mut()
        && let Ok(mut pb) = particle_brush.single_mut()
        && pb.spawning
    {
        while pb.positions.len() != 1 {
            if let Some(position) = pb.positions.pop_front() {
                g.spawn_brush(position, pb.size, pb.particle_kind.as_ref());
            }
        }
        if let Some(&position) = pb.positions.front() {
            g.spawn_brush(position, pb.size, pb.particle_kind.as_ref());
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
             particle_buttons: Query<&ParticleRadio>| {
                if let Ok(pr) = particle_buttons.get(m.entity)
                    && let Ok(mut pb) = particle_brush.single_mut()
                {
                    pb.particle_kind = pr.0.clone();
                }
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
                            pb.positions = VecDeque::new();
                            pb.set_position(p, (config.width, config.height));
                            pb.last_position = Some(p);
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
                    if let Ok(mut pb) = particle_brush.single_mut()
                        && let Some(p) = m.hit.position
                        && pb.spawning
                    {
                        pb.set_position_linear(p, (config.width, config.height));
                    }
                },
            );
    }
}

fn brush_node(font: Handle<Font>) -> impl Bundle {
    (
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::Wrap,
            column_gap: px(0),
            ..default()
        },
        BackgroundColor(Color::BLACK),
        children![
            radio(Some(Particle::from(Sand::new())), font.clone()),
            radio(Some(Particle::from(Salt::new())), font.clone()),
            radio(Some(Particle::from(Water::new())), font.clone()),
            radio(Some(Particle::from(Rock::new())), font.clone()),
            radio(Some(Particle::from(Drain::new())), font.clone()),
            radio(Some(Particle::from(Tap::new())), font.clone()),
            radio(None, font.clone()),
            (Node {
                flex_grow: 100.0,
                width: Val::Auto,
                ..default()
            })
        ],
    )
}

fn radio(particle: Option<Particle>, font: Handle<Font>) -> impl Bundle {
    let color = match particle.clone() {
        Some(p) => p.color(),
        None => BACKGROUND_COLOR,
    };
    let text = match particle.clone() {
        Some(p) => p.to_string(),
        None => "empty".to_string(),
    };

    (
        Node {
            height: px(26),
            flex_grow: 1.0,
            padding: UiRect::all(px(2)),
            margin: UiRect::all(px(2)),
            border: UiRect::all(px(3)),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        BorderColor::all(color),
        ParticleRadio({
            match particle {
                Some(p) => Some(p.kind),
                None => None,
            }
        }),
        BackgroundColor(color.with_alpha(0.3)),
        Button,
        children![(
            Text::new(text),
            TextFont {
                font_size: 16.,
                font,
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
    use crate::component::particles::rock::Rock;
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
            g.spawn_particle((0, 1), Particle::from(Water::new()));
            g.spawn_particle((1, 1), Particle::from(Sand::new()));
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
                Particle::from(Water::new()).color(),
                image.get_color_at(0, 1).unwrap()
            );
            assert_color_srgb_eq!(
                Particle::from(Sand::new()).color(),
                image.get_color_at(1, 1).unwrap()
            );
        } else {
            panic!("image not found");
        }
    }

    #[test]
    fn test_particle_brush_spawn_position_until_no_positions_remains_but_keep_the_last_one() {
        let mut app = App::new();
        app.init_resource::<Assets<Image>>();
        app.add_plugins(GridPlugin {
            config: ConfigResource::new(2, 2, 100.),
        });

        app.update();

        let mut grid = app.world_mut().query::<&Grid>();
        if let Ok(g) = grid.single(app.world()) {
            assert_eq!(
                &vec![Cell::empty(), Cell::empty(), Cell::empty(), Cell::empty(),],
                g.get_cells()
            );
        } else {
            panic!("grid not found");
        }

        let mut s = app.world_mut().query::<&mut ParticleBrush>();
        if let Ok(mut s) = s.single_mut(app.world_mut()) {
            s.spawning = true;
            s.size = 1;
            s.positions = VecDeque::from([(0, 0), (1, 1), (0, 1)]);
        } else {
            panic!("ParticleBrush not found");
        }
        app.update();

        let mut grid = app.world_mut().query::<&Grid>();
        if let Ok(g) = grid.single(app.world()) {
            assert_eq!(
                &vec![
                    Cell::new(Particle::from(Sand::new())),
                    Cell::empty(),
                    Cell::new(Particle::from(Sand::new())),
                    Cell::new(Particle::from(Sand::new()))
                ],
                g.get_cells()
            );
        } else {
            panic!("grid not found");
        }

        let mut s = app.world_mut().query::<&mut ParticleBrush>();
        if let Ok(s) = s.single(app.world_mut()) {
            assert_eq!(VecDeque::from([(0, 1)]), s.positions);
        } else {
            panic!("ParticleBrush not found");
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
                &vec![Cell::empty(), Cell::empty(), Cell::empty(), Cell::empty(),],
                g.get_cells()
            );
        } else {
            panic!("grid not found");
        }

        let mut s = app.world_mut().query::<&mut ParticleBrush>();
        if let Ok(mut s) = s.single_mut(app.world_mut()) {
            s.spawning = true;
            s.size = 2;
            s.positions.push_back((0, 0));
        } else {
            panic!("ParticleBrush not found");
        }
        app.update();

        let mut grid = app.world_mut().query::<&Grid>();
        if let Ok(g) = grid.single(app.world()) {
            assert_eq!(
                &vec![
                    Cell::new(Particle::from(Sand::new())),
                    Cell::new(Particle::from(Sand::new())),
                    Cell::new(Particle::from(Sand::new())),
                    Cell::empty(),
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
        assert!(query_particle_brush(&mut app).spawning);

        trigger_released_event(&mut app);
        assert!(!query_particle_brush(&mut app).spawning);

        trigger_pressed_event(&mut app, Vec3::ZERO);
        assert!(query_particle_brush(&mut app).spawning);

        trigger_out_event(&mut app);
        assert!(!query_particle_brush(&mut app).spawning);
    }

    #[test]
    fn test_particle_brush_pressed_event_sets_brush_position_and_clears_the_positions() {
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

        trigger_pressed_event(&mut app, vec3(0., 0., 0.));
        assert_eq!(
            VecDeque::from([(150, 100)]),
            query_particle_brush(&mut app).positions
        );
        assert_eq!(
            vec3(0., 0., 0.),
            query_particle_brush(&mut app).last_position.unwrap()
        );

        trigger_pressed_event(&mut app, vec3(0.5, 0.5, 0.));
        assert_eq!(
            VecDeque::from([(300, 200)]),
            query_particle_brush(&mut app).positions
        );
        assert_eq!(
            vec3(0.5, 0.5, 0.),
            query_particle_brush(&mut app).last_position.unwrap()
        );
    }
    #[test]
    fn test_particle_brush_move_brush_after_press_event() {
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

        trigger_move_event(&mut app, vec3(-0.4, -0.4, 0.));

        assert_eq!(
            VecDeque::from([
                (0, 0),
                (3, 2),
                (5, 3),
                (9, 6),
                (11, 7),
                (15, 10),
                (18, 12),
                (20, 13),
                (23, 15),
                (27, 18),
                (29, 19),
            ]),
            query_particle_brush(&mut app).positions
        );

        trigger_move_event(&mut app, vec3(-0.5, -0.5, 0.));

        assert_eq!(
            VecDeque::from([
                (0, 0),
                (3, 2),
                (5, 3),
                (9, 6),
                (11, 7),
                (15, 10),
                (18, 12),
                (20, 13),
                (23, 15),
                (27, 18),
                (29, 19),
                (27, 18),
                (23, 15),
                (20, 13),
                (18, 12),
                (15, 10),
                (11, 7),
                (9, 6),
                (5, 3),
                (2, 1),
                (0, 0),
            ]),
            query_particle_brush(&mut app).positions
        );
    }

    #[test]
    fn test_particle_brush_move_brush_only_set_positions_when_spawning() {
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
        assert_eq!(VecDeque::from([]), query_particle_brush(&mut app).positions);

        trigger_pressed_event(&mut app, vec3(-0.5, -0.5, 0.));

        trigger_move_event(&mut app, vec3(-0.5, -0.5, 0.));
        assert_eq!(
            VecDeque::from([
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
                (0, 0),
            ]),
            query_particle_brush(&mut app).positions
        );
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

        assert_eq!(
            Some(ParticleKind::from(Sand::new())),
            query_particle_brush(&mut app).particle_kind,
            "default brush is set to sand"
        );

        trigger_particle_button_click_event(&mut app, Some(Particle::from(Salt::new())));
        assert_eq!(
            Some(ParticleKind::from(Salt::new())),
            query_particle_brush(&mut app).particle_kind
        );

        trigger_particle_button_click_event(&mut app, Some(Particle::from(Sand::new())));
        assert_eq!(
            Some(ParticleKind::from(Sand::new())),
            query_particle_brush(&mut app).particle_kind
        );

        trigger_particle_button_click_event(&mut app, Some(Particle::from(Water::new())));
        assert_eq!(
            Some(ParticleKind::from(Water::new())),
            query_particle_brush(&mut app).particle_kind
        );

        trigger_particle_button_click_event(&mut app, Some(Particle::from(Rock::new())));
        assert_eq!(
            Some(ParticleKind::from(Rock::new())),
            query_particle_brush(&mut app).particle_kind
        );

        trigger_particle_button_click_event(&mut app, Some(Particle::from(Drain::new())));
        assert_eq!(
            Some(ParticleKind::from(Drain::new())),
            query_particle_brush(&mut app).particle_kind
        );

        trigger_particle_button_click_event(&mut app, Some(Particle::from(Tap::new())));
        assert_eq!(
            Some(ParticleKind::from(Tap::new())),
            query_particle_brush(&mut app).particle_kind
        );

        trigger_particle_button_click_event(&mut app, None);
        assert_eq!(None, query_particle_brush(&mut app).particle_kind);
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

    fn query_particle_brush(app: &mut App) -> &ParticleBrush {
        if let Ok(s) = app
            .world_mut()
            .query::<&ParticleBrush>()
            .single(app.world())
        {
            s
        } else {
            panic!("ParticleBrush not found");
        }
    }

    fn trigger_particle_button_click_event(app: &mut App, particle: Option<Particle>) {
        let radio_button = app
            .world_mut()
            .query::<(Entity, &ParticleRadio)>()
            .iter(app.world())
            .filter(|(_, p)| p.0 == particle.clone().map(|p| p.kind))
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

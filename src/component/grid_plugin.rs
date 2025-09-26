use bevy::{
    app::{App, FixedUpdate, Plugin, Startup, Update},
    asset::{Assets, Handle},
    ecs::{
        query::With,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    image::Image,
    prelude::Vec3,
    sprite::Sprite,
    time::{Fixed, Time},
    transform::components::Transform,
    window::Window,
};

use super::grid::Grid;

#[derive(Resource)]
struct OutputFrameHandle(Handle<Image>);

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
            .add_systems(Update, GridPlugin::scale_output_frame_to_window_system);
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
        ));
        commands.insert_resource(OutputFrameHandle(handle));
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
        grid: Query<&Grid>,
        output_frame_handle: Res<OutputFrameHandle>,
        mut images: ResMut<Assets<Image>>,
    ) {
        if let Ok(g) = grid.single() {
            if let Some(image) = images.get_mut(&output_frame_handle.0) {
                g.draw_grid(image);
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::component::particles::particle::Particle;
    use crate::component::{grid::BACKGROUND_COLOR, macros::assert_color_srgb_eq};

    use super::*;
    use bevy::prelude::default;
    use bevy::window::WindowResolution;
    use bevy::{ecs::query::With, window::WindowPlugin};

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
                resolution: WindowResolution::new(400., 400.),
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
                resolution: WindowResolution::new(400., 400.),
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
}

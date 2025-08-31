use bevy::{
    app::{App, FixedUpdate, Plugin, Startup, Update},
    asset::{Assets, Handle},
    ecs::{
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    image::Image,
    sprite::Sprite,
    time::{Fixed, Time},
};

use super::particle::Grid;

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
            .add_systems(Update, GridPlugin::draw_grid_system);
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
        commands.spawn(Sprite::from_image(handle.clone()));
        commands.insert_resource(OutputFrameHandle(handle));
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
                g.draw_grid(image)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::component::{
        macros::assert_color_srgb_eq,
        particle::{Particle, BACKGROUND_COLOR},
    };

    use super::*;
    use bevy::prelude::IntoScheduleConfigs;

    #[test]
    fn test_init_grid_system_creates_a_grid() {
        let mut app = App::new();
        app.insert_resource(ConfigResource::new(2, 3, 100.));
        app.init_resource::<Assets<Image>>();
        app.add_systems(Startup, GridPlugin::init_grid_system);
        app.update();
        assert_eq!(1, app.world_mut().query::<&Grid>().iter(app.world()).len());
    }

    #[test]
    fn test_draw_grid_system() {
        fn fixture_spawn_particle_system(mut grid: Query<&mut Grid>) {
            let mut g = grid.iter_mut().last().unwrap();
            g.spawn_particle(0, 0, Particle::Sand);
            g.spawn_particle(1, 0, Particle::Water);
        }

        fn assert_read_output_frame_system(
            output_frame_handle: Res<OutputFrameHandle>,
            images: ResMut<Assets<Image>>,
        ) {
            let image = images.get(&output_frame_handle.0).expect("Image not found");
            assert_color_srgb_eq!(Particle::Sand.color(), image.get_color_at(0, 0).unwrap());
            assert_color_srgb_eq!(Particle::Water.color(), image.get_color_at(1, 0).unwrap());
            assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(0, 1).unwrap());
            assert_color_srgb_eq!(BACKGROUND_COLOR, image.get_color_at(1, 1).unwrap());
        }

        let mut app = App::new();
        app.insert_resource(ConfigResource::new(5, 6, 100.));
        app.init_resource::<Assets<Image>>();

        //app.insert_resource(OutputFrameHandle);
        app.add_systems(Startup, GridPlugin::init_grid_system);
        app.add_systems(
            Update,
            (
                fixture_spawn_particle_system,
                GridPlugin::draw_grid_system,
                assert_read_output_frame_system,
            )
                .chain(),
        );
        app.update();
    }
}

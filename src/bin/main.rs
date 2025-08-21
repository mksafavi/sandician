use bevy::app::{App, Startup};
use bevy::core_pipeline::core_2d::Camera2d;
use bevy::ecs::system::Commands;
use bevy::DefaultPlugins;
use sandsim::component::particle::{ConfigResource, GridPlugin};

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(GridPlugin {
            config: ConfigResource::new(400, 300, 120.),
        })
        .add_systems(Startup, setup_camera)
        .run();
}

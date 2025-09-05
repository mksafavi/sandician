use bevy::app::{App, FixedUpdate, Startup};
use bevy::core_pipeline::core_2d::Camera2d;
use bevy::ecs::system::Commands;
use bevy::prelude::PluginGroup;
use bevy::render::texture::ImagePlugin;
use bevy::DefaultPlugins;
use sandsim::component::grid_plugin::{ConfigResource, GridPlugin};
use sandsim::component::inputs;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(GridPlugin {
            config: ConfigResource::new(400, 300, 120.),
        })
        .add_systems(Startup, setup_camera)
        .add_systems(FixedUpdate, inputs::mouse_spawn_brush_system)
        .run();
}

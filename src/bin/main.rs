use bevy::DefaultPlugins;
use bevy::app::{App, FixedUpdate, Startup};
use bevy::camera::Camera2d;
use bevy::ecs::system::Commands;
use bevy::image::ImagePlugin;
use bevy::prelude::PluginGroup;
use bevy::utils::default;
use bevy::window::{Window, WindowPlugin, WindowResolution};
use sandsim::component::grid_plugin::{ConfigResource, GridPlugin};
use sandsim::component::inputs;

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(300, 300),
                        canvas: Some("#window_canvas".to_string()),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(GridPlugin {
            config: ConfigResource::new(300, 300, 240.),
        })
        .add_systems(Startup, setup_camera)
        .add_systems(FixedUpdate, inputs::mouse_spawn_brush_system)
        .run();
}

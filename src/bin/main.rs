use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::image::ImagePlugin;
use bevy::prelude::PluginGroup;
use bevy::utils::default;
use bevy::window::{Window, WindowPlugin, WindowResolution};
use sandsim::component::grid_plugin::{ConfigResource, GridPlugin};
use sandsim::component::render::RenderSimPlugin;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(300, 330),
                        canvas: Some("#window_canvas".to_string()),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins(GridPlugin {
            config: ConfigResource::new(300, 300, 240.),
        })
        .add_plugins(RenderSimPlugin)
        .run();
}

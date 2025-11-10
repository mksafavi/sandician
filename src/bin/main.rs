use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::image::ImagePlugin;
use bevy::prelude::PluginGroup;
use bevy::utils::default;
use bevy::window::{Window, WindowPlugin, WindowResolution};
use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_embedded_assets::PluginMode;
use sandsim::component::grid_plugin::{ConfigResource, GridPlugin};
use sandsim::component::render::RenderSimPlugin;

fn main() {
    App::new()
        .add_plugins((
            EmbeddedAssetPlugin {
                mode: PluginMode::ReplaceDefault,
            },
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: WindowResolution::new(250, 420),
                        canvas: Some("#window_canvas".to_string()),
                        ..default()
                    }),
                    ..default()
                }),
        ))
        .add_plugins(GridPlugin {
            config: ConfigResource::new(250, 360, 240.),
        })
        .add_plugins(RenderSimPlugin)
        .run();
}

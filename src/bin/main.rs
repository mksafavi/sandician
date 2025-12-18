use bevy::window::{Window, WindowPlugin, WindowResolution};
use bevy::{DefaultPlugins, app::App, image::ImagePlugin, prelude::PluginGroup, utils::default};
use bevy_embedded_assets::{EmbeddedAssetPlugin, PluginMode};
use sandsim::component::{
    grid_plugin::{ConfigResource, GridPlugin},
    render::RenderSimPlugin,
};

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
            GridPlugin {
                config: ConfigResource::new(250, 360, 240., 50 * 128),
            },
            RenderSimPlugin,
        ))
        .run();
}

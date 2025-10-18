use bevy::{
    app::{App, Plugin, Startup},
    camera::Camera2d,
    ecs::system::Commands,
};

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

pub struct RenderSimPlugin;

impl Plugin for RenderSimPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
    }
}

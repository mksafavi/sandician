use bevy::{
    ecs::system::{Query, Res},
    input::{mouse::MouseButton, ButtonInput},
    window::Window,
};

use super::particle::{Grid, Particle};

pub fn mouse_spawn_brush_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    window_query: Query<&Window>,
    mut grid: Query<&mut Grid>,
) {
    let _cursor = match window_query.single() {
        Ok(w) => w.cursor_position(),
        Err(_) => None,
    };
    let mut g = grid.iter_mut().last().unwrap();
    if let Some(c) = _cursor {
        if mouse_button.pressed(MouseButton::Left) {
            g.spawn_brush(c.x as usize, c.y as usize, 25, Particle::Sand);
        };
        if mouse_button.pressed(MouseButton::Right) {
            g.spawn_brush(c.x as usize, c.y as usize, 25, Particle::Water);
        };
    };
}

use bevy::{
    ecs::system::{Query, Res},
    input::{mouse::MouseButton, ButtonInput},
    window::Window,
};
use bevy::{math::Vec2, window::WindowResolution};

use super::{
    grid_plugin::ConfigResource,
    particle::{Grid, Particle},
};

fn scale_input_position(
    cursor: Option<Vec2>,
    window_resolution: &WindowResolution,
    grid_size: (usize, usize),
) -> Option<(usize, usize)> {
    match cursor {
        Some(c) => {
            let rx = window_resolution.width() / grid_size.0 as f32;
            let ry = window_resolution.height() / grid_size.1 as f32;
            Some(((c.x / rx) as usize, (c.y / ry) as usize))
        }
        None => None,
    }
}

pub fn mouse_spawn_brush_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    config: Res<ConfigResource>,
    window_query: Query<&Window>,
    mut grid: Query<&mut Grid>,
) {
    let cursor_position = match window_query.single() {
        Ok(w) => scale_input_position(
            w.cursor_position(),
            &w.resolution,
            (config.width, config.height),
        ),
        Err(_) => None,
    };

    if let Ok(mut g) = grid.single_mut() {
        if let Some((x, y)) = cursor_position {
            if mouse_button.pressed(MouseButton::Left) {
                g.spawn_brush(x, y, 25, Particle::Sand);
            };
            if mouse_button.pressed(MouseButton::Right) {
                g.spawn_brush(x, y, 25, Particle::Water);
            };
            if mouse_button.pressed(MouseButton::Middle) {
                g.spawn_brush(x, y, 25, Particle::Salt);
            };
        };
    };
}

#[cfg(test)]
mod tests {

    use super::*;
    use bevy::{math::vec2, window::WindowResolution};

    #[test]
    fn test_scale_input_position_to_window_size_at_none() {
        assert_eq!(
            None,
            scale_input_position(None, &WindowResolution::new(100., 100.), (10, 10))
        );
    }

    #[test]
    fn test_scale_input_position_to_window_size_at_zero() {
        assert_eq!(
            Some((0, 0)),
            scale_input_position(
                Some(vec2(0., 0.)),
                &WindowResolution::new(100., 100.),
                (10, 10)
            )
        );
    }

    #[test]
    fn test_scale_input_position_to_window_size_window_bigger_than_grid() {
        assert_eq!(
            Some((5, 5)),
            scale_input_position(
                Some(vec2(50., 100.)),
                &WindowResolution::new(100., 200.),
                (10, 10)
            )
        );
    }

    #[test]
    fn test_scale_input_position_to_window_size_window_smaller_than_grid() {
        assert_eq!(
            Some((500, 500)),
            scale_input_position(
                Some(vec2(50., 100.)),
                &WindowResolution::new(100., 200.),
                (1000, 1000)
            )
        );
    }
}

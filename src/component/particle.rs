use bevy::{
    app::{App, FixedUpdate, Plugin, Startup, Update},
    asset::{Assets, Handle, RenderAssetUsages},
    color::{palettes::css, Color, ColorToPacked},
    ecs::{
        component::Component,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    image::Image,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::Sprite,
};
use rand::random_range;

#[derive(Resource, Clone)]
pub struct ConfigResource {
    width: usize,
    height: usize,
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

#[derive(Resource)]
struct OutputFrameHandle(Handle<Image>);

#[derive(Component, Clone, PartialEq, Debug)]
struct Position {
    x: usize,
    y: usize,
}

#[derive(Component, Clone, PartialEq, Debug)]
enum ParticleType {
    Sand,
    Water,
}

enum ParticleHorizontalDirection {
    Stay = 0,
    Left = -1,
    Right = 1,
}

enum ParticleVerticalDirection {
    Stay = 0,
    Bottom = 1,
}

#[derive(Component, Clone, PartialEq, Debug)]
struct Particle {
    position: Position,
    particle_type: ParticleType,
    simulated: bool,
}

impl Particle {
    fn new(x: usize, y: usize, particle_type: ParticleType) -> Self {
        Self {
            position: Position { x, y },
            particle_type,
            simulated: false,
        }
    }
}

#[derive(Component, Clone, PartialEq, Debug)]
struct Grid {
    cells: Vec<Option<Particle>>,
    width: usize,
    height: usize,
    random: fn() -> ParticleHorizontalDirection,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        fn random() -> ParticleHorizontalDirection {
            match random_range(0..=1) {
                0 => ParticleHorizontalDirection::Left,
                _ => ParticleHorizontalDirection::Right,
            }
        }
        Self {
            cells: (0..width * height).map(|_| None).collect(),
            width,
            height,
            random,
        }
    }

    fn new_with_rand(
        width: usize,
        height: usize,
        random: fn() -> ParticleHorizontalDirection,
    ) -> Self {
        Self {
            cells: (0..width * height).map(|_| None).collect(),
            width,
            height,
            random,
        }
    }

    fn spawn_particle(&mut self, p: Particle) {
        let index = self.width * p.position.y + p.position.x;
        if self.cells[index].is_none() {
            self.cells[index] = Some(p);
        }
    }

    fn update_grid(&mut self) {
        for y in (0..self.height).rev() {
            for x in 0..self.width {
                let index = y * self.width + x;
                if let Some(p) = &self.cells[index] {
                    if p.simulated {
                        continue;
                    }
                    let index_right = {
                        if (x + 1 < self.width) && self.cells[y * self.width + (x + 1)].is_none() {
                            Some(y * self.width + (x + 1))
                        } else {
                            None
                        }
                    };
                    let index_left = {
                        if (0 < x) && (self.cells[y * self.width + (x - 1)].is_none()) {
                            Some(y * self.width + (x - 1))
                        } else {
                            None
                        }
                    };
                    let index_bottom = {
                        if (y + 1 < self.height) && self.cells[(y + 1) * self.width + x].is_none() {
                            Some((y + 1) * self.width + x)
                        } else {
                            None
                        }
                    };
                    let index_bottom_right = {
                        if (y + 1 < self.height)
                            && (x + 1 < self.width)
                            && (self.cells[(y + 1) * self.width + (x + 1)].is_none())
                        {
                            Some((y + 1) * self.width + (x + 1))
                        } else {
                            None
                        }
                    };
                    let index_bottom_left = {
                        if (y + 1 < self.height)
                            && (0 < x)
                            && (self.cells[(y + 1) * self.width + (x - 1)].is_none())
                        {
                            Some((y + 1) * self.width + (x - 1))
                        } else {
                            None
                        }
                    };
                    let (next_location_index, direction) = match p.particle_type {
                        ParticleType::Sand => {
                            match (index_bottom_left, index_bottom, index_bottom_right) {
                                (None, None, None) => (None, None),
                                (_, Some(i), _) => (
                                    Some(i),
                                    Some((
                                        ParticleHorizontalDirection::Stay,
                                        ParticleVerticalDirection::Bottom,
                                    )),
                                ),
                                (None, None, Some(i)) => (
                                    Some(i),
                                    Some((
                                        ParticleHorizontalDirection::Right,
                                        ParticleVerticalDirection::Bottom,
                                    )),
                                ),
                                (Some(i), None, None) => (
                                    Some(i),
                                    Some((
                                        ParticleHorizontalDirection::Left,
                                        ParticleVerticalDirection::Bottom,
                                    )),
                                ),
                                (Some(l), None, Some(r)) => {
                                    let direction = (self.random)();
                                    let i = match direction {
                                        ParticleHorizontalDirection::Left => l,
                                        ParticleHorizontalDirection::Right => r,
                                        ParticleHorizontalDirection::Stay => 0,
                                    };
                                    (
                                        Some(i),
                                        Some((direction, ParticleVerticalDirection::Bottom)),
                                    )
                                }
                            }
                        }
                        ParticleType::Water => {
                            match (
                                index_left,
                                index_bottom_left,
                                index_bottom,
                                index_bottom_right,
                                index_right,
                            ) {
                                (None, None, None, None, None) => (None, None),
                                (_, _, Some(i), _, _) => (
                                    Some(i),
                                    Some((
                                        ParticleHorizontalDirection::Stay,
                                        ParticleVerticalDirection::Bottom,
                                    )),
                                ),
                                (_, None, None, Some(i), _) => (
                                    Some(i),
                                    Some((
                                        ParticleHorizontalDirection::Right,
                                        ParticleVerticalDirection::Bottom,
                                    )),
                                ),
                                (_, Some(i), None, None, _) => (
                                    Some(i),
                                    Some((
                                        ParticleHorizontalDirection::Left,
                                        ParticleVerticalDirection::Bottom,
                                    )),
                                ),
                                (_, Some(l), None, Some(r), _) => {
                                    let direction = (self.random)();
                                    let i = match direction {
                                        ParticleHorizontalDirection::Left => l,
                                        ParticleHorizontalDirection::Right => r,
                                        ParticleHorizontalDirection::Stay => 0,
                                    };
                                    (
                                        Some(i),
                                        Some((direction, ParticleVerticalDirection::Bottom)),
                                    )
                                }
                                (None, None, None, None, Some(i)) => (
                                    Some(i),
                                    Some((
                                        ParticleHorizontalDirection::Right,
                                        ParticleVerticalDirection::Stay,
                                    )),
                                ),
                                (Some(i), None, None, None, None) => (
                                    Some(i),
                                    Some((
                                        ParticleHorizontalDirection::Left,
                                        ParticleVerticalDirection::Stay,
                                    )),
                                ),
                                (Some(l), None, None, None, Some(r)) => {
                                    let direction = (self.random)();
                                    let i = match direction {
                                        ParticleHorizontalDirection::Left => l,
                                        ParticleHorizontalDirection::Right => r,
                                        ParticleHorizontalDirection::Stay => 0,
                                    };
                                    (Some(i), Some((direction, ParticleVerticalDirection::Stay)))
                                }
                            }
                        }
                    };

                    if let (Some(i), Some((hd, vd))) = (next_location_index, direction) {
                        self.cells[i] = Some({
                            let mut np = p.clone();
                            np.simulated = true;
                            np.position.y += vd as usize;
                            np.position.x = (np.position.x as isize + hd as isize) as usize;
                            np
                        });
                        self.cells[index] = None;
                    };
                }
            }
        }
        self.cells.iter_mut().for_each(|x| {
            if let Some(x) = x {
                x.simulated = false
            }
        });
    }

    fn create_output_frame(width: usize, height: usize) -> Image {
        Image::new_fill(
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &(css::BLACK.to_u8_array()),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        )
    }

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
        if let Some(mut g) = grid.iter_mut().last() {
            g.update_grid();
        }
    }

    fn draw_grid(&self, image: &mut Image) {
        for (index, particle) in self.cells.iter().enumerate() {
            match particle {
                Some(p) => match p.particle_type {
                    ParticleType::Sand => image
                        .set_color_at(
                            p.position.x as u32,
                            p.position.y as u32,
                            Color::srgb(1., 1., 1.),
                        )
                        .expect("temp: TODO: panic"),
                    ParticleType::Water => image
                        .set_color_at(
                            p.position.x as u32,
                            p.position.y as u32,
                            Color::srgb(0., 0., 1.),
                        )
                        .expect("temp: TODO: panic"),
                },
                _ => {
                    let x: u32 = index as u32 % self.width as u32;
                    let y: u32 = (index as u32 - x) / self.width as u32;
                    image
                        .set_color_at(x, y, Color::srgb(0., 0., 0.))
                        .expect("temp: TODO: panic");
                }
            }
        }
    }

    fn draw_grid_system(
        grid: Query<&Grid>,
        output_frame_handle: Res<OutputFrameHandle>,
        mut images: ResMut<Assets<Image>>,
    ) {
        let image = images
            .get_mut(&output_frame_handle.0)
            .expect("Image not found");

        match grid.iter().last() {
            Some(g) => g.draw_grid(image),
            None => todo!(),
        }
    }
}

#[cfg(test)]
mod tests_grid {

    use super::*;

    use bevy::app::{App, Startup, Update};
    use bevy::prelude::IntoScheduleConfigs;

    #[test]
    fn test_create_grid() {
        let g = Grid::new(2, 3);
        assert_eq!(6, g.cells.len());
        assert_eq!(2, g.width);
        assert_eq!(3, g.height);
    }

    #[test]
    fn test_grid_spawn_particle_to_grid() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle(Particle::new(0, 0, ParticleType::Sand));

        g.spawn_particle(Particle::new(1, 1, ParticleType::Water));

        match &g.cells[0] {
            Some(p) => assert_eq!(ParticleType::Sand, p.particle_type),
            None => panic!(),
        }

        match &g.cells[3] {
            Some(p) => assert_eq!(ParticleType::Water, p.particle_type),
            None => panic!(),
        }
    }

    #[test]
    fn test_grid_spawn_particle_to_non_empty_location_silently_fails() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle(Particle::new(0, 0, ParticleType::Sand));

        g.spawn_particle(Particle::new(0, 0, ParticleType::Water));

        match &g.cells[0] {
            Some(p) => assert_eq!(ParticleType::Sand, p.particle_type),
            None => panic!(),
        }
    }

    #[test]
    fn test_init_grid_system_creates_a_n_by_m_grid() {
        let mut app = App::new();
        app.insert_resource(ConfigResource::new(2, 3, 100.));
        app.init_resource::<Assets<Image>>();
        app.add_systems(Startup, Grid::init_grid_system);
        app.update();
        assert_eq!(1, app.world_mut().query::<&Grid>().iter(app.world()).len());
        for g in app.world_mut().query::<&Grid>().iter(app.world()) {
            assert_eq!(2, g.width);
            assert_eq!(3, g.height);
            for p in &g.cells {
                assert!(p.is_none());
            }
        }
    }

    #[test]
    fn test_update_grid_sand_falling_down_at_last_row_stays_there() {
        let mut g = Grid::new(2, 2);
        g.spawn_particle(Particle::new(0, 1, ParticleType::Sand));

        g.update_grid(); /* should stay at the last line*/
        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Particle::new(0, 1, ParticleType::Sand)), g.cells[2]);
        assert_eq!(None, g.cells[3]);
    }

    #[test]
    fn test_update_grid_sand_falls_down_when_bottom_cell_is_empty() {
        let mut g = Grid::new(2, 2);

        g.spawn_particle(Particle::new(0, 0, ParticleType::Sand));

        assert_eq!(Some(Particle::new(0, 0, ParticleType::Sand)), g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Particle::new(0, 1, ParticleType::Sand)), g.cells[2]);
        assert_eq!(None, g.cells[3]);
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_right_when_bottom_cell_is_full_and_bottom_left_is_wall_and_bottom_right_is_empty(
    ) {
        let mut g = Grid::new(2, 2);

        g.spawn_particle(Particle::new(0, 0, ParticleType::Sand));

        g.spawn_particle(Particle::new(0, 1, ParticleType::Sand));

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Particle::new(0, 1, ParticleType::Sand)), g.cells[2]);
        assert_eq!(Some(Particle::new(1, 1, ParticleType::Sand)), g.cells[3]);
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_left_when_bottom_cell_is_full_and_bottom_right_is_wall_and_bottom_left_is_empty(
    ) {
        let mut g = Grid::new(2, 2);

        g.spawn_particle(Particle::new(1, 0, ParticleType::Sand));

        g.spawn_particle(Particle::new(1, 1, ParticleType::Sand));

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Particle::new(0, 1, ParticleType::Sand)), g.cells[2]);
        assert_eq!(Some(Particle::new(1, 1, ParticleType::Sand)), g.cells[3]);
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_for_testing_forced_left(
    ) {
        let mut g = Grid::new_with_rand(3, 2, || ParticleHorizontalDirection::Left);

        g.spawn_particle(Particle::new(1, 0, ParticleType::Sand));

        g.spawn_particle(Particle::new(1, 1, ParticleType::Sand));

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Particle::new(0, 1, ParticleType::Sand)), g.cells[3]);
        assert_eq!(Some(Particle::new(1, 1, ParticleType::Sand)), g.cells[4]);
        assert_eq!(None, g.cells[5]);
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_for_testing_forced_right(
    ) {
        let mut g = Grid::new_with_rand(3, 2, || ParticleHorizontalDirection::Right);

        g.spawn_particle(Particle::new(1, 0, ParticleType::Sand));

        g.spawn_particle(Particle::new(1, 1, ParticleType::Sand));

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);
        assert_eq!(Some(Particle::new(1, 1, ParticleType::Sand)), g.cells[4]);
        assert_eq!(Some(Particle::new(2, 1, ParticleType::Sand)), g.cells[5]);
    }

    #[test]
    fn test_update_grid_water_moves_right_when_bottom_cell_and_left_is_full_and_right_cell_is_empty(
    ) {
        let mut g = Grid::new(3, 2);

        g.spawn_particle(Particle::new(0, 1, ParticleType::Sand));

        g.spawn_particle(Particle::new(1, 1, ParticleType::Water));

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Particle::new(0, 1, ParticleType::Sand)), g.cells[3]);
        assert_eq!(None, g.cells[4]);
        assert_eq!(Some(Particle::new(2, 1, ParticleType::Water)), g.cells[5]);
    }

    #[test]
    fn test_update_grid_water_moves_left_when_bottom_cell_and_right_is_full_and_left_cell_is_empty()
    {
        let mut g = Grid::new(3, 2);

        g.spawn_particle(Particle::new(1, 1, ParticleType::Water));

        g.spawn_particle(Particle::new(2, 1, ParticleType::Sand));

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Particle::new(0, 1, ParticleType::Water)), g.cells[3]);
        assert_eq!(None, g.cells[4]);
        assert_eq!(Some(Particle::new(2, 1, ParticleType::Sand)), g.cells[5]);
    }

    #[test]
    fn test_update_grid_water_moves_left_or_right_when_bottom_cell_is_empty_and_both_right_and_left_are_empty_forced_right(
    ) {
        let mut g = Grid::new_with_rand(3, 2, || ParticleHorizontalDirection::Right);

        g.spawn_particle(Particle::new(1, 1, ParticleType::Water));

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);
        assert_eq!(None, g.cells[4]);
        assert_eq!(Some(Particle::new(2, 1, ParticleType::Water)), g.cells[5]);
    }

    #[test]
    fn test_update_grid_water_moves_left_or_right_when_bottom_cell_is_empty_and_both_right_and_left_are_empty_forced_left(
    ) {
        let mut g = Grid::new_with_rand(3, 2, || ParticleHorizontalDirection::Left);

        g.spawn_particle(Particle::new(1, 1, ParticleType::Water));

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Particle::new(0, 1, ParticleType::Water)), g.cells[3]);
        assert_eq!(None, g.cells[4]);
        assert_eq!(None, g.cells[5]);
    }

    #[test]
    fn test_draw_grid() {
        let mut g = Grid::new(2, 2);
        let mut image = Grid::create_output_frame(2, 2);
        g.draw_grid(&mut image);
        assert_eq!(
            vec![
                Color::srgb(0., 0., 0.),
                Color::srgb(0., 0., 0.),
                Color::srgb(0., 0., 0.),
                Color::srgb(0., 0., 0.)
            ],
            vec![
                image.get_color_at(0, 0).unwrap(),
                image.get_color_at(1, 0).unwrap(),
                image.get_color_at(0, 1).unwrap(),
                image.get_color_at(1, 1).unwrap()
            ]
        );

        g.spawn_particle(Particle::new(0, 0, ParticleType::Sand));
        g.draw_grid(&mut image);
        assert_eq!(
            vec![
                Color::srgb(1., 1., 1.),
                Color::srgb(0., 0., 0.),
                Color::srgb(0., 0., 0.),
                Color::srgb(0., 0., 0.)
            ],
            vec![
                image.get_color_at(0, 0).unwrap(),
                image.get_color_at(1, 0).unwrap(),
                image.get_color_at(0, 1).unwrap(),
                image.get_color_at(1, 1).unwrap()
            ]
        );
        g.spawn_particle(Particle::new(1, 0, ParticleType::Sand));
        g.cells[0] = None;
        g.draw_grid(&mut image);
        assert_eq!(
            vec![
                Color::srgb(0., 0., 0.),
                Color::srgb(1., 1., 1.),
                Color::srgb(0., 0., 0.),
                Color::srgb(0., 0., 0.)
            ],
            vec![
                image.get_color_at(0, 0).unwrap(),
                image.get_color_at(1, 0).unwrap(),
                image.get_color_at(0, 1).unwrap(),
                image.get_color_at(1, 1).unwrap()
            ]
        );
    }

    #[test]
    fn test_draw_grid_system() {
        fn fixture_spawn_particle_system(mut grid: Query<&mut Grid>) {
            let mut g = grid.iter_mut().last().unwrap();
            g.spawn_particle(Particle::new(0, 0, ParticleType::Sand));
            g.spawn_particle(Particle::new(1, 0, ParticleType::Water));
        }

        fn assert_read_output_frame_system(
            output_frame_handle: Res<OutputFrameHandle>,
            images: ResMut<Assets<Image>>,
        ) {
            let image = images.get(&output_frame_handle.0).expect("Image not found");
            assert_eq!(
                vec![
                    Color::srgb(1., 1., 1.),
                    Color::srgb(0., 0., 1.),
                    Color::srgb(0., 0., 0.),
                    Color::srgb(0., 0., 0.)
                ],
                vec![
                    image.get_color_at(0, 0).unwrap(),
                    image.get_color_at(1, 0).unwrap(),
                    image.get_color_at(0, 1).unwrap(),
                    image.get_color_at(1, 1).unwrap()
                ]
            );
        }

        let mut app = App::new();
        app.insert_resource(ConfigResource::new(5, 6, 100.));
        app.init_resource::<Assets<Image>>();

        //app.insert_resource(OutputFrameHandle);
        app.add_systems(Startup, Grid::init_grid_system);
        app.add_systems(
            Update,
            (
                fixture_spawn_particle_system,
                Grid::draw_grid_system,
                assert_read_output_frame_system,
            )
                .chain(),
        );
        app.update();
    }
}

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
    time::{Fixed, Time},
};
use rand::random_range;

// TODO: remove all unwrap and expects

pub struct GridPlugin {
    pub config: ConfigResource,
}
impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(self.config.clone())
            .insert_resource(Time::<Fixed>::from_hz(self.config.update_rate))
            .add_systems(Startup, Grid::init_grid_system)
            .add_systems(FixedUpdate, Grid::update_grid_system)
            .add_systems(Update, Grid::draw_grid_system);
    }
}

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

#[derive(Clone, Debug)]
enum ParticleHorizontalDirection {
    Stay = 0,
    Left = -1,
    Right = 1,
}

#[derive(Clone, Debug)]
enum ParticleVerticalDirection {
    Stay = 0,
    Bottom = 1,
}

enum RowUpdateDirection {
    Forward = 0,
    Reverse = 1,
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
    water_direction: fn() -> ParticleHorizontalDirection,
    row_update_direction: fn() -> RowUpdateDirection,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        fn random_water_direction() -> ParticleHorizontalDirection {
            match random_range(0..=1) {
                0 => ParticleHorizontalDirection::Left,
                _ => ParticleHorizontalDirection::Right,
            }
        }
        fn random_row_update_direction() -> RowUpdateDirection {
            match random_range(0..=1) {
                0 => RowUpdateDirection::Forward,
                _ => RowUpdateDirection::Reverse,
            }
        }
        Self {
            cells: (0..width * height).map(|_| None).collect(),
            width,
            height,
            water_direction: random_water_direction,
            row_update_direction: random_row_update_direction,
        }
    }

    fn spawn_particle(&mut self, p: Particle) {
        if p.position.y < self.height && p.position.x < self.width {
            let index = self.width * p.position.y + p.position.x;
            if self.cells[index].is_none() {
                self.cells[index] = Some(p);
            }
        }
    }

    fn find_sand_particle_next_direction(
        &self,
        x: usize,
        y: usize,
    ) -> (
        Option<usize>,
        Option<(ParticleHorizontalDirection, ParticleVerticalDirection)>,
    ) {
        let index_bottom = {
            if y + 1 < self.height {
                match &self.cells[(y + 1) * self.width + x] {
                    Some(p) => match p.particle_type {
                        ParticleType::Sand => None,
                        ParticleType::Water => match p.simulated {
                            false => Some((y + 1) * self.width + x),
                            true => None,
                        },
                    },
                    None => Some((y + 1) * self.width + x),
                }
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
                let direction = (self.water_direction)();
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

    fn find_water_particle_next_direction(
        &self,
        x: usize,
        y: usize,
    ) -> (
        Option<usize>,
        Option<(ParticleHorizontalDirection, ParticleVerticalDirection)>,
    ) {
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
                let direction = (self.water_direction)();
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
                let direction = (self.water_direction)();
                let i = match direction {
                    ParticleHorizontalDirection::Left => l,
                    ParticleHorizontalDirection::Right => r,
                    ParticleHorizontalDirection::Stay => 0,
                };
                (Some(i), Some((direction, ParticleVerticalDirection::Stay)))
            }
        }
    }

    fn update_grid(&mut self) {
        for y in (0..self.height).rev() {
            let it = match (self.row_update_direction)() {
                RowUpdateDirection::Forward => (0..self.width).collect::<Vec<_>>(),
                RowUpdateDirection::Reverse => (0..self.width).rev().collect::<Vec<_>>(),
            };
            for x in it {
                let index = y * self.width + x;
                let (next_location_index, direction) = match &self.cells[index] {
                    Some(p) => {
                        if p.simulated {
                            continue;
                        }
                        match p.particle_type {
                            ParticleType::Sand => self.find_sand_particle_next_direction(x, y),
                            ParticleType::Water => self.find_water_particle_next_direction(x, y),
                        }
                    }
                    None => (None, None),
                };
                if let (Some(new_index), Some((hd, vd))) = (next_location_index, direction) {
                    self.cells.swap(index, new_index);
                    if let Some(p) = &mut self.cells[index] {
                        p.simulated = true;
                        p.position.y -= vd.clone() as usize;
                        p.position.x = (p.position.x as isize - hd.clone() as isize) as usize;
                    };

                    if let Some(p) = &mut self.cells[new_index] {
                        p.simulated = true;
                        p.position.y += vd as usize;
                        p.position.x = (p.position.x as isize + hd as isize) as usize;
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

    impl Grid {
        fn new_with_rand(
            width: usize,
            height: usize,
            water_direction: Option<fn() -> ParticleHorizontalDirection>,
            row_update_direction: Option<fn() -> RowUpdateDirection>,
        ) -> Self {
            let mut g = Self::new(width, height);
            g.water_direction = match water_direction {
                Some(f) => f,
                None => || ParticleHorizontalDirection::Stay,
            };

            g.row_update_direction = match row_update_direction {
                Some(f) => f,
                None => || RowUpdateDirection::Forward,
            };
            g
        }
    }

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
    fn test_grid_spawn_particle_out_of_grid_bound_silently_fails() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle(Particle::new(0, 3, ParticleType::Sand));
        g.spawn_particle(Particle::new(2, 0, ParticleType::Water));

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);
        assert_eq!(None, g.cells[4]);
        assert_eq!(None, g.cells[5]);
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
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

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
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

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
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

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
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Left), None);

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
    fn test_updating_rows_in_forward_order_creates_a_left_bias_on_water() {
        /*
         * updating in forward: -ww- => ww-- or w--w
         */
        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Left),
            Some(|| RowUpdateDirection::Forward),
        );

        g.spawn_particle(Particle::new(1, 0, ParticleType::Water));
        g.spawn_particle(Particle::new(2, 0, ParticleType::Water));

        g.update_grid();

        assert_eq!(Some(Particle::new(0, 0, ParticleType::Water)), g.cells[0]);
        assert_eq!(Some(Particle::new(1, 0, ParticleType::Water)), g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);

        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Right),
            Some(|| RowUpdateDirection::Forward),
        );

        g.spawn_particle(Particle::new(1, 0, ParticleType::Water));
        g.spawn_particle(Particle::new(2, 0, ParticleType::Water));

        g.update_grid();

        assert_eq!(Some(Particle::new(0, 0, ParticleType::Water)), g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Particle::new(3, 0, ParticleType::Water)), g.cells[3]);
    }

    #[test]
    fn test_updating_rows_in_reverse_order_creates_a_right_bias_on_water() {
        /*
         * updating in reverse: -ww- => --ww or w--w
         */
        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Right),
            Some(|| RowUpdateDirection::Reverse),
        );

        g.spawn_particle(Particle::new(1, 0, ParticleType::Water));
        g.spawn_particle(Particle::new(2, 0, ParticleType::Water));

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(Some(Particle::new(2, 0, ParticleType::Water)), g.cells[2]);
        assert_eq!(Some(Particle::new(3, 0, ParticleType::Water)), g.cells[3]);

        let mut g = Grid::new_with_rand(
            4,
            1,
            Some(|| ParticleHorizontalDirection::Left),
            Some(|| RowUpdateDirection::Reverse),
        );

        g.spawn_particle(Particle::new(1, 0, ParticleType::Water));
        g.spawn_particle(Particle::new(2, 0, ParticleType::Water));

        g.update_grid();

        assert_eq!(Some(Particle::new(0, 0, ParticleType::Water)), g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(Some(Particle::new(3, 0, ParticleType::Water)), g.cells[3]);
    }

    #[test]
    fn test_sand_should_sink_in_water() {
        let mut g = Grid::new_with_rand(3, 2, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(Particle::new(1, 0, ParticleType::Sand));
        g.spawn_particle(Particle::new(0, 1, ParticleType::Sand));
        g.spawn_particle(Particle::new(1, 1, ParticleType::Water));
        g.spawn_particle(Particle::new(2, 1, ParticleType::Sand));

        g.update_grid();

        assert_eq!(
            vec![
                None,
                Some(Particle::new(1, 0, ParticleType::Water)),
                None,
                Some(Particle::new(0, 1, ParticleType::Sand)),
                Some(Particle::new(1, 1, ParticleType::Sand)),
                Some(Particle::new(2, 1, ParticleType::Sand)),
            ],
            g.cells
        );
    }

    #[test]
    fn test_sand_should_sink_in_water_but_water_shouldnot_climb_sands() {
        let mut g = Grid::new_with_rand(1, 3, Some(|| ParticleHorizontalDirection::Right), None);

        g.spawn_particle(Particle::new(0, 0, ParticleType::Sand));
        g.spawn_particle(Particle::new(0, 1, ParticleType::Sand));
        g.spawn_particle(Particle::new(0, 2, ParticleType::Water));

        assert_eq!(
            vec![
                Some(Particle::new(0, 0, ParticleType::Sand)),
                Some(Particle::new(0, 1, ParticleType::Sand)),
                Some(Particle::new(0, 2, ParticleType::Water)),
            ],
            g.cells
        );

        g.update_grid();

        assert_eq!(
            vec![
                Some(Particle::new(0, 0, ParticleType::Sand)),
                Some(Particle::new(0, 1, ParticleType::Water)),
                Some(Particle::new(0, 2, ParticleType::Sand)),
            ],
            g.cells
        );

        g.update_grid();

        assert_eq!(
            vec![
                Some(Particle::new(0, 0, ParticleType::Water)),
                Some(Particle::new(0, 1, ParticleType::Sand)),
                Some(Particle::new(0, 2, ParticleType::Sand)),
            ],
            g.cells
        );
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

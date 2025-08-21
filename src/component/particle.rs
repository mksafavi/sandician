use bevy::{
    app::{App, Startup, Update},
    asset::{Assets, Handle, RenderAssetUsages},
    color::{palettes::css, Color, ColorToPacked},
    ecs::{
        component::Component,
        resource::Resource,
        system::{Commands, Query, Res, ResMut},
    },
    image::Image,
    prelude::IntoScheduleConfigs,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
    sprite::Sprite,
};
use rand::random_range;

#[derive(Resource)]
struct ConfigResource {
    width: usize,
    height: usize,
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

enum ParticleDirection {
    BottomLeft = -1,
    BottomRight = 1,
}

#[derive(Component, Clone, PartialEq, Debug)]
struct Particle {
    position: Position,
    particle_type: ParticleType,
}

#[derive(Component, Clone, PartialEq, Debug)]
struct Grid {
    cells: Vec<Option<Particle>>,
    width: usize,
    height: usize,
    random: fn() -> ParticleDirection,
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        fn random() -> ParticleDirection {
            match random_range(0..=1) {
                0 => ParticleDirection::BottomLeft,
                _ => ParticleDirection::BottomRight,
            }
        }
        Self {
            cells: (0..width * height).map(|_| None).collect(),
            width: width,
            height: height,
            random: random,
        }
    }

    fn new_with_rand(width: usize, height: usize, random: fn() -> ParticleDirection) -> Self {
        Self {
            cells: (0..width * height).map(|_| None).collect(),
            width: width,
            height: height,
            random: random,
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
                    if y + 1 == self.height {
                        continue;
                    }
                    let index_bottom = {
                        if y + 1 < self.height {
                            if self.cells[(y + 1) * self.width + x].is_none() {
                                Some((y + 1) * self.width + x)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };
                    let index_bottom_right = {
                        if x + 1 < self.width {
                            if self.cells[(y + 1) * self.width + (x + 1)].is_none() {
                                Some((y + 1) * self.width + (x + 1))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };
                    let index_bottom_left = {
                        if 0 <= x as isize - 1 {
                            if self.cells[(y + 1) * self.width + (x - 1)].is_none() {
                                Some((y + 1) * self.width + (x - 1))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    match (index_bottom_left, index_bottom, index_bottom_right) {
                        (None, None, None) => (),
                        (_, Some(i), _) => {
                            self.cells[i] = Some({
                                let mut np = p.clone();
                                np.position.y = np.position.y + 1;
                                np
                            });
                            self.cells[index] = None;
                        }
                        (None, None, Some(i)) => {
                            self.cells[i] = Some({
                                let mut np = p.clone();
                                np.position.y = np.position.y + 1;
                                np.position.x = np.position.x + 1;
                                np
                            });
                            self.cells[index] = None;
                        }
                        (Some(i), None, None) => {
                            self.cells[i] = Some({
                                let mut np = p.clone();
                                np.position.y = np.position.y + 1;
                                np.position.x = np.position.x - 1;
                                np
                            });
                            self.cells[index] = None;
                        }
                        (Some(l), None, Some(r)) => {
                            let direction = (self.random)();
                            let i = match direction {
                                ParticleDirection::BottomLeft => l,
                                ParticleDirection::BottomRight => r,
                            };
                            self.cells[i] = Some({
                                let mut np = p.clone();
                                np.position.y = np.position.y + 1;
                                np.position.x =
                                    (np.position.x as isize + direction as isize) as usize;
                                np
                            });
                            self.cells[index] = None;
                        }
                    }
                }
            }
        }
    }

    fn create_output_frame(width: usize, height: usize) -> Image {
        let image = Image::new_fill(
            Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            &(css::BLACK.to_u8_array()),
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        );
        image
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
mod tests {
    use super::*;

    #[test]
    fn test_particle_entity_has_position() {
        let p = Particle {
            position: Position { x: 10, y: 20 },
            particle_type: ParticleType::Sand,
        };
        assert_eq!((10, 20), (p.position.x, p.position.y));
    }

    #[test]
    fn test_particle_entity_can_have_particle_type_of_sand() {
        let p = Particle {
            position: Position { x: 10, y: 20 },
            particle_type: ParticleType::Sand,
        };
        assert_eq!(ParticleType::Sand, p.particle_type);
    }

    #[test]
    fn test_particle_entity_can_have_particle_type_of_water() {
        let p = Particle {
            position: Position { x: 10, y: 20 },
            particle_type: ParticleType::Water,
        };
        assert_eq!(ParticleType::Water, p.particle_type);
    }
}

#[cfg(test)]
mod tests_grid {

    use super::*;

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
        g.spawn_particle(Particle {
            position: Position { x: 0, y: 0 },
            particle_type: ParticleType::Sand,
        });

        g.spawn_particle(Particle {
            position: Position { x: 1, y: 1 },
            particle_type: ParticleType::Water,
        });

        match &g.cells[0] {
            Some(p) => assert_eq!(ParticleType::Sand, p.particle_type),
            None => assert!(false),
        }

        match &g.cells[3] {
            Some(p) => assert_eq!(ParticleType::Water, p.particle_type),
            None => assert!(false),
        }
    }

    #[test]
    fn test_grid_spawn_particle_to_non_empty_location_silently_fails() {
        let mut g = Grid::new(2, 3);
        g.spawn_particle(Particle {
            position: Position { x: 0, y: 0 },
            particle_type: ParticleType::Sand,
        });

        g.spawn_particle(Particle {
            position: Position { x: 0, y: 0 },
            particle_type: ParticleType::Water,
        });

        match &g.cells[0] {
            Some(p) => assert_eq!(ParticleType::Sand, p.particle_type),
            None => assert!(false),
        }
    }

    #[test]
    fn test_init_grid_system_creates_a_n_by_m_grid() {
        let mut app = App::new();
        app.insert_resource(ConfigResource {
            width: 2,
            height: 3,
        });
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
        g.spawn_particle(Particle {
            position: Position { x: 0, y: 1 },
            particle_type: ParticleType::Sand,
        });

        g.update_grid(); /* should stay at the last line*/
        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(
            Some(Particle {
                position: Position { x: 0, y: 1 },
                particle_type: ParticleType::Sand,
            }),
            g.cells[2]
        );
        assert_eq!(None, g.cells[3]);
    }

    #[test]
    fn test_update_grid_sand_falls_down_when_bottom_cell_is_empty() {
        let mut g = Grid::new(2, 2);

        g.spawn_particle(Particle {
            position: Position { x: 0, y: 0 },
            particle_type: ParticleType::Sand,
        });

        assert_eq!(
            Some(Particle {
                position: Position { x: 0, y: 0 },
                particle_type: ParticleType::Sand,
            }),
            g.cells[0]
        );
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(
            Some(Particle {
                position: Position { x: 0, y: 1 },
                particle_type: ParticleType::Sand,
            }),
            g.cells[2]
        );
        assert_eq!(None, g.cells[3]);
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_right_when_bottom_cell_is_full_and_bottom_left_is_wall_and_bottom_right_is_empty(
    ) {
        let mut g = Grid::new(2, 2);

        g.spawn_particle(Particle {
            position: Position { x: 0, y: 0 },
            particle_type: ParticleType::Sand,
        });

        g.spawn_particle(Particle {
            position: Position { x: 0, y: 1 },
            particle_type: ParticleType::Sand,
        });

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(
            Some(Particle {
                position: Position { x: 0, y: 1 },
                particle_type: ParticleType::Sand,
            }),
            g.cells[2]
        );
        assert_eq!(
            Some(Particle {
                position: Position { x: 1, y: 1 },
                particle_type: ParticleType::Sand,
            }),
            g.cells[3]
        );
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_left_when_bottom_cell_is_full_and_bottom_right_is_wall_and_bottom_left_is_empty(
    ) {
        let mut g = Grid::new(2, 2);

        g.spawn_particle(Particle {
            position: Position { x: 1, y: 0 },
            particle_type: ParticleType::Sand,
        });

        g.spawn_particle(Particle {
            position: Position { x: 1, y: 1 },
            particle_type: ParticleType::Sand,
        });

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(
            Some(Particle {
                position: Position { x: 0, y: 1 },
                particle_type: ParticleType::Sand,
            }),
            g.cells[2]
        );
        assert_eq!(
            Some(Particle {
                position: Position { x: 1, y: 1 },
                particle_type: ParticleType::Sand,
            }),
            g.cells[3]
        );
    }

    #[test]
    fn test_update_grid_sand_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_for_testing_forced_left(
    ) {
        let mut g = Grid::new_with_rand(3, 2, || ParticleDirection::BottomLeft);

        g.spawn_particle(Particle {
            position: Position { x: 1, y: 0 },
            particle_type: ParticleType::Sand,
        });

        g.spawn_particle(Particle {
            position: Position { x: 1, y: 1 },
            particle_type: ParticleType::Sand,
        });

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(
            Some(Particle {
                position: Position { x: 0, y: 1 },
                particle_type: ParticleType::Sand,
            }),
            g.cells[3]
        );
        assert_eq!(
            Some(Particle {
                position: Position { x: 1, y: 1 },
                particle_type: ParticleType::Sand,
            }),
            g.cells[4]
        );
        assert_eq!(None, g.cells[5]);
    }
    #[test]
    fn test_update_grid_sand_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_for_testing_forced_right(
    ) {
        let mut g = Grid::new_with_rand(3, 2, || ParticleDirection::BottomRight);

        g.spawn_particle(Particle {
            position: Position { x: 1, y: 0 },
            particle_type: ParticleType::Sand,
        });

        g.spawn_particle(Particle {
            position: Position { x: 1, y: 1 },
            particle_type: ParticleType::Sand,
        });

        g.update_grid();

        assert_eq!(None, g.cells[0]);
        assert_eq!(None, g.cells[1]);
        assert_eq!(None, g.cells[2]);
        assert_eq!(None, g.cells[3]);
        assert_eq!(
            Some(Particle {
                position: Position { x: 1, y: 1 },
                particle_type: ParticleType::Sand,
            }),
            g.cells[4]
        );
        assert_eq!(
            Some(Particle {
                position: Position { x: 2, y: 1 },
                particle_type: ParticleType::Sand,
            }),
            g.cells[5]
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

        g.spawn_particle(Particle {
            position: Position { x: 0, y: 0 },
            particle_type: ParticleType::Sand,
        });
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
        g.spawn_particle(Particle {
            position: Position { x: 1, y: 0 },
            particle_type: ParticleType::Sand,
        });
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
            g.spawn_particle(Particle {
                position: Position { x: 0, y: 0 },
                particle_type: ParticleType::Sand,
            });
            g.spawn_particle(Particle {
                position: Position { x: 1, y: 0 },
                particle_type: ParticleType::Water,
            });
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
        app.insert_resource(ConfigResource {
            width: 5,
            height: 6,
        });
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

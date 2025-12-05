use core::fmt;

use bevy::{
    color::{ColorToPacked, Hsva, Srgba},
    prelude::{Color, Saturation},
};

use crate::component::grid::{GridAccess, ParticleHorizontalDirection};

use super::{acid::Acid, drain::Drain, rock::Rock, salt::Salt, sand::Sand, tap::Tap, water::Water};

#[derive(Clone, PartialEq, Debug)]
pub enum ParticleKind {
    Sand(Sand),
    Water(Water),
    Salt(Salt),
    Rock(Rock),
    Drain(Drain),
    Tap(Tap),
    Acid(Acid),
}

impl ParticleKind {
    pub fn to_default(&self) -> Self {
        match self {
            ParticleKind::Sand(..) => Self::from(Sand::new()),
            ParticleKind::Water(..) => Self::from(Water::new()),
            ParticleKind::Salt(..) => Self::from(Salt::new()),
            ParticleKind::Rock(..) => Self::from(Rock::new()),
            ParticleKind::Drain(..) => Self::from(Drain::new()),
            ParticleKind::Tap(..) => Self::from(Tap::new()),
            ParticleKind::Acid(..) => Self::from(Acid::new()),
        }
    }
}

impl From<Sand> for ParticleKind {
    fn from(sand: Sand) -> Self {
        Self::Sand(sand)
    }
}

impl From<Salt> for ParticleKind {
    fn from(salt: Salt) -> Self {
        Self::Salt(salt)
    }
}

impl From<Water> for ParticleKind {
    fn from(water: Water) -> Self {
        Self::Water(water)
    }
}

impl From<Rock> for ParticleKind {
    fn from(rock: Rock) -> Self {
        Self::Rock(rock)
    }
}

impl From<Drain> for ParticleKind {
    fn from(drain: Drain) -> Self {
        Self::Drain(drain)
    }
}

impl From<Tap> for ParticleKind {
    fn from(tap: Tap) -> Self {
        Self::Tap(tap)
    }
}

impl From<Acid> for ParticleKind {
    fn from(acid: Acid) -> Self {
        Self::Acid(acid)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Particle {
    pub weight: u8,
    viscosity: u8,
    pub cloneable: bool,
    color: [u8; 3],
    pub kind: ParticleKind,
    pub seed: u8,
    pub velocity: u8,
    pub health: u8,
}

impl Particle {
    pub fn new(color: Color, kind: ParticleKind) -> Self {
        Self {
            weight: u8::MIN,
            viscosity: u8::MAX,
            cloneable: true,
            color: color.to_srgba().to_u8_array_no_alpha(),
            kind,
            seed: 127,
            velocity: u8::MAX,
            health: u8::MAX,
        }
    }

    pub fn with_weight(mut self, weight: u8) -> Self {
        self.weight = weight;
        self
    }
    pub fn with_cloneable(mut self, cloneable: bool) -> Self {
        self.cloneable = cloneable;
        self
    }

    pub fn with_viscosity(mut self, viscosity: u8) -> Self {
        self.viscosity = viscosity;
        self
    }

    pub fn with_seed(mut self, seed: u8) -> Self {
        self.seed = seed;
        let color: Hsva = self.color().into();
        let color: Srgba = color
            .with_value(color.value + ((self.seed as f32) - 127.0) / (255.0 * 10.0))
            .into();
        self.color = color.to_u8_array_no_alpha();
        self
    }

    pub fn with_velocity(mut self, velocity: u8) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_health(mut self, health: u8) -> Self {
        self.health = health;
        self
    }

    pub fn color(&self) -> Color {
        Color::srgb_u8(self.color[0], self.color[1], self.color[2])
    }
}

impl From<ParticleKind> for Particle {
    fn from(kind: ParticleKind) -> Self {
        match kind {
            ParticleKind::Sand(sand) => Self::from(sand),
            ParticleKind::Water(water) => Self::from(water),
            ParticleKind::Salt(salt) => Self::from(salt),
            ParticleKind::Rock(rock) => Self::from(rock),
            ParticleKind::Drain(drain) => Self::from(drain),
            ParticleKind::Tap(tap) => Self::from(tap),
            ParticleKind::Acid(acid) => Self::from(acid),
        }
    }
}

impl From<Sand> for Particle {
    fn from(sand: Sand) -> Self {
        Self::new(
            Color::hsva(43.20, 0.34, 0.76, 1.00),
            ParticleKind::Sand(sand),
        )
        .with_weight(5)
    }
}

impl From<Salt> for Particle {
    fn from(salt: Salt) -> Self {
        Self::new(
            Color::hsva(0.00, 0.00, 1.00, 1.00),
            ParticleKind::Salt(salt),
        )
        .with_weight(5)
    }
}

impl From<Water> for Particle {
    fn from(water: Water) -> Self {
        let weight = 1 + 3 - water.solvant_capacity;
        let viscosity = u8::MIN + 3 - water.solvant_capacity;
        let color = Color::hsva(201.60, 1.0, 0.80, 1.00)
            .with_saturation(1.0 - (3 - water.solvant_capacity) as f32 * 0.1);
        Self::new(color, ParticleKind::Water(water))
            .with_weight(weight)
            .with_viscosity(viscosity)
    }
}

impl From<Rock> for Particle {
    fn from(rock: Rock) -> Self {
        Self::new(
            Color::hsva(28.0, 0.25, 0.30, 1.00),
            ParticleKind::Rock(rock),
        )
    }
}

impl From<Drain> for Particle {
    fn from(drain: Drain) -> Self {
        Self::new(
            Color::hsva(0.0, 0.0, 0.10, 1.00),
            ParticleKind::Drain(drain),
        )
        .with_cloneable(false)
    }
}

impl From<Tap> for Particle {
    fn from(tap: Tap) -> Self {
        Self::new(
            Color::hsva(190.00, 0.40, 0.75, 1.00),
            ParticleKind::Tap(tap),
        )
        .with_cloneable(false)
    }
}

impl From<Acid> for Particle {
    fn from(acid: Acid) -> Self {
        Self::new(
            Color::hsva(126.00, 1.0, 0.9, 1.00),
            ParticleKind::Acid(acid),
        )
        .with_weight(1)
        .with_viscosity(u8::MIN)
    }
}

impl Particle {
    pub fn update<T: GridAccess>(grid: &mut T, position: (usize, usize)) {
        Self::kill(grid, position);

        if Self::gravity(grid, position) {
            return;
        }

        if Self::flow(grid, position) {
            return;
        }

        let c = grid.get_cell(grid.to_index(position));
        if let Some(this) = &c.particle {
            match this.kind.clone() {
                ParticleKind::Sand(..) => (),
                ParticleKind::Water(water) => water.update(grid, position),
                ParticleKind::Salt(..) => (),
                ParticleKind::Rock(..) => (),
                ParticleKind::Drain(drain) => drain.update(grid, position),
                ParticleKind::Tap(tap) => tap.update(grid, position),
                ParticleKind::Acid(acid) => acid.update(grid, position),
            };
        }
    }

    fn flow<T: GridAccess>(grid: &mut T, position: (usize, usize)) -> bool {
        let c = grid.get_cell(grid.to_index(position));
        let Some(ref this) = c.particle else {
            return false;
        };

        if this.viscosity == u8::MAX {
            return false;
        }

        let index_left = match grid.get_neighbor_index(position, (-1, 0)) {
            Ok(i) => {
                let c = grid.get_cell(i);
                match &c.particle {
                    Some(p) => {
                        if p.viscosity < this.viscosity {
                            Some(i)
                        } else {
                            None
                        }
                    }
                    None => match grid.get_neighbor_index(position, (-2, 0)) {
                        Ok(ii) => {
                            let c = grid.get_cell(ii);
                            match &c.particle {
                                Some(_) => Some(i),
                                None => Some(ii),
                            }
                        }
                        Err(_) => Some(i),
                    },
                }
            }
            Err(_) => None,
        };

        let index_right = match grid.get_neighbor_index(position, (1, 0)) {
            Ok(i) => {
                let c = grid.get_cell(i);
                match &c.particle {
                    Some(p) => {
                        if p.viscosity < this.viscosity {
                            Some(i)
                        } else {
                            None
                        }
                    }
                    None => match grid.get_neighbor_index(position, (2, 0)) {
                        Ok(ii) => {
                            let c = grid.get_cell(ii);
                            match &c.particle {
                                Some(_) => Some(i),
                                None => Some(ii),
                            }
                        }
                        Err(_) => Some(i),
                    },
                }
            }
            Err(_) => None,
        };

        let index = match (index_left, index_right) {
            (None, None) => None,
            (None, Some(i)) => Some(i),
            (Some(i), None) => Some(i),
            (Some(l), Some(r)) => match grid.particle_direction() {
                ParticleHorizontalDirection::Left => Some(l),
                ParticleHorizontalDirection::Right => Some(r),
            },
        };

        if let Some(index) = index {
            grid.swap_particles(grid.to_index(position), index);
            true
        } else {
            false
        }
    }

    fn gravity<T: GridAccess>(grid: &mut T, position: (usize, usize)) -> bool {
        let c = grid.get_cell(grid.to_index(position));
        let (weight, velocity) = if let Some(p) = &c.particle {
            (p.weight, p.velocity)
        } else {
            return false;
        };

        if weight == u8::MIN {
            return false;
        }

        let velocity_probability = grid.velocity_probability();

        if let Ok(index_n) = grid.get_neighbor_index(position, (0, 1)) {
            let cell = grid.get_cell(index_n);
            match &cell.particle {
                Some(p) => {
                    if !grid.is_simulated(cell) && p.weight < weight && p.weight != u8::MIN {
                        let neighbor_viscosity = p.viscosity;
                        if let Some(ref mut this) =
                            grid.get_cell_mut(grid.to_index(position)).particle
                        {
                            let velocity = if neighbor_viscosity < this.viscosity {
                                ((velocity as u16) * 9 / 10) as u8
                            } else {
                                velocity
                            };
                            this.velocity = velocity.saturating_add(1);
                        };
                        if velocity_probability <= velocity {
                            grid.swap_particles(grid.to_index(position), index_n);
                            return true;
                        }
                    }
                }
                None => {
                    if let Some(ref mut this) = grid.get_cell_mut(grid.to_index(position)).particle
                    {
                        this.velocity = velocity.saturating_add(1);
                    };
                    if velocity_probability <= velocity {
                        grid.swap_particles(grid.to_index(position), index_n);
                    }
                    return true;
                }
            };
        }

        let bottom_left = match grid.get_neighbor_index(position, (-1, 1)) {
            Ok(index_n) => match &grid.get_cell(index_n).particle {
                Some(p) => {
                    if p.weight < weight && p.weight != u8::MIN {
                        Some(index_n)
                    } else {
                        None
                    }
                }
                None => Some(index_n),
            },
            Err(_) => None,
        };

        let bottom_right = match grid.get_neighbor_index(position, (1, 1)) {
            Ok(index_n) => match &grid.get_cell(index_n).particle {
                Some(p) => {
                    if p.weight < weight && p.weight != u8::MIN {
                        Some(index_n)
                    } else {
                        None
                    }
                }
                None => Some(index_n),
            },
            Err(_) => None,
        };

        if let Some(index_n) = match (bottom_left, bottom_right) {
            (None, None) => None,
            (None, Some(r)) => Some(r),
            (Some(l), None) => Some(l),
            (Some(l), Some(r)) => match grid.particle_direction() {
                ParticleHorizontalDirection::Left => Some(l),
                ParticleHorizontalDirection::Right => Some(r),
            },
        } {
            if let Some(ref mut this) = grid.get_cell_mut(grid.to_index(position)).particle {
                this.velocity = velocity.saturating_add(1);
            };
            if velocity_probability <= velocity {
                grid.swap_particles(grid.to_index(position), index_n);
            }
            return true;
        }
        let initial_velocity = grid.get_particle_initial_velocity();
        if let Some(ref mut this) = grid.get_cell_mut(grid.to_index(position)).particle {
            this.velocity = velocity.saturating_sub(1).max(initial_velocity);
        };
        false
    }

    fn kill<T: GridAccess>(grid: &mut T, position: (usize, usize)) -> bool {
        let cycle = grid.cycle();
        let c = grid.get_cell_mut(grid.to_index(position));
        let Some(particle) = &mut c.particle else {
            return false;
        };
        if particle.health == 0 {
            c.particle = None;
            c.cycle = cycle;
            return true;
        }
        false
    }
}

impl fmt::Display for Particle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self.kind {
            ParticleKind::Sand(..) => "sand",
            ParticleKind::Water(..) => "water",
            ParticleKind::Salt(..) => "salt",
            ParticleKind::Rock(..) => "rock",
            ParticleKind::Drain(..) => "drain",
            ParticleKind::Tap(..) => "tap",
            ParticleKind::Acid(..) => "acid",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_particle_string_names() {
        assert_eq!("sand", Particle::from(Sand::new()).to_string());
        assert_eq!("salt", Particle::from(Salt::new()).to_string());
        assert_eq!("water", Particle::from(Water::new()).to_string());
        assert_eq!("rock", Particle::from(Rock::new()).to_string());
        assert_eq!("drain", Particle::from(Drain::new()).to_string());
        assert_eq!("tap", Particle::from(Tap::new()).to_string());
        assert_eq!("acid", Particle::from(Acid::new()).to_string());
    }

    #[test]
    fn test_particle_kind_to_default() {
        assert_eq!(
            ParticleKind::from(Water::new()),
            ParticleKind::from(Water::with_capacity(0)).to_default()
        );

        assert_eq!(
            ParticleKind::from(Tap::new()),
            ParticleKind::from(Tap::with_particle(&Particle::from(Rock))).to_default()
        );

        assert_eq!(
            ParticleKind::from(Acid::new()),
            ParticleKind::from(Acid::with_acidity(200)).to_default()
        );
    }
}

#[cfg(test)]
mod weightless {
    use super::*;
    use crate::component::grid::{Cell, Grid};
    use pretty_assertions::assert_eq;

    fn weightless_particle() -> Vec<Particle> {
        vec![
            Particle::from(Rock::new()),
            Particle::from(Tap::new()),
            Particle::from(Drain::new()),
        ]
    }

    #[test]
    fn test_weightless_particles_stay_in_place() {
        /*
         * t- -> t-
         * --    --
         */
        for particle in weightless_particle() {
            let mut g = Grid::new(2, 2);

            g.spawn_particle((0, 0), particle.clone());

            assert_eq!(
                vec![
                    Cell::new(particle.clone()),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                ],
                *g.get_cells()
            );

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(particle),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                ],
                *g.get_cells()
            );
        }
    }
}

#[cfg(test)]
mod powder {
    use super::*;
    use crate::component::grid::{Cell, Grid, Random};
    use pretty_assertions::assert_eq;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn weighted_particle() -> Vec<Particle> {
        vec![Particle::from(Sand::new()), Particle::from(Salt::new())]
    }

    #[test]
    fn test_weighted_particle_falls_down_when_bottom_cell_is_empty() {
        /*
         * S- -> --
         * --    S-
         */
        for particle in weighted_particle() {
            let mut g = Grid::new(2, 2);

            g.spawn_particle((0, 0), particle.clone());

            assert_eq!(
                vec![
                    Cell::new(particle.clone()),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                ],
                *g.get_cells()
            );

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty().with_cycle(1),
                    Cell::empty(),
                    Cell::new(particle).with_cycle(1),
                    Cell::empty(),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_falls_to_bottom_right_when_bottom_cell_and_bottom_left_are_full_and_bottom_right_is_empty()
     {
        /*
         * S- -> --
         * S-    SS
         */
        for particle in weighted_particle() {
            let mut g = Grid::new(2, 2);

            g.spawn_particle((0, 0), particle.clone());
            g.spawn_particle((0, 1), particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty().with_cycle(1),
                    Cell::empty(),
                    Cell::new(particle.clone()),
                    Cell::new(particle).with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_falls_to_bottom_left_when_bottom_cell_and_bottom_right_are_full_and_bottom_left_is_empty()
     {
        /*
         * -S -> --
         * -S    SS
         */
        for particle in weighted_particle() {
            let mut g = Grid::new(2, 2);

            g.spawn_particle((1, 0), particle.clone());
            g.spawn_particle((1, 1), particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty().with_cycle(1),
                    Cell::new(particle.clone()).with_cycle(1),
                    Cell::new(particle),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_falls_to_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_left()
     {
        /*
         * -S- -> ---
         * -S-    SS-
         */
        for particle in weighted_particle() {
            let mut g =
                Grid::new(3, 2).with_rand_particle_direction(|_| ParticleHorizontalDirection::Left);

            g.spawn_particle((1, 0), particle.clone());
            g.spawn_particle((1, 1), particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty().with_cycle(1),
                    Cell::empty(),
                    Cell::new(particle.clone()).with_cycle(1),
                    Cell::new(particle),
                    Cell::empty(),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_falls_to_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_right()
     {
        /*
         * -S- -> ---
         * -S-    -SS
         */
        for particle in weighted_particle() {
            let mut g = Grid::new(3, 2)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Right);

            g.spawn_particle((1, 0), particle.clone());
            g.spawn_particle((1, 1), particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty().with_cycle(1),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::new(particle.clone()),
                    Cell::new(particle.clone()).with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_fall_when_probability_is_max() {
        /*
         * S -> -
         * -    S
         */
        for particle in weighted_particle() {
            let mut g = Grid::new(1, 2).with_rand_velocity(|_| 255);

            g.spawn_particle((0, 0), particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty().with_cycle(1),
                    Cell::new(particle.clone()).with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_does_not_fall_when_velocity_is_zero() {
        /*
         * S -> S
         * -    -
         */
        for particle in weighted_particle() {
            let mut g = Grid::new(1, 2).with_rand_velocity(|_| 255);

            g.spawn_particle((0, 0), particle.clone().with_velocity(0));

            g.update_grid();

            assert_eq!(
                vec![Cell::new(particle.clone().with_velocity(1)), Cell::empty(),],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_does_not_fall_to_right_when_velocity_is_zero() {
        /*
         * S- -> S-
         * r-    r-
         */
        for particle in weighted_particle() {
            let mut g = Grid::new(2, 2).with_rand_velocity(|_| 255);

            g.spawn_particle((0, 0), particle.clone().with_velocity(0));
            g.spawn_particle((0, 1), Particle::from(Rock::new()));

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(particle.clone().with_velocity(1)),
                    Cell::empty(),
                    Cell::new(Particle::from(Rock::new())),
                    Cell::empty(),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_does_not_fall_to_left_when_velocity_is_zero() {
        /*
         * -S -> -S
         * -r    -r
         */
        for particle in weighted_particle() {
            let mut g = Grid::new(2, 2).with_rand_velocity(|_| 255);

            g.spawn_particle((1, 0), particle.clone().with_velocity(0));
            g.spawn_particle((1, 1), Particle::from(Rock::new()));

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::new(particle.clone().with_velocity(1)),
                    Cell::empty(),
                    Cell::new(Particle::from(Rock::new())),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_should_use_the_same_velocity_probability_for_all_direction_checks_so_that_it_spreads_less()
     {
        /*
         * -S- -> -S-
         * ---    ---
         */
        for particle in weighted_particle() {
            static V: &[u8] = &[255, 0]; /*255 won't swap but 0 will*/
            static V_INDEX: AtomicUsize = AtomicUsize::new(0);
            V_INDEX.store(0, Ordering::SeqCst);
            fn velocity_probability(_: &mut Random) -> u8 {
                let idx = V_INDEX.fetch_add(1, Ordering::SeqCst);
                V[idx]
            }

            let mut g = Grid::new(3, 2).with_rand_velocity(velocity_probability);

            g.spawn_particle((1, 0), particle.clone().with_velocity(0));

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::new(particle.clone().with_velocity(1)),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_should_lose_velocity_till_the_initial_velocity_if_they_do_not_move() {
        /*
         * S -> S
         */
        for particle in weighted_particle() {
            let mut g = Grid::new(1, 1)
                .with_rand_velocity(|_| 0)
                .with_initial_particle_velocity(50);

            g.spawn_particle((0, 0), particle.clone().with_velocity(60));

            g.update_grid();

            assert_eq!(
                vec![Cell::new(particle.clone().with_velocity(59))],
                *g.get_cells()
            );

            for _ in 0..20 {
                g.update_grid();
            }

            assert_eq!(
                vec![Cell::new(particle.clone().with_velocity(50))],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_dies_when_their_health_is_zero() {
        for particle in weighted_particle() {
            let mut g = Grid::new(1, 1);

            let particle = particle.clone().with_health(0);
            g.spawn_particle((0, 0), particle.clone());

            assert_eq!(vec![Cell::new(particle)], *g.get_cells());

            g.update_grid();

            assert_eq!(vec![Cell::empty().with_cycle(1)], *g.get_cells());
        }
    }
}

#[cfg(test)]
mod liquid {
    use crate::component::{
        grid::{Cell, Grid, ParticleHorizontalDirection, Random, RowUpdateDirection},
        particles::{acid::Acid, particle::Particle, rock::Rock, salt::Salt, sand::Sand},
    };
    use pretty_assertions::assert_eq;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    fn liquid_particle() -> Vec<Particle> {
        vec![
            Particle::from(Water::with_capacity(0)),
            Particle::from(Acid::with_acidity(0)),
        ]
    }

    fn weighted_particle() -> Vec<Particle> {
        vec![Particle::from(Sand::new()), Particle::from(Salt::new())]
    }

    #[test]
    fn test_update_grid_liquid_particle_falls_down_to_last_row_stays_there() {
        /*
         * w -> - -> -
         * -    w    -
         * -    -    w
         */

        for liquid_particle in liquid_particle() {
            let mut g = Grid::new(1, 3);
            g.spawn_particle((0, 0), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty().with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::empty(),
                ],
                *g.get_cells()
            );

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty().with_cycle(1),
                    Cell::empty().with_cycle(2),
                    Cell::new(liquid_particle.clone()).with_cycle(2),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_moves_right_when_bottom_cell_and_left_are_full() {
        /*
         * --- -> ---
         * sw-    s-w
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(3, 2);
                g.spawn_particle((0, 1), particle.clone());
                g.spawn_particle((1, 1), liquid_particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::empty(),
                        Cell::empty(),
                        Cell::new(particle.clone()),
                        Cell::empty().with_cycle(1),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_moves_left_when_bottom_cell_and_right_are_full() {
        /*
         * --- -> ---
         * -ws    w-s
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(3, 2);
                g.spawn_particle((1, 1), liquid_particle.clone());
                g.spawn_particle((2, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::empty(),
                        Cell::empty(),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                        Cell::empty().with_cycle(1),
                        Cell::new(particle.clone()),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_moves_left_or_right_when_both_right_and_left_are_empty_forced_right()
     {
        /*
         * --- -> ---
         * -w-    --w
         */
        for liquid_particle in liquid_particle() {
            let mut g = Grid::new(3, 2)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Right);

            g.spawn_particle((1, 1), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty().with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_moves_left_or_right_when_both_right_and_left_are_empty_forced_left()
     {
        /*
         * --- -> ---
         * -w-    w--
         */
        for liquid_particle in liquid_particle() {
            let mut g =
                Grid::new(3, 2).with_rand_particle_direction(|_| ParticleHorizontalDirection::Left);

            g.spawn_particle((1, 1), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::empty().with_cycle(1),
                    Cell::empty(),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_liquid_particle_can_slide_two_cells_to_right() {
        /*
         * --- -> ---
         * w--    --w
         */
        for liquid_particle in liquid_particle() {
            let mut g = Grid::new(3, 2)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Right);

            g.spawn_particle((0, 1), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty().with_cycle(1),
                    Cell::empty(),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_liquid_particle_can_slide_two_cells_to_left() {
        /*
         * --- -> ---
         * --w    w--
         */
        for liquid_particle in liquid_particle() {
            let mut g =
                Grid::new(3, 2).with_rand_particle_direction(|_| ParticleHorizontalDirection::Left);

            g.spawn_particle((2, 1), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::empty(),
                    Cell::empty().with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_falls_to_bottom_right_when_bottom_cell_and_bottom_left_are_full_and_bottom_right_is_empty()
     {
        /*
         * w- -> --
         * s-    sw
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(2, 2);

                g.spawn_particle((0, 0), liquid_particle.clone());
                g.spawn_particle((0, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty().with_cycle(1),
                        Cell::empty(),
                        Cell::new(particle.clone()),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_falls_bottom_left_when_bottom_cell_and_bottom_right_are_full_and_bottom_left_is_empty()
     {
        /*
         * -w -> --
         * -s    ws
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(2, 2);

                g.spawn_particle((1, 0), liquid_particle.clone());
                g.spawn_particle((1, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::empty().with_cycle(1),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                        Cell::new(particle.clone()),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_left()
     {
        /*
         * -w- -> ---
         * -s-    ws-
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(3, 2)
                    .with_rand_particle_direction(|_| ParticleHorizontalDirection::Left);

                g.spawn_particle((1, 0), liquid_particle.clone());
                g.spawn_particle((1, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::empty().with_cycle(1),
                        Cell::empty(),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                        Cell::new(particle.clone()),
                        Cell::empty(),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_update_grid_liquid_particle_falls_bottom_left_or_bottom_right_when_bottom_cell_is_full_and_both_bottom_right_and_bottom_left_are_empty_forced_right()
     {
        /*
         * -w- -> ---
         * -s-    -sw
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(3, 2)
                    .with_rand_particle_direction(|_| ParticleHorizontalDirection::Right);

                g.spawn_particle((1, 0), liquid_particle.clone());
                g.spawn_particle((1, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::empty().with_cycle(1),
                        Cell::empty(),
                        Cell::empty(),
                        Cell::new(particle.clone()),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_updating_rows_in_forward_order_creates_a_left_bias_on_liquid_particles() {
        /*
         * -ww- => ww-- or w--w
         */
        for liquid_particle in liquid_particle() {
            let mut g = Grid::new(4, 1)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Left)
                .with_rand_row_update_direction(|_| RowUpdateDirection::Forward);

            g.spawn_particle((1, 0), liquid_particle.clone());
            g.spawn_particle((2, 0), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::empty().with_cycle(1),
                    Cell::empty(),
                ],
                *g.get_cells()
            );

            let mut g = Grid::new(4, 1)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Right)
                .with_rand_row_update_direction(|_| RowUpdateDirection::Forward);

            g.spawn_particle((1, 0), liquid_particle.clone());
            g.spawn_particle((2, 0), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::empty().with_cycle(1),
                    Cell::empty().with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_updating_rows_in_reverse_order_creates_a_right_bias_on_liquid_particles() {
        /*
         * -ww- => --ww or w--w
         */
        for liquid_particle in liquid_particle() {
            let mut g = Grid::new(4, 1)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Right)
                .with_rand_row_update_direction(|_| RowUpdateDirection::Reverse);

            g.spawn_particle((1, 0), liquid_particle.clone());
            g.spawn_particle((2, 0), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::empty().with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                ],
                *g.get_cells()
            );

            let mut g = Grid::new(4, 1)
                .with_rand_particle_direction(|_| ParticleHorizontalDirection::Left)
                .with_rand_row_update_direction(|_| RowUpdateDirection::Reverse);

            g.spawn_particle((1, 0), liquid_particle.clone());
            g.spawn_particle((2, 0), liquid_particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                    Cell::empty().with_cycle(1),
                    Cell::empty().with_cycle(1),
                    Cell::new(liquid_particle.clone()).with_cycle(1),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_should_sink_to_bottom_in_liquid_particle() {
        /*
         * -s- -> -w-
         * sws    sss
         */

        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let particle = particle.with_velocity(0);

                let mut g = Grid::new(3, 2)
                    .with_rand_velocity(|_| 0)
                    .with_initial_particle_velocity(0);

                g.spawn_particle((1, 0), particle.clone());
                g.spawn_particle((0, 1), particle.clone());
                g.spawn_particle((1, 1), liquid_particle.clone().with_velocity(0));
                g.spawn_particle((2, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::new(liquid_particle.clone().with_velocity(0)).with_cycle(1),
                        Cell::empty(),
                        Cell::new(particle.clone()),
                        Cell::new(particle.clone().with_velocity(1)).with_cycle(1),
                        Cell::new(particle.clone()),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_weighted_particle_should_sink_to_bottom_left_in_liquid_particle() {
        /*
         * -s- -> -w-
         * wss    sss
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let particle = particle.with_velocity(0);

                let mut g = Grid::new(3, 2)
                    .with_rand_velocity(|_| 0)
                    .with_initial_particle_velocity(0);

                g.spawn_particle((1, 0), particle.clone());
                g.spawn_particle((0, 1), liquid_particle.clone().with_velocity(0));
                g.spawn_particle((1, 1), particle.clone());
                g.spawn_particle((2, 1), particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::new(liquid_particle.clone().with_velocity(0)).with_cycle(1),
                        Cell::empty(),
                        Cell::new(particle.clone().with_velocity(1)).with_cycle(1),
                        Cell::new(particle.clone()),
                        Cell::new(particle.clone()),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_weighted_particle_should_sink_to_bottom_right_in_liquid_particle() {
        /*
         * -s- -> -w-
         * ssw    sss
         */

        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let particle = particle.with_velocity(0);

                let mut g = Grid::new(3, 2)
                    .with_rand_velocity(|_| 0)
                    .with_initial_particle_velocity(0);

                g.spawn_particle((1, 0), particle.clone());
                g.spawn_particle((0, 1), particle.clone());
                g.spawn_particle((1, 1), particle.clone());
                g.spawn_particle((2, 1), liquid_particle.clone().with_velocity(0));

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::empty(),
                        Cell::new(liquid_particle.clone().with_velocity(0)).with_cycle(1),
                        Cell::empty(),
                        Cell::new(particle.clone()),
                        Cell::new(particle.clone()),
                        Cell::new(particle.clone().with_velocity(1)).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_weighted_particle_should_sink_in_liquid_particle_but_liquid_particle_should_not_climb_on_the_weighted_particle()
     {
        /*
         * s -> s -> w
         * s    w    s
         * w    s    s
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let particle = particle.with_velocity(0);

                let mut g = Grid::new(1, 3)
                    .with_rand_velocity(|_| 0)
                    .with_initial_particle_velocity(0);

                g.spawn_particle((0, 0), particle.clone());
                g.spawn_particle((0, 1), particle.clone());
                g.spawn_particle((0, 2), liquid_particle.clone().with_velocity(0));

                assert_eq!(
                    vec![
                        Cell::new(particle.clone()),
                        Cell::new(particle.clone()),
                        Cell::new(liquid_particle.clone().with_velocity(0)),
                    ],
                    *g.get_cells()
                );

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::new(particle.clone()),
                        Cell::new(liquid_particle.clone().with_velocity(0)).with_cycle(1),
                        Cell::new(particle.clone().with_velocity(1)).with_cycle(1),
                    ],
                    *g.get_cells()
                );

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::new(liquid_particle.clone().with_velocity(0)).with_cycle(2),
                        Cell::new(particle.clone().with_velocity(1)).with_cycle(2),
                        Cell::new(particle.clone().with_velocity(0)).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_weighted_particle_can_sink_to_left_in_liquid_particle_even_if_the_destination_cell_is_simulated()
     {
        /*
         * ws -> -s -> -w
         * -r    wr    sr
         */

        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g =
                    Grid::new(2, 2).with_rand_row_update_direction(|_| RowUpdateDirection::Forward);

                g.spawn_particle((0, 0), liquid_particle.clone());
                g.spawn_particle((1, 0), particle.clone());
                g.spawn_particle((1, 1), Particle::from(Rock::new()));

                g.update_grid();
                assert_eq!(
                    vec![
                        Cell::empty().with_cycle(1),
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                        Cell::new(particle.clone()).with_cycle(1),
                        Cell::new(Particle::from(Rock::new())),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_weighted_particle_can_sink_to_right_in_liquid_particle_even_if_the_destination_cell_is_simulated()
     {
        /*
         * sw -> s- -> w-
         * r-    rw    rs
         */

        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g =
                    Grid::new(2, 2).with_rand_row_update_direction(|_| RowUpdateDirection::Reverse);

                g.spawn_particle((0, 0), particle.clone());
                g.spawn_particle((1, 0), liquid_particle.clone());
                g.spawn_particle((0, 1), Particle::from(Rock::new()));

                g.update_grid();
                assert_eq!(
                    vec![
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                        Cell::empty().with_cycle(1),
                        Cell::new(Particle::from(Rock::new())),
                        Cell::new(particle.clone()).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_if_liquid_particle_did_not_fall_due_to_probability_should_not_flow_either() {
        /*
         * w- -> w-
         * --    --
         */
        for liquid_particle in liquid_particle() {
            static V: &[u8] = &[255]; /*255 won't swap*/
            static V_INDEX: AtomicUsize = AtomicUsize::new(0);
            V_INDEX.store(0, Ordering::SeqCst);
            fn velocity_probability(_: &mut Random) -> u8 {
                let idx = V_INDEX.fetch_add(1, Ordering::SeqCst);
                V[idx]
            }

            let mut g = Grid::new(2, 2).with_rand_velocity(velocity_probability);
            let particle = liquid_particle.clone().with_velocity(0);

            g.spawn_particle((0, 0), particle.clone());

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(particle.clone().with_velocity(1)),
                    Cell::empty(),
                    Cell::empty(),
                    Cell::empty(),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_if_liquid_particle_did_not_fall_left_due_to_probability_should_not_flow_either() {
        /*
         * -w -> -w
         * -r    -r
         */
        for liquid_particle in liquid_particle() {
            static V: &[u8] = &[255]; /*255 won't swap*/
            static V_INDEX: AtomicUsize = AtomicUsize::new(0);
            V_INDEX.store(0, Ordering::SeqCst);
            fn velocity_probability(_: &mut Random) -> u8 {
                let idx = V_INDEX.fetch_add(1, Ordering::SeqCst);
                V[idx]
            }

            let mut g = Grid::new(2, 2).with_rand_velocity(velocity_probability);
            let particle = liquid_particle.clone().with_velocity(0);

            g.spawn_particle((1, 0), particle.clone());
            g.spawn_particle((1, 1), Particle::from(Rock::new()));

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::empty(),
                    Cell::new(particle.clone().with_velocity(1)),
                    Cell::empty(),
                    Cell::new(Particle::from(Rock::new())),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_if_liquid_particle_did_not_fall_right_due_to_probability_should_not_flow_either() {
        /*
         * w- -> w-
         * r-    r-
         */
        for liquid_particle in liquid_particle() {
            static V: &[u8] = &[255]; /*255 won't swap*/
            static V_INDEX: AtomicUsize = AtomicUsize::new(0);
            V_INDEX.store(0, Ordering::SeqCst);
            fn velocity_probability(_: &mut Random) -> u8 {
                let idx = V_INDEX.fetch_add(1, Ordering::SeqCst);
                V[idx]
            }

            let mut g = Grid::new(2, 2).with_rand_velocity(velocity_probability);
            let particle = liquid_particle.clone().with_velocity(0);

            g.spawn_particle((0, 0), particle.clone());
            g.spawn_particle((0, 1), Particle::from(Rock::new()));

            g.update_grid();

            assert_eq!(
                vec![
                    Cell::new(particle.clone().with_velocity(1)),
                    Cell::empty(),
                    Cell::new(Particle::from(Rock::new())),
                    Cell::empty(),
                ],
                *g.get_cells()
            );
        }
    }

    #[test]
    fn test_weighted_particle_does_not_sink_into_liquid_particles_when_velocity_is_zero() {
        /*
         * S -> S
         * w    w
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(1, 2)
                    .with_rand_velocity(|_| 255)
                    .with_initial_particle_velocity(0);

                g.spawn_particle((0, 0), particle.clone().with_velocity(0));
                g.spawn_particle((0, 1), liquid_particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::new(particle.clone().with_velocity(0)),
                        Cell::new(liquid_particle.clone().with_velocity(254))
                    ],
                    *g.get_cells()
                );
            }
        }
    }

    #[test]
    fn test_weighted_particle_loses_10_percent_of_its_velocity_when_sinking_in_liquid_particles() {
        /*
         * S -> w
         * w    S
         */
        for liquid_particle in liquid_particle() {
            for particle in weighted_particle() {
                let mut g = Grid::new(1, 2).with_rand_velocity(|_| 0);

                g.spawn_particle((0, 0), particle.clone().with_velocity(100));
                g.spawn_particle((0, 1), liquid_particle.clone());

                g.update_grid();

                assert_eq!(
                    vec![
                        Cell::new(liquid_particle.clone()).with_cycle(1),
                        Cell::new(particle.clone().with_velocity(91)).with_cycle(1),
                    ],
                    *g.get_cells()
                );
            }
        }
    }
}


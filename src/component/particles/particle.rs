use core::fmt;

use bevy::{
    color::{ColorToPacked, Hsva},
    prelude::{Color, Saturation},
};

use crate::component::grid::{GridAccess, ParticleHorizontalDirection};

use super::{drain::Drain, rock::Rock, salt::Salt, sand::Sand, tap::Tap, water::Water};

#[derive(Clone, PartialEq, Debug)]
pub enum ParticleKind {
    Sand(Sand),
    Water(Water),
    Salt(Salt),
    Rock(Rock),
    Drain(Drain),
    Tap(Tap),
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

#[derive(Clone, PartialEq, Debug)]
pub struct Particle {
    pub weight: u8,
    viscosity: u8,
    pub cloneable: bool,
    color: [u8; 3],
    pub kind: ParticleKind,
    pub seed: u8,
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
        self
    }

    pub fn color(&self) -> Color {
        let color: Hsva = (Color::srgb_u8(self.color[0], self.color[1], self.color[2])).into();
        color
            .with_value(color.value + ((self.seed as f32) - 127.0) / (255.0 * 10.0))
            .into()
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

impl Particle {
    pub fn update<T: GridAccess>(grid: &mut T, position: (usize, usize)) {
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
        let Some(ref this) = c.particle else {
            return false;
        };

        if this.weight == u8::MIN {
            return false;
        }

        if let Ok(index_n) = grid.get_neighbor_index(position, (0, 1)) {
            let cell = grid.get_cell(index_n);
            match &cell.particle {
                Some(p) => {
                    if !grid.is_simulated(cell) && p.weight < this.weight && p.weight != u8::MIN {
                        grid.swap_particles(grid.to_index(position), index_n);
                        return true;
                    }
                }
                None => {
                    grid.swap_particles(grid.to_index(position), index_n);
                    return true;
                }
            };
        }

        let bottom_left = match grid.get_neighbor_index(position, (-1, 1)) {
            Ok(index_n) => match &grid.get_cell(index_n).particle {
                Some(p) => {
                    if p.weight < this.weight && p.weight != u8::MIN {
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
                    if p.weight < this.weight && p.weight != u8::MIN {
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
            grid.swap_particles(grid.to_index(position), index_n);
            true
        } else {
            false
        }
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
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use crate::component::grid::{Cell, Grid};

    use super::*;

    #[test]
    fn test_particle_string_names() {
        assert_eq!("sand", Particle::from(Sand::new()).to_string());
        assert_eq!("salt", Particle::from(Salt::new()).to_string());
        assert_eq!("water", Particle::from(Water::new()).to_string());
        assert_eq!("rock", Particle::from(Rock::new()).to_string());
        assert_eq!("drain", Particle::from(Drain::new()).to_string());
        assert_eq!("tap", Particle::from(Tap::new()).to_string());
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
    }

    #[test]
    fn test_weightless_particles_stay_in_place() {
        /*
         * t- -> t-
         * --    --
         */
        for particle in vec![
            Particle::from(Rock::new()),
            Particle::from(Tap::new()),
            Particle::from(Drain::new()),
        ] {
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

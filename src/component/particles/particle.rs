use core::fmt;

use bevy::{
    color::ColorToPacked,
    prelude::{Color, Saturation},
};

use crate::component::grid::{GridAccess, ParticleHorizontalDirection};

use super::{drain::Drain, rock::Rock, salt::Salt, sand::Sand, tap::Tap, water::Water};

#[derive(Clone, PartialEq, Debug)]
pub enum ParticleProperty {
    Sand(Sand),
    Water(Water),
    Salt(Salt),
    Rock(Rock),
    Drain(Drain),
    Tap(Tap),
}

#[derive(Clone, PartialEq, Debug)]
pub struct Particle {
    weight: u8,
    viscosity: u8,
    pub cloneable: bool,
    color: [u8; 3],
    pub property: ParticleProperty,
}

impl Particle {
    pub fn color(&self) -> Color {
        let color = Color::srgb_u8(self.color[0], self.color[1], self.color[2]);
        match &self.property {
            ParticleProperty::Water(water) => Color::Hsva(color.into())
                .with_saturation(1.0 - (3 - water.solvant_capacity) as f32 * 0.1),
            _ => color,
        }
    }

    fn weight(&self) -> u8 {
        match &self.property {
            ParticleProperty::Water(water) => self.weight + (3 - water.solvant_capacity),
            _ => self.weight,
        }
    }

    fn viscosity(&self) -> u8 {
        match &self.property {
            ParticleProperty::Water(water) => self.viscosity + (3 - water.solvant_capacity),
            _ => self.viscosity,
        }
    }
}

impl From<Sand> for Particle {
    fn from(sand: Sand) -> Self {
        Self {
            weight: 5,
            viscosity: u8::MAX,
            cloneable: true,
            color: Color::hsva(43.20, 0.34, 0.76, 1.00)
                .to_srgba()
                .to_u8_array_no_alpha(),

            property: ParticleProperty::Sand(sand),
        }
    }
}

impl From<Salt> for Particle {
    fn from(salt: Salt) -> Self {
        Self {
            weight: 5,
            viscosity: u8::MAX,
            cloneable: true,
            color: Color::hsva(0.00, 0.00, 1.00, 1.00)
                .to_srgba()
                .to_u8_array_no_alpha(),
            property: ParticleProperty::Salt(salt),
        }
    }
}

impl From<Water> for Particle {
    fn from(water: Water) -> Self {
        Self {
            weight: 1,
            viscosity: u8::MIN,
            cloneable: true,
            color: Color::hsva(201.60, 1.0, 0.80, 1.00)
                .to_srgba()
                .to_u8_array_no_alpha(),
            property: ParticleProperty::Water(water),
        }
    }
}

impl From<Rock> for Particle {
    fn from(rock: Rock) -> Self {
        Self {
            weight: u8::MIN,
            viscosity: u8::MAX,
            cloneable: true,
            color: Color::hsva(28.0, 0.25, 0.30, 1.00)
                .to_srgba()
                .to_u8_array_no_alpha(),
            property: ParticleProperty::Rock(rock),
        }
    }
}

impl From<Drain> for Particle {
    fn from(drain: Drain) -> Self {
        Self {
            weight: u8::MIN,
            viscosity: u8::MAX,
            cloneable: false,
            color: Color::hsva(0.0, 0.0, 0.10, 1.00)
                .to_srgba()
                .to_u8_array_no_alpha(),
            property: ParticleProperty::Drain(drain),
        }
    }
}

impl From<Tap> for Particle {
    fn from(tap: Tap) -> Self {
        Self {
            weight: u8::MIN,
            viscosity: u8::MAX,
            cloneable: false,
            color: Color::hsva(190.0, 0.4, 0.75, 1.00)
                .to_srgba()
                .to_u8_array_no_alpha(),
            property: ParticleProperty::Tap(tap),
        }
    }
}

impl Particle {
    pub fn update<T: GridAccess>(&self, grid: &mut T, position: (usize, usize)) {
        if self.gravity(grid, position) {
            return;
        }
        if self.flow(grid, position) {
            return;
        }
        match &self.property {
            ParticleProperty::Sand(..) => (),
            ParticleProperty::Water(water) => water.update(grid, position),
            ParticleProperty::Salt(..) => (),
            ParticleProperty::Rock(..) => (),
            ParticleProperty::Drain(drain) => drain.update(grid, position),
            ParticleProperty::Tap(tap) => tap.update(grid, position),
        };
    }

    fn flow<T: GridAccess>(&self, grid: &mut T, position: (usize, usize)) -> bool {
        let viscosity = self.viscosity();

        if viscosity == u8::MAX {
            return false;
        }

        let index_left = match grid.get_neighbor_index(position, (-1, 0)) {
            Ok(i) => {
                let c = grid.get_cell(i);
                match &c.particle {
                    Some(p) => {
                        if p.viscosity() < viscosity {
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
                        if p.viscosity() < viscosity {
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

    fn gravity<T: GridAccess>(&self, grid: &mut T, position: (usize, usize)) -> bool {
        let weight = self.weight();

        if weight == u8::MIN {
            return false;
        }

        if let Ok(index_n) = grid.get_neighbor_index(position, (0, 1)) {
            let cell = grid.get_cell(index_n);
            match &cell.particle {
                Some(p) => {
                    if !grid.is_simulated(cell) && p.weight() < weight && p.weight() != u8::MIN {
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
                    if p.weight() < weight && p.weight() != u8::MIN {
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
                    if p.weight() < weight && p.weight() != u8::MIN {
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
        let s = match self.property {
            ParticleProperty::Sand(..) => "sand",
            ParticleProperty::Water(..) => "water",
            ParticleProperty::Salt(..) => "salt",
            ParticleProperty::Rock(..) => "rock",
            ParticleProperty::Drain(..) => "drain",
            ParticleProperty::Tap(..) => "tap",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
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
}

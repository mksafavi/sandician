use core::fmt;

use bevy::prelude::Color;

use crate::component::grid::GridAccess;

use super::{salt::update_salt, sand::update_sand, water::update_water};

#[derive(Clone, PartialEq, Debug)]
pub enum Particle {
    Sand,
    Water { solute: u8 },
    Salt,
}

impl Particle {
    pub fn update<T: GridAccess>(&self, grid: &mut T, (x, y): (usize, usize)) {
        match self {
            Particle::Sand => update_sand(grid, (x, y)),
            Particle::Water { solute } => update_water(grid, *solute, (x, y)),
            Particle::Salt => update_salt(grid, (x, y)),
        };
    }

    pub fn color(&self) -> Color {
        match self {
            Particle::Sand => Color::hsva(43.20, 0.34, 0.76, 1.00),
            Particle::Water { solute } => match solute {
                0 => Color::hsva(201.60, 0.60, 0.80, 1.00),
                _ => Color::hsva(201.60, 1.00, 0.80, 1.00),
            },
            Particle::Salt => Color::hsva(0.00, 0.00, 1.00, 1.00),
        }
    }

    pub fn new_water() -> Particle {
        Particle::Water { solute: 3 }
    }
}

impl fmt::Display for Particle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Particle::Sand => "sand",
            Particle::Water { .. } => "water",
            Particle::Salt => "salt",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_particle_string_names() {
        assert_eq!("sand", Particle::Sand.to_string());
        assert_eq!("salt", Particle::Salt.to_string());
        assert_eq!("water", Particle::new_water().to_string());
    }
}

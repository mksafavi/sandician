//use super::grid::{GridAccess, ParticleHorizontalDirection, ParticleOperation};
use bevy::prelude::Color;

use crate::component::grid::{GridAccess, ParticleOperation};

use super::{salt::update_salt, sand::update_sand, water::update_water};

#[derive(Clone, PartialEq, Debug)]
pub enum Particle {
    Sand,
    Water,
    Salt,
}

impl Particle {
    pub fn update<T: GridAccess>(&self, grid: &T, x: usize, y: usize) -> Option<ParticleOperation> {
        match self {
            Particle::Sand => update_sand(grid, (x, y)),
            Particle::Water => update_water(grid, (x, y)),
            Particle::Salt => update_salt(grid, (x, y)),
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Particle::Sand => Color::srgb(0.76, 0.70, 0.50),
            Particle::Water => Color::srgb(0.05, 0.53, 0.80),
            Particle::Salt => Color::srgb(1.00, 1.00, 1.00),
        }
    }
}

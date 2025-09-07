//use super::grid::{GridAccess, ParticleHorizontalDirection, ParticleOperation};
use bevy::prelude::Color;

use crate::component::grid::{GridAccess, ParticleOperation};

use super::{
    salt::find_salt_particle_next_location, sand::find_sand_particle_next_location,
    water::find_water_particle_next_location,
};

#[derive(Clone, PartialEq, Debug)]
pub enum Particle {
    Sand,
    Water,
    Salt,
}

impl Particle {
    pub fn find_particle_next_location<T: GridAccess>(
        &self,
        grid: &T,
        x: usize,
        y: usize,
    ) -> Option<ParticleOperation> {
        match self {
            Particle::Sand => find_sand_particle_next_location(grid, (x, y)),
            Particle::Water => find_water_particle_next_location(grid, (x, y)),
            Particle::Salt => find_salt_particle_next_location(grid, (x, y)),
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

use crate::component::grid::{GridAccess, ParticleHorizontalDirection, ParticleOperation};

use super::particle::Particle;

pub fn find_sand_particle_next_location<T: GridAccess>(
    grid: &T,
    position: (usize, usize),
) -> Option<ParticleOperation> {
    let index_bottom = match grid.get_neighbor_index(position, (0, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(p) => match p.particle {
                Particle::Water => match p.simulated {
                    true => None,
                    false => Some(i),
                },
                _ => None,
            },
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_bottom_right = match grid.get_neighbor_index(position, (1, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(p) => match p.particle {
                Particle::Water => match p.simulated {
                    true => None,
                    false => Some(i),
                },
                _ => None,
            },
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_bottom_left = match grid.get_neighbor_index(position, (-1, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(p) => match p.particle {
                Particle::Water => match p.simulated {
                    true => None,
                    false => Some(i),
                },
                _ => None,
            },
            None => Some(i),
        },
        Err(_) => None,
    };

    let index = match (index_bottom_left, index_bottom, index_bottom_right) {
        (None, None, None) => None,
        (None, None, Some(r)) => Some(r),
        (Some(l), None, None) => Some(l),
        (Some(l), None, Some(r)) => match grid.water_direction() {
            ParticleHorizontalDirection::Left => Some(l),
            ParticleHorizontalDirection::Right => Some(r),
        },
        (_, Some(i), _) => Some(i),
    };

    index.map(ParticleOperation::Swap)
}

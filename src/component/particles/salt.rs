use crate::component::{grid::{GridAccess, ParticleHorizontalDirection, ParticleOperation}, particles::particle::Particle};

pub fn find_salt_particle_next_location<T: GridAccess>(
    grid: &T,
    position: (usize, usize),
) -> Option<ParticleOperation> {
    let neighboring_water =
        (-1..=1)
            .flat_map(|y| (-1..=1).map(move |x| (y, x)))
            .fold(false, |acc, (xo, yo)| {
                let n = match grid.get_neighbor_index(position, (xo, yo)) {
                    Ok(i) => match grid.get_cell(i) {
                        Some(p) => matches!(p.particle, Particle::Water),
                        None => false,
                    },
                    Err(_) => false,
                };
                acc || n
            });
    if neighboring_water {
        return Some(ParticleOperation::Dissolve(Particle::Water));
    }
    let index_bottom = match grid.get_neighbor_index(position, (0, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(_) => None,
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_bottom_right = match grid.get_neighbor_index(position, (1, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(_) => None,
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_bottom_left = match grid.get_neighbor_index(position, (-1, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(_) => None,
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

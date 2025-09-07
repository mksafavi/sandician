use crate::component::grid::{GridAccess, ParticleHorizontalDirection, ParticleOperation};

pub fn find_water_particle_next_location<T: GridAccess>(
    grid: &T,
    position: (usize, usize),
) -> Option<ParticleOperation> {
    let index_bottom = match grid.get_neighbor_index(position, (0, 1)) {
        Ok(i) => match grid.get_cell(i) {
            Some(_) => None,
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_right = match grid.get_neighbor_index(position, (1, 0)) {
        Ok(i) => match grid.get_cell(i) {
            Some(_) => None,
            None => Some(i),
        },
        Err(_) => None,
    };

    let index_left = match grid.get_neighbor_index(position, (-1, 0)) {
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

    let index = match (
        index_left,
        index_bottom_left,
        index_bottom,
        index_bottom_right,
        index_right,
    ) {
        (None, None, None, None, None) => None,
        (_, _, Some(i), _, _) => Some(i),
        (_, None, None, Some(i), _) => Some(i),
        (_, Some(i), None, None, _) => Some(i),
        (_, Some(l), None, Some(r), _) => match grid.water_direction() {
            ParticleHorizontalDirection::Left => Some(l),
            ParticleHorizontalDirection::Right => Some(r),
        },
        (None, None, None, None, Some(i)) => Some(i),
        (Some(i), None, None, None, None) => Some(i),
        (Some(l), None, None, None, Some(r)) => match grid.water_direction() {
            ParticleHorizontalDirection::Left => Some(l),
            ParticleHorizontalDirection::Right => Some(r),
        },
    };

    index.map(ParticleOperation::Swap)
}

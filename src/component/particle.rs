use bevy::ecs::component::Component;

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
}

impl Grid {
    fn new(width: usize, height: usize) -> Self {
        Self {
            cells: (0..width * height).map(|_| None).collect(),
            width: width,
            height: height,
        }
    }

    fn spawn_particle(&mut self, p: Particle) {
        let index = self.width * p.position.y + p.position.x;
        if self.cells[index].is_none() {
            self.cells[index] = Some(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Particle, ParticleType, Position};

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
}

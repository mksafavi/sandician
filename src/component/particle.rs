use bevy::ecs::component::Component;

#[derive(Component)]
struct Position {
    x: u32,
    y: u32,
}

#[derive(Component, PartialEq, Debug)]
enum ParticleType {
    Sand,
    Water,
}

struct Particle {
    position: Position,
    particle_type: ParticleType,
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

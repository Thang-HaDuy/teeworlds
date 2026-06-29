use crate::character_core::CharacterCore;
use crate::tuning::Tuning;

pub const MAX_CLIENTS: usize = 64;

pub struct WorldCore {
    pub tuning: Tuning,
    pub characters: [Option<CharacterCore>; MAX_CLIENTS],
}

impl WorldCore {
    pub fn new() -> Self {
        Self {
            tuning: Tuning::default(),
            characters: std::array::from_fn(|_| None),
        }
    }
}

impl Default for WorldCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_clients_is_64() {
        assert_eq!(MAX_CLIENTS, 64);
    }

    #[test]
    fn default_tuning_gravity() {
        let world = WorldCore::new();
        assert_eq!(world.tuning.gravity, 0.5);
    }

    #[test]
    fn default_and_new_are_equivalent() {
        let a = WorldCore::new();
        let b = WorldCore::default();
        assert_eq!(a.tuning.gravity, b.tuning.gravity);
    }
}

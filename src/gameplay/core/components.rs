use bevy::prelude::*;

/// Base health of a unit/entity. Gets removed when exposed to high temperatures (fever).
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct Health(pub f32);

impl Default for Health {
    fn default() -> Self {
        Self(100.)
    }
}

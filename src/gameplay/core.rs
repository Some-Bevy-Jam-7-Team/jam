/// Can be split up and renamed later.
use bevy::prelude::*;

/// Temperature of an entity
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct Temperature(pub f32);

/// Base damage of a unit/entity (modified by temperature).
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct Damage(pub f32);

/// Global temperature of the world
#[derive(Resource, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Resource)]
pub struct GlobalTemperature(pub f32);

/// Multiplier for the global temperature can be placed in the world.
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct EnvironmentTemperature(pub f32);

/// Base health of a unit/entity (modified by temperature).
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct Health(pub f32);

use avian3d::prelude::*;
use bevy::prelude::*;

/// Marker component for a temperature sensor, e.g., inserted as a child on the player character controller.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
#[require(Collider::sphere(2.0), Sensor, CollidingEntities::default())]
pub struct TemperatureSensor;

/// Current temperature of an entity
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct Temperature(pub f32);

impl Default for Temperature {
    fn default() -> Self {
        Self(37.0)
    }
}

/// Base temperature of an entity for simulating Homeostasis,
/// i.e., it's the minimum temperature that an entity can reach.
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct BaseTemperature(pub f32);

impl Default for BaseTemperature {
    fn default() -> Self {
        Self(37.0)
    }
}

/// Temperature that a unit can handle before it takes damage.
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct TemperatureThreshold(pub f32);

impl Default for TemperatureThreshold {
    fn default() -> Self {
        Self(40.0)
    }
}

/// Maximum temperature limit of a unit/entity.
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct MaxTemperature(pub f32);

impl Default for MaxTemperature {
    fn default() -> Self {
        Self(45.0)
    }
}

/// Base damage of a unit/entity (could be modified by temperature/fever).
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct BaseDamage(pub f32);

impl Default for BaseDamage {
    fn default() -> Self {
        Self(10.)
    }
}

/// Temperature of objects/entities/space in the environment, affecting the temperature of units around them.
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
#[require(CollisionEventsEnabled, Collider)]
pub struct EnvironmentTemperature(pub f32);

impl Default for EnvironmentTemperature {
    fn default() -> Self {
        Self(45.)
    }
}

#[derive(Component, Debug, Deref, DerefMut, Clone, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct Conductivity(pub f32);

impl Default for Conductivity {
    fn default() -> Self {
        Self(2.0)
    }
}

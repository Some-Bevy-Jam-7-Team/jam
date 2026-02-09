use avian3d::prelude::*;
use bevy::prelude::*;

/// Marker component for a temperature sensor, e.g., inserted as a child on the player character controller.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
#[require(Collider::sphere(2.0), Sensor, CollidingEntities::default())]
pub struct TemperatureSensor;

/// The parent entity with a [`Temperature`] that this [`TemperatureSensorOf`] belongs to.
#[derive(Component, Debug, Deref, Reflect, Clone, Copy)]
#[reflect(Component, Clone, Debug)]
#[relationship(relationship_target = TemperatureSensors)]
pub struct TemperatureSensorOf(#[relationship] pub Entity);

#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect, Default)]
#[reflect(Clone, Debug, Component, Default)]
#[relationship_target(relationship = TemperatureSensorOf)]
pub struct TemperatureSensors(Vec<Entity>);

/// Current temperature of an entity
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
#[require(MaxTemperature, BaseTemperature, TemperatureThreshold)]
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

/// Temperature of objects/entities/space in the environment, affecting the temperature of units around them.
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct EnvironmentTemperature(pub f32);

impl Default for EnvironmentTemperature {
	fn default() -> Self {
		Self(45.)
	}
}

/// Controls how conductive an object is to temperature transfer.
#[derive(Component, Debug, Deref, DerefMut, Clone, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct Conductivity(pub f32);

impl Default for Conductivity {
	fn default() -> Self {
		Self(2.0)
	}
}

/// Controls how sensitive to penetration depth the temperature transfer system is.
///
/// Affects collisions with the heat sensor (multiplied by penetration depth),
/// as well as eaten entities (flat value, defaults to `10.`).
///
/// Might have to tweak this to use higher env temperatures, other temperature scales or a larger sensor.
#[derive(Component, Debug, Deref, DerefMut, Clone, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct DepthSensitivity(pub f32);

impl Default for DepthSensitivity {
	fn default() -> Self {
		Self(10.)
	}
}

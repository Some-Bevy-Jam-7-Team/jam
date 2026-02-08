use bevy::prelude::*;

/// Global temperature of the world
#[derive(Resource, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Resource)]
pub struct GlobalTemperature(pub f32);

impl Default for GlobalTemperature {
	fn default() -> Self {
		Self(20.)
	}
}

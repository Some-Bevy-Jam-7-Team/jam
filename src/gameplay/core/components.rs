use bevy::prelude::*;

/// Base health of a unit/entity. Gets removed when exposed to high temperatures (fever).
///
/// Could be changed to resistance or something, i.e., acts as a buffer before the units die from overheating,
/// so they don't instantly die from standing near very hot objects.
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct Health(pub f32);

impl Default for Health {
	fn default() -> Self {
		Self(100.)
	}
}

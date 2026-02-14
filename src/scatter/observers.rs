use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy_feronia::prelude::*;

pub fn scatter_extended(
	_: On<ScatterFinished<StandardMaterial>>,
	mut cmd: Commands,
	root: Single<Entity, With<ScatterRoot>>,
) {
	cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*root));
}

pub fn scatter_instanced(
	_: On<ScatterFinished<ExtendedWindAffectedMaterial>>,
	mut cmd: Commands,
	root: Single<Entity, With<ScatterRoot>>,
) {
	// Scatter the grass last so it doesn't grow on occupied areas.
	cmd.trigger(Scatter::<InstancedWindAffectedMaterial>::new(*root));
}

use crate::gameplay::level::EnvironmentAssets;
use crate::scatter::observers::*;
use crate::scatter::systems::*;
use bevy::app::prelude::*;
use bevy::prelude::*;
use bevy_eidolon::prelude::*;
use bevy_feronia::asset::backend::scene_backend::SceneAssetBackendPlugin;
use bevy_feronia::prelude::*;

pub fn plugin(app: &mut App) {
	app.add_plugins(ScatterPlugin);
}

pub struct ScatterPlugin;

impl Plugin for ScatterPlugin {
	fn build(&self, app: &mut App) {
		app.insert_resource(GlobalWind {
			current: Wind {
				noise_scale: 0.005,
				..WindPreset::Normal.into()
			},
			..default()
		})
		.add_plugins((
			SceneAssetBackendPlugin,
			StandardScatterPlugin,
			InstancedWindAffectedScatterPlugin,
			ExtendedWindAffectedScatterPlugin,
			GpuComputeCullCorePlugin,
			GpuCullComputePlugin::<InstancedWindAffectedMaterial>::default(),
		));

		app.add_systems(OnEnter(ScatterState::Ready), scatter)
			.add_systems(Startup, spawn_scatter_root)
			.add_systems(
				Update,
				spawn_scatter_layers.run_if(resource_added::<EnvironmentAssets>),
			)
			.add_systems(
				Update,
				(
					scatter.run_if(resource_exists_and_changed::<EnvironmentAssets>),
					update_density_map.run_if(resource_exists::<EnvironmentAssets>),
				),
			)
			.add_observer(scatter_extended)
			.add_observer(scatter_instanced);
	}
}

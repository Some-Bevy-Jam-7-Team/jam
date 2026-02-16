use crate::gameplay::level::EnvironmentAssets;
use crate::scatter::observers::*;
use crate::scatter::quality::QualitySetting;
use crate::scatter::systems::*;
use crate::screens::Screen;
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
		app.init_resource::<QualitySetting>()
			.insert_resource(GlobalWind {
				current: Wind {
					noise_scale: 0.01,
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
				(
					spawn_scatter_layers.run_if(resource_added::<EnvironmentAssets>),
					(
						update_rock_layers,
						update_mushroom_layers,
						update_grass_layers,
					)
						.run_if(resource_changed::<QualitySetting>),
				),
			)
			.add_systems(
				OnEnter(ScatterState::Loading),
				(
					advance_to_setup,
					toggle_chunk_root,
					toggle_mushroom_layer,
					toggle_rock_layer,
					toggle_grass_layer,
				),
			)
			.add_systems(OnExit(Screen::Gameplay), clear_scatter_root)
			.add_systems(
				Update,
				(
					scatter.run_if(
						resource_exists_and_changed::<EnvironmentAssets>
							.and(in_state(Screen::Gameplay))
							.and(in_state(ScatterState::Ready)),
					),
					update_density_map.run_if(resource_exists::<EnvironmentAssets>),
				),
			)
			.add_observer(scatter_extended)
			.add_observer(scatter_instanced)
			.add_observer(scatter_done);
	}
}

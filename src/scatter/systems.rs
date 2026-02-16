use crate::gameplay::level::{CurrentLevel, EnvironmentAssets};
use crate::scatter::{components::*, layers::*};

use crate::scatter::quality::*;
use bevy::asset::{AssetEvent, Assets};
use bevy::image::Image;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy_feronia::prelude::*;
use tracing::debug;

pub fn spawn_scatter_layers(mut cmd: Commands, q_scatter_root: Query<Entity, With<ScatterRoot>>) {
	for root in &q_scatter_root {
		debug!("Spawning scatter layers...");

		cmd.spawn((RockLayer, ChildOf(root)));
		cmd.spawn((MushroomLayer, ChildOf(root)));
		cmd.spawn((GrassLayer, ChildOf(root)));
	}
}

pub fn update_rock_layers(
	mut cmd: Commands,
	settings: ResMut<QualitySetting>,
	q_rock_layer: Query<Entity, With<RockLayer>>,
) {
	let rock_density_settings = RockDensitySetting::from(*settings);
	let rock_visibility_settings = RockVisibilityRangeQuality::from(*settings);

	for layer in &q_rock_layer {
		cmd.entity(layer).insert((
			DistributionDensity::from(rock_density_settings),
			LodConfig::from(rock_visibility_settings),
		));
	}
}

pub fn update_mushroom_layers(
	mut cmd: Commands,
	settings: ResMut<QualitySetting>,
	q_mushroom_layer: Query<Entity, With<MushroomLayer>>,
) {
	let mushroom_density_settings = MushroomDensitySetting::from(*settings);
	let mushroom_visibility_settings = MushroomVisibilityRangeQuality::from(*settings);

	for layer in &q_mushroom_layer {
		cmd.entity(layer).insert((
			DistributionDensity::from(mushroom_density_settings),
			LodConfig::from(mushroom_visibility_settings),
		));
	}
}

pub fn update_grass_layers(
	mut cmd: Commands,
	settings: ResMut<QualitySetting>,
	q_grass_layer: Query<Entity, With<GrassLayer>>,
) {
	let grass_density_settings = GrassDensitySetting::from(*settings);
	let grass_visibility_settings = GrassVisibilityRangeQuality::from(*settings);

	for layer in &q_grass_layer {
		cmd.entity(layer).insert((
			DistributionDensity::from(grass_density_settings),
			LodConfig::from(grass_visibility_settings),
		));
	}
}

pub fn clear_scatter_root(
	mut mw_clear_root: MessageWriter<ClearScatterRoot>,
	q_scatter_root: Query<Entity, With<ScatterRoot>>,
) {
	debug!("Clearing scatter roots...");
	for root in &q_scatter_root {
		mw_clear_root.write(root.into());
	}
}

pub fn toggle_chunk_root(
	mut cmd: Commands,
	q_chunk_root: Query<Entity, With<ChunkRoot>>,
	current_level: Res<CurrentLevel>,
) {
	let enabled = matches!(
		*current_level,
		CurrentLevel::Commune | CurrentLevel::Shaders
	);

	toggle::<ChunkRootDisabled>(&mut cmd, q_chunk_root.iter(), enabled);
}

pub fn toggle_grass_layer(
	mut cmd: Commands,
	q_layer: Query<Entity, With<GrassLayer>>,
	current_level: Res<CurrentLevel>,
) {
	let enabled = matches!(
		*current_level,
		CurrentLevel::Commune | CurrentLevel::Shaders
	);

	toggle::<ScatterLayerDisabled>(&mut cmd, q_layer.iter(), enabled);
}

pub fn toggle_mushroom_layer(
	mut cmd: Commands,
	q_layer: Query<Entity, With<MushroomLayer>>,
	current_level: Res<CurrentLevel>,
) {
	let enabled = matches!(
		*current_level,
		CurrentLevel::Commune | CurrentLevel::Shaders
	);

	toggle::<ScatterLayerDisabled>(&mut cmd, q_layer.iter(), enabled);
}

pub fn toggle_rock_layer(
	mut cmd: Commands,
	q_layer: Query<Entity, With<RockLayer>>,
	current_level: Res<CurrentLevel>,
) {
	let enabled = matches!(
		*current_level,
		CurrentLevel::Commune | CurrentLevel::Shaders | CurrentLevel::Karoline
	);

	toggle::<ScatterLayerDisabled>(&mut cmd, q_layer.iter(), enabled);
}

fn toggle<T: Default + Component>(
	cmd: &mut Commands,
	iter: impl Iterator<Item = Entity>,
	enabled: bool,
) {
	for e in iter {
		if enabled {
			cmd.entity(e).remove::<T>();
		} else {
			cmd.entity(e).insert(T::default());
		}
	}
}

pub fn advance_to_setup(
	mut ns_scatter: ResMut<NextState<ScatterState>>,
	mut ns_height_state: ResMut<NextState<HeightMapState>>,
) {
	ns_scatter.set(ScatterState::Setup);
	ns_height_state.set(HeightMapState::Setup);
}

pub fn scatter(
	mut cmd: Commands,
	q_root: Query<Entity, With<ScatterRoot>>,
	current_level: Res<CurrentLevel>,
) {
	for root in &q_root {
		match *current_level {
			CurrentLevel::Commune | CurrentLevel::Shaders => {
				debug!("Scattering...");
				cmd.trigger(Scatter::<StandardMaterial>::new(root));
			}
			_ => {
				cmd.trigger(ScatterDone);
			}
		}
	}
}

pub fn update_density_map(
	mut ev_asset: MessageReader<AssetEvent<Image>>,
	mut assets: ResMut<Assets<Image>>,
	mut level_assets: ResMut<EnvironmentAssets>,
) {
	for id in ev_asset.read().filter_map(|ev| {
		let AssetEvent::Modified { id, .. } = ev else {
			return None;
		};
		Some(id)
	}) {
		if *id == level_assets.grass_density_map.id() {
			level_assets.grass_density_map = assets.get_strong_handle(*id).unwrap();
		}
		if *id == level_assets.rock_density_map.id() {
			level_assets.rock_density_map = assets.get_strong_handle(*id).unwrap();
		}
		if *id == level_assets.mushroom_density_map.id() {
			level_assets.mushroom_density_map = assets.get_strong_handle(*id).unwrap();
		}
	}
}

pub fn spawn_scatter_root(mut cmd: Commands) {
	cmd.spawn((ScatterRoot::default(), ChunkRoot::default()));
}

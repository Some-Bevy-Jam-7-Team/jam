use crate::gameplay::level::{CurrentLevel, EnvironmentAssets};
use crate::scatter::{components::*, layers::*};

use bevy::asset::{AssetEvent, Assets};
use bevy::image::Image;
use bevy::pbr::StandardMaterial;
use bevy::prelude::*;
use bevy_feronia::prelude::*;
use tracing::debug;

pub fn clear_scatter_root(mut cmd: Commands, root: Single<Entity, With<ScatterRoot>>) {
	debug!("Clearing scatter root...");
	cmd.trigger(ClearScatterRoot(*root));
}

pub fn spawn_scatter_layers(mut cmd: Commands, landscape: Single<Entity, With<ScatterRoot>>) {
	let landscape = landscape.into_inner();
	debug!("Spawning scatter layers...");

	cmd.spawn((RockLayer, ChildOf(landscape)));
	cmd.spawn((MushroomLayer, ChildOf(landscape)));
	cmd.spawn((GrassLayer, ChildOf(landscape)));
}

pub fn scatter(
	mut cmd: Commands,
	mut mw_clear_root: MessageWriter<ClearScatterRoot>,
	root: Single<Entity, With<ScatterRoot>>,
	current_level: Res<CurrentLevel>,
) {
	debug!("Scattering...");
	if *current_level != CurrentLevel::DayTwo {
		cmd.trigger(ScatterDone);
		return;
	};

	mw_clear_root.write((*root).into());

	cmd.trigger(Scatter::<StandardMaterial>::new(*root));
}

pub fn update_density_map(
	mut ev_asset: MessageReader<AssetEvent<Image>>,
	mut assets: ResMut<Assets<Image>>,
	mut level_assets: ResMut<EnvironmentAssets>,
) {
	for ev in ev_asset.read() {
		if let AssetEvent::Modified { id, .. } = ev {
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
}

pub fn spawn_scatter_root(mut cmd: Commands) {
	cmd.spawn((ScatterRoot::default(), ChunkRoot::default(), MapHeight));
}

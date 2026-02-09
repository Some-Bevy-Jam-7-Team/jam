//! Spawn the main level.

use crate::{audio::MusicPool, gameplay::npc::NPC_RADIUS, screens::Screen};
use bevy::prelude::*;
use bevy_landmass::prelude::*;
use bevy_rerecast::prelude::*;
use bevy_seedling::prelude::*;
use bevy_seedling::sample::AudioSample;

use landmass_rerecast::{Island3dBundle, NavMeshHandle3d};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(OnExit(Screen::Gameplay), cleanup_level_assets);
}

#[derive(Resource)]
pub(crate) struct CurrentLevel {
	pub(crate) map_path: &'static str,
	pub(crate) nav_path: &'static str,
}

pub(crate) fn load_level_assets(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	current_level: Res<CurrentLevel>,
) {
	commands.insert_resource(LevelAssets {
		level: asset_server.load(current_level.map_path),
		navmesh: asset_server.load(current_level.nav_path),
		music: asset_server.load("audio/music/Ambiance_Rain_Calm_Loop_Stereo.ogg"),
	});
}

/// A system that spawns the main level.
/// Idempotent: returns early if `LevelAssets` is not yet available or a `Level` entity already exists.
pub(crate) fn spawn_level(
	mut commands: Commands,
	level_assets: Option<Res<LevelAssets>>,
	existing_level: Query<(), With<Level>>,
) {
	let Some(level_assets) = level_assets else {
		return;
	};
	if !existing_level.is_empty() {
		return;
	}

	commands.spawn((
		Name::new("Level"),
		SceneRoot(level_assets.level.clone()),
		DespawnOnExit(Screen::Gameplay),
		Level,
		children![(
			Name::new("Level Music"),
			SamplePlayer::new(level_assets.music.clone()).looping(),
			MusicPool
		)],
	));

	let archipelago = commands
		.spawn((
			Name::new("Main Level Archipelago"),
			DespawnOnExit(Screen::Gameplay),
			Archipelago3d::new(ArchipelagoOptions::from_agent_radius(NPC_RADIUS)),
		))
		.id();

	commands.spawn((
		Name::new("Main Level Island"),
		DespawnOnExit(Screen::Gameplay),
		Island3dBundle {
			island: Island,
			archipelago_ref: ArchipelagoRef3d::new(archipelago),
			nav_mesh: NavMeshHandle3d(level_assets.navmesh.clone()),
		},
	));
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Level;

/// A [`Resource`] that contains all the assets needed to spawn the level.
#[derive(Resource)]
pub(crate) struct LevelAssets {
	pub(crate) level: Handle<Scene>,
	pub(crate) navmesh: Handle<Navmesh>,
	pub(crate) music: Handle<AudioSample>,
}

fn cleanup_level_assets(mut commands: Commands) {
	commands.remove_resource::<LevelAssets>();
}

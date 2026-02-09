//! Spawn the main level.

use crate::{audio::MusicPool, gameplay::npc::NPC_RADIUS, screens::Screen};
use bevy::prelude::*;
use bevy_landmass::prelude::*;
use bevy_rerecast::prelude::*;
use bevy_seedling::prelude::*;
use bevy_seedling::sample::AudioSample;

use landmass_rerecast::{Island3dBundle, NavMeshHandle3d};

pub(super) fn plugin(app: &mut App) {
	app.init_asset::<LevelAssets>();
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
	let handle = asset_server.add(LevelAssets {
		level: asset_server.load(current_level.map_path),
		navmesh: asset_server.load(current_level.nav_path),
		music: asset_server.load("audio/music/Ambiance_Rain_Calm_Loop_Stereo.ogg"),
	});
	commands.insert_resource(LevelAssetsHandle(handle));
}

/// A system that spawns the main level.
/// Idempotent: returns early if assets aren't fully loaded or a `Level` entity already exists.
pub(crate) fn spawn_level(
	mut commands: Commands,
	asset_server: Res<AssetServer>,
	handle: Option<Res<LevelAssetsHandle>>,
	level_assets: Res<Assets<LevelAssets>>,
	existing_level: Query<(), With<Level>>,
) {
	let Some(handle) = handle else { return; };
	if !existing_level.is_empty() {
		return;
	}
	if !asset_server.is_loaded_with_dependencies(&**handle) {
		return;
	}
	let Some(assets) = level_assets.get(&**handle) else {
		return;
	};

	// Insert as a resource so other systems (e.g. debug_ui) can access via Res<LevelAssets>.
	commands.insert_resource(assets.clone());

	commands.spawn((
		Name::new("Level"),
		SceneRoot(assets.level.clone()),
		DespawnOnExit(Screen::Gameplay),
		Level,
		children![(
			Name::new("Level Music"),
			SamplePlayer::new(assets.music.clone()).looping(),
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
			nav_mesh: NavMeshHandle3d(assets.navmesh.clone()),
		},
	));
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Level;

/// A [`Resource`] that contains all the assets needed to spawn the level.
/// Also an [`Asset`] so we can track loading via `is_loaded_with_dependencies`.
#[derive(Asset, Resource, Clone, TypePath)]
pub(crate) struct LevelAssets {
	#[dependency]
	pub(crate) level: Handle<Scene>,
	#[dependency]
	pub(crate) navmesh: Handle<Navmesh>,
	#[dependency]
	pub(crate) music: Handle<AudioSample>,
}

#[derive(Resource, Deref)]
pub(crate) struct LevelAssetsHandle(Handle<LevelAssets>);

fn cleanup_level_assets(mut commands: Commands) {
	commands.remove_resource::<LevelAssets>();
	commands.remove_resource::<LevelAssetsHandle>();
}

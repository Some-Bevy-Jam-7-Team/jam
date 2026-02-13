//! Spawn the main level.

use crate::gameplay::scatter::Landscape;
use crate::third_party::avian3d::CollisionLayer;
use crate::{
	asset_tracking::LoadResource, audio::MusicPool, gameplay::npc::NPC_RADIUS, screens::Screen,
};
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_feronia::prelude::*;
use bevy_landmass::prelude::*;
use bevy_rerecast::prelude::*;
use bevy_seedling::prelude::*;
use bevy_seedling::sample::AudioSample;

use landmass_rerecast::{Island3dBundle, NavMeshHandle3d};

pub(super) fn plugin(app: &mut App) {
	app.load_resource::<LevelAssets>();
}

/// A system that spawns the main level.
pub(crate) fn spawn_level(mut commands: Commands, level_assets: Res<LevelAssets>) {
	commands.spawn((
		Landscape,
		ScatterRoot::default(),
		ChunkRoot::default(),
		MapHeight,
		children![(
			Name::new("Level"),
			SceneRoot(level_assets.level.clone()),
			DespawnOnExit(Screen::Gameplay),
			Level,
			children![
				(
					Name::new("Level Music"),
					SamplePlayer::new(level_assets.music.clone()).looping(),
					MusicPool
				),
				(
					RigidBody::Static,
					SceneRoot(level_assets.landscape.clone()),
					ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh)
						.with_default_layers(CollisionLayers::new(
							CollisionLayer::Default,
							LayerMask::ALL,
						))
						.with_default_density(1_000.0)
				),
			]
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
/// We use this to preload assets before the level is spawned.
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct LevelAssets {
	#[dependency]
	pub(crate) landscape: Handle<Scene>,
	#[dependency]
	pub(crate) level: Handle<Scene>,
	#[dependency]
	pub(crate) navmesh: Handle<Navmesh>,
	#[dependency]
	pub(crate) music: Handle<AudioSample>,
	#[dependency]
	pub(crate) grass: Handle<Scene>,
	#[dependency]
	pub(crate) grass_med: Handle<Scene>,
	#[dependency]
	pub(crate) grass_low: Handle<Scene>,
	#[dependency]
	pub(crate) rocks: Handle<Scene>,
	#[dependency]
	pub(crate) rocks_med: Handle<Scene>,
	#[dependency]
	pub(crate) rocks_low: Handle<Scene>,
	#[dependency]
	pub(crate) grass_density_map: Handle<Image>,
	#[dependency]
	pub(crate) rock_density_map: Handle<Image>,
	#[dependency]
	pub(crate) mushroom: Handle<Scene>,
	#[dependency]
	pub(crate) mushroom_density_map: Handle<Image>,
	pub(crate) break_room_alarm: Handle<AudioSample>,
}

impl FromWorld for LevelAssets {
	fn from_world(world: &mut World) -> Self {
		let assets = world.resource::<AssetServer>();

		Self {
			// Our main level is inspired by the TheDarkMod fan mission [Volta I: The Stone](https://www.thedarkmod.com/missiondetails/?internalName=volta1_3)
			level: assets.load("maps/main/one/one.map#Scene"),
			// You can regenerate the navmesh by using `bevy_rerecast_editor`
			navmesh: assets.load("maps/main/one/one.nav"),
			landscape: assets.load("models/landscape/landscape_flat_large.gltf#Scene0"),
			grass: assets.load("models/grass/grass.glb#Scene0"),
			grass_med: assets.load("models/grass/grass_medium_lod.glb#Scene0"),
			grass_low: assets.load("models/grass/grass_low_lod.glb#Scene0"),
			rocks: assets.load("models/rocks/rocks_low_lod.gltf#Scene0"),
			rocks_med: assets.load("models/rocks/rocks_low_lod.gltf#Scene0"),
			rocks_low: assets.load("models/rocks/rocks_low_lod.gltf#Scene0"),
			#[cfg(feature = "dev")]
			grass_density_map: assets.load("textures/density_map.png"),
			#[cfg(feature = "release")]
			grass_density_map: assets.load("textures/density_map.ktx2"),
			#[cfg(feature = "dev")]
			rock_density_map: assets.load("textures/rock_density_map.png"),
			#[cfg(feature = "release")]
			rock_density_map: assets.load("textures/rock_density_map.ktx2"),
			#[cfg(feature = "dev")]
			mushroom_density_map: assets.load("textures/mushroom_density_map.png"),
			#[cfg(feature = "release")]
			mushroom_density_map: assets.load("textures/mushroom_density_map.ktx2"),
			mushroom: assets.load("models/mushroom/mushroom.gltf#Scene0"),
			music: assets.load("audio/music/corpo slop to eat your computer to.ogg"),
			break_room_alarm: assets.load("audio/sound_effects/mental_health_alarm.ogg"),
		}
	}
}

//! Spawn the main level.

use crate::gameplay::TargetName;
use crate::scatter::components::Landscape;
use crate::{
	asset_tracking::{LoadResource, ResourceHandles},
	audio::MusicPool,
	gameplay::npc::NPC_RADIUS,
	gameplay::objectives::Objective,
	props::logic_entity::ObjectiveEntity,
	screens::{Screen, loading::LoadingScreen},
};
use bevy::input::common_conditions::input_just_pressed;
use bevy::prelude::*;
use bevy_eidolon::prelude::GpuCullCompute;
use bevy_feronia::prelude::*;
use bevy_landmass::prelude::*;
use bevy_rerecast::prelude::*;
use bevy_seedling::prelude::*;
use bevy_seedling::sample::AudioSample;

use crate::shader_compilation::CompileShadersAssets;
use landmass_rerecast::{Island3dBundle, NavMeshHandle3d};

pub(super) fn plugin(app: &mut App) {
	app.load_resource::<EnvironmentAssets>()
		.init_asset::<LevelOneAssets>()
		.init_asset::<LevelTwoAssets>()
		.init_asset::<LevelTrainAssets>()
		.init_asset::<LevelKarolineAssets>();

	#[cfg(feature = "dev")]
	app.add_systems(
		Update,
		(|mut commands: Commands| {
			commands.trigger(AdvanceLevel);
		})
		.run_if(input_just_pressed(KeyCode::F10)),
	);

	app.add_systems(
		Update,
		enter_level.run_if(
			all_assets_loaded
				.and(in_state(LoadingScreen::Assets))
				.and(not(resource_equals(CurrentLevel::CompileShaders))),
		),
	);

	app.add_observer(advance_level);
	app.init_resource::<CurrentLevel>();
}

pub fn all_assets_loaded(resource_handles: Res<ResourceHandles>) -> bool {
	resource_handles.is_all_done()
}

fn enter_level(mut next_screen: ResMut<NextState<LoadingScreen>>) {
	println!("Entering level...");
	next_screen.set(LoadingScreen::Level);
}

#[derive(Resource, Reflect, Debug, Default, Copy, Clone, PartialEq)]
#[reflect(Resource)]
pub(crate) enum CurrentLevel {
	#[default]
	CompileShaders,
	DayOne,
	DayTwo,
	Train,
	Karoline,
}

impl CurrentLevel {
	pub(crate) fn next(&self) -> Self {
		match self {
			CurrentLevel::CompileShaders => CurrentLevel::DayOne,
			CurrentLevel::DayOne => CurrentLevel::DayTwo,
			CurrentLevel::DayTwo => CurrentLevel::Train,
			CurrentLevel::Train => CurrentLevel::Karoline,
			CurrentLevel::Karoline => CurrentLevel::DayOne,
		}
	}
}

pub(crate) fn spawn_landscape(mut cmd: Commands, scatter_root: Single<Entity, With<ScatterRoot>>) {
	debug!("Spawning landscape...");
	cmd.spawn((
		Landscape,
		DespawnOnExit(Screen::Gameplay),
		ChildOf(*scatter_root),
	));
}

/// A system that spawns the main level.
pub(crate) fn spawn_level(
	mut commands: Commands,
	level_assets: Option<Res<LevelOneAssets>>,
	level_two_assets: Option<Res<LevelTwoAssets>>,
	level_train_assets: Option<Res<LevelTrainAssets>>,
	level_karoline_assets: Option<Res<LevelKarolineAssets>>,
	current_level: Res<CurrentLevel>,
	scatter_root: Single<Entity, With<ScatterRoot>>,
	compile_shaders_assets: Res<CompileShadersAssets>,
) {
	match *current_level {
		CurrentLevel::CompileShaders => {
			commands.spawn((
				Center,
				GpuCullCompute,
				Name::new("Compile Shaders Level"),
				SceneRoot(compile_shaders_assets.level.clone()),
				DespawnOnExit(LoadingScreen::Shaders),
			));
		}
		CurrentLevel::DayOne => {
			commands.spawn((
				Objective::new("Clock In"),
				TargetName::new("start_work"),
				ObjectiveEntity {
					target: None,
					objective_order: -1.0,
				},
			));

			let level_one_assets = level_assets.expect("If we don't have level assets when spawning level one, we're in deep shit. Sorry player, we bail here.");
			commands.spawn((
				Name::new("Level"),
				SceneRoot(level_one_assets.level.clone()),
				DespawnOnExit(Screen::Gameplay),
				Level,
				children![(
					Name::new("Level Music"),
					SamplePlayer::new(level_one_assets.music.clone()).looping(),
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
					nav_mesh: NavMeshHandle3d(level_one_assets.navmesh.clone()),
				},
			));
		}
		CurrentLevel::DayTwo => {
			commands.spawn((
				Objective::new("Clock In"),
				TargetName::new("start_work"),
				ObjectiveEntity {
					target: None,
					objective_order: -1.0,
				},
			));
			let level_two_assets = level_two_assets.expect("If we don't have level two assets when spawning level two, we're in deep shit. Sorry player, we bail here.");

			commands.spawn((
				ChildOf(*scatter_root),
				Name::new("Level"),
				SceneRoot(level_two_assets.level.clone()),
				DespawnOnExit(Screen::Gameplay),
				Level,
				children![(
					Name::new("Level Music"),
					SamplePlayer::new(level_two_assets.music.clone()).looping(),
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
					nav_mesh: NavMeshHandle3d(level_two_assets.navmesh.clone()),
				},
			));
		}
		CurrentLevel::Train => {
			let level_train_assets = level_train_assets.expect("If we don't have level two assets when spawning level two, we're in deep shit. Sorry player, we bail here.");

			commands.spawn((
				Name::new("Level"),
				SceneRoot(level_train_assets.level.clone()),
				DespawnOnExit(Screen::Gameplay),
				Level,
				children![(
					Name::new("Level Music"),
					SamplePlayer::new(level_train_assets.music.clone()).looping(),
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
					nav_mesh: NavMeshHandle3d(level_train_assets.navmesh.clone()),
				},
			));
		}
		CurrentLevel::Karoline => {
			let level_karoline_assets = level_karoline_assets.expect("If we don't have level two assets when spawning level two, we're in deep shit. Sorry player, we bail here.");

			commands.spawn((
				Name::new("Level"),
				SceneRoot(level_karoline_assets.level.clone()),
				DespawnOnExit(Screen::Gameplay),
				Level,
				children![(
					Name::new("Level Music"),
					SamplePlayer::new(level_karoline_assets.music.clone()).looping(),
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
					nav_mesh: NavMeshHandle3d(level_karoline_assets.navmesh.clone()),
				},
			));
		}
	}
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct Level;

/// A [`Resource`] that contains all the assets needed to spawn the level.
/// We use this to preload assets before the level is spawned.
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct LevelOneAssets {
	#[dependency]
	pub(crate) level: Handle<Scene>,
	#[dependency]
	pub(crate) navmesh: Handle<Navmesh>,
	#[dependency]
	pub(crate) music: Handle<AudioSample>,
	#[dependency]
	pub(crate) break_room_alarm: Handle<AudioSample>,
}

/// A [`Resource`] that contains all the assets needed to spawn the Environment (landscape and scatter assets).
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct EnvironmentAssets {
	#[dependency]
	pub(crate) landscape: Handle<Scene>,
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
}

impl FromWorld for EnvironmentAssets {
	fn from_world(world: &mut World) -> Self {
		let assets = world.resource::<AssetServer>();
		Self {
			landscape: assets.load("models/landscape/landscape_flat_large.gltf#Scene0"),
			grass: assets.load("models/grass/grass.gltf#Scene0"),
			grass_med: assets.load("models/grass/grass_medium_lod.gltf#Scene0"),
			grass_low: assets.load("models/grass/grass_low_lod.gltf#Scene0"),
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
		}
	}
}

impl FromWorld for LevelOneAssets {
	fn from_world(world: &mut World) -> Self {
		let assets = world.resource::<AssetServer>();

		Self {
			// Our main level is inspired by the TheDarkMod fan mission [Volta I: The Stone](https://www.thedarkmod.com/missiondetails/?internalName=volta1_3)
			level: assets.load("maps/main/one/one.map#Scene"),
			// You can regenerate the navmesh by using `bevy_rerecast_editor`
			navmesh: assets.load("maps/main/one/one.nav"),

			music: assets.load("audio/music/corpo slop to eat your computer to.ogg"),
			break_room_alarm: assets.load("audio/sound_effects/mental_health_alarm.ogg"),
		}
	}
}

/// A [`Resource`] that contains all the assets needed to spawn the level.
/// We use this to preload assets before the level is spawned.
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct LevelTwoAssets {
	#[dependency]
	pub(crate) level: Handle<Scene>,
	#[dependency]
	pub(crate) navmesh: Handle<Navmesh>,
	#[dependency]
	pub(crate) music: Handle<AudioSample>,
}

impl FromWorld for LevelTwoAssets {
	fn from_world(world: &mut World) -> Self {
		let assets = world.resource::<AssetServer>();

		Self {
			// Our main level is inspired by the TheDarkMod fan mission [Volta I: The Stone](https://www.thedarkmod.com/missiondetails/?internalName=volta1_3)
			level: assets.load("maps/main/two/two.map#Scene"),
			// You can regenerate the navmesh by using `bevy_rerecast_editor`
			navmesh: assets.load("maps/main/two/two.nav"),
			music: assets.load("audio/music/corpo slop to eat your computer to.ogg"),
		}
	}
}

/// A [`Resource`] that contains all the assets needed to spawn the level.
/// We use this to preload assets before the level is spawned.
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct LevelTrainAssets {
	#[dependency]
	pub(crate) level: Handle<Scene>,
	#[dependency]
	pub(crate) navmesh: Handle<Navmesh>,
	#[dependency]
	pub(crate) music: Handle<AudioSample>,
}

impl FromWorld for LevelTrainAssets {
	fn from_world(world: &mut World) -> Self {
		let assets = world.resource::<AssetServer>();

		Self {
			// Our main level is inspired by the TheDarkMod fan mission [Volta I: The Stone](https://www.thedarkmod.com/missiondetails/?internalName=volta1_3)
			level: assets.load("maps/main/train/train.map#Scene"),
			// You can regenerate the navmesh by using `bevy_rerecast_editor`
			navmesh: assets.load("maps/main/train/train.nav"),
			music: assets.load("audio/music/corpo slop to eat your computer to.ogg"),
		}
	}
}

/// A [`Resource`] that contains all the assets needed to spawn the level.
/// We use this to preload assets before the level is spawned.
#[derive(Resource, Asset, Clone, TypePath)]
pub(crate) struct LevelKarolineAssets {
	#[dependency]
	pub(crate) level: Handle<Scene>,
	#[dependency]
	pub(crate) navmesh: Handle<Navmesh>,
	#[dependency]
	pub(crate) music: Handle<AudioSample>,
}

impl FromWorld for LevelKarolineAssets {
	fn from_world(world: &mut World) -> Self {
		let assets = world.resource::<AssetServer>();

		Self {
			// Our main level is inspired by the TheDarkMod fan mission [Volta I: The Stone](https://www.thedarkmod.com/missiondetails/?internalName=volta1_3)
			level: assets.load("maps/main/karoline/karoline.map#Scene"),
			// You can regenerate the navmesh by using `bevy_rerecast_editor`
			navmesh: assets.load("maps/main/karoline/karoline.nav"),
			music: assets.load("audio/music/corpo slop to eat your computer to.ogg"),
		}
	}
}

#[derive(Event)]
pub(crate) struct AdvanceLevel;

fn advance_level(
	_done: On<AdvanceLevel>,
	mut commands: Commands,
	current_level: Res<CurrentLevel>,
) {
	match *current_level {
		CurrentLevel::CompileShaders => commands.queue(advance_level_command::<LevelOneAssets>()),
		CurrentLevel::DayOne => commands.queue(advance_level_command::<LevelTwoAssets>()),
		CurrentLevel::DayTwo => commands.queue(advance_level_command::<LevelTrainAssets>()),
		CurrentLevel::Train => commands.queue(advance_level_command::<LevelKarolineAssets>()),
		CurrentLevel::Karoline => commands.queue(advance_level_command::<LevelOneAssets>()),
	};
}

fn advance_level_command<T: Asset + Resource + Clone + FromWorld>() -> impl Command {
	|world: &mut World| {
		let value = T::from_world(world);
		let assets = world.resource::<AssetServer>();
		let handle = assets.add(value);
		let mut handles = world.resource_mut::<ResourceHandles>();
		handles
			.waiting
			.push_back((handle.untyped(), move |world, handle| {
				let assets = world.resource::<Assets<T>>();
				if let Some(value) = assets.get(handle.id().typed::<T>()) {
					world.insert_resource(value.clone());
				}
			}));

		world
			.resource_mut::<NextState<LoadingScreen>>()
			.set(LoadingScreen::Assets);
		world
			.resource_mut::<NextState<Screen>>()
			.set(Screen::Loading);

		let mut current_level = world.resource_mut::<CurrentLevel>();
		*current_level = current_level.next();
	}
}

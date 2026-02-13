//! The loading screen that appears when the game is starting, but still spawning the level.

use bevy::{prelude::*, scene::SceneInstance};
use bevy_eidolon::prelude::*;
use bevy_feronia::asset::backend::scene_backend::SceneAssetBackendPlugin;
use bevy_feronia::prelude::*;
use bevy_landmass::{NavMesh, coords::ThreeD};

use super::LoadingScreen;
use crate::gameplay::level::{LevelAssets, spawn_level};
use crate::gameplay::scatter::{
	GrassLayer, Landscape, MushroomLayer, RockLayer, scattered_shroom, update_density_map,
};
use crate::{
	screens::Screen,
	theme::{palette::SCREEN_BACKGROUND, prelude::*},
};

pub(super) fn plugin(app: &mut App) {
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

	app.add_systems(
		OnEnter(LoadingScreen::Level),
		(spawn_level, spawn_level_loading_screen),
	);
	app.add_systems(OnEnter(HeightMapState::Ready), spawn_grass);
	app.add_systems(OnEnter(ScatterState::Ready), scatter);
	app.add_observer(scatter_extended);
	app.add_observer(scatter_instanced);
	app.add_observer(scattered_shroom);
	app.add_observer(on_scatter_done);
	app.add_systems(
		Update,
		(
			scatter.run_if(resource_exists_and_changed::<LevelAssets>),
			update_density_map.run_if(resource_exists::<LevelAssets>),
		),
	);
	app.add_systems(
		Update,
		advance_to_gameplay_screen.run_if(in_state(LoadingScreen::Level)),
	);
	app.init_resource::<ScatterDoneProbably>();
}

pub fn spawn_grass(mut cmd: Commands, landscape: Single<Entity, With<Landscape>>) {
	let landscape = landscape.into_inner();
	cmd.spawn((RockLayer, ChildOf(landscape)));
	cmd.spawn((MushroomLayer, ChildOf(landscape)));
	cmd.spawn((GrassLayer, ChildOf(landscape)));
}

fn scatter(
	mut cmd: Commands,
	root: Single<Entity, With<ScatterRoot>>,

	mut mw_clear_root: MessageWriter<ClearScatterRoot>,
) {
	mw_clear_root.write((*root).into());

	debug!("Scattering...");
	cmd.trigger(Scatter::<StandardMaterial>::new(*root));
}

fn scatter_extended(
	_: On<ScatterFinished<StandardMaterial>>,
	mut cmd: Commands,
	root: Single<Entity, With<ScatterRoot>>,
) {
	cmd.trigger(Scatter::<ExtendedWindAffectedMaterial>::new(*root));
}

fn scatter_instanced(
	_: On<ScatterFinished<ExtendedWindAffectedMaterial>>,
	mut cmd: Commands,
	root: Single<Entity, With<ScatterRoot>>,
) {
	// Scatter the grass last so it doesn't grow on occupied areas.
	cmd.trigger(Scatter::<InstancedWindAffectedMaterial>::new(*root));
}

fn spawn_level_loading_screen(mut commands: Commands) {
	commands.spawn((
		widget::ui_root("Loading Screen"),
		BackgroundColor(SCREEN_BACKGROUND),
		DespawnOnExit(LoadingScreen::Level),
		children![widget::label("Spawning Level...")],
	));
}

fn advance_to_gameplay_screen(
	mut next_screen: ResMut<NextState<Screen>>,
	scene_spawner: Res<SceneSpawner>,
	scene_instances: Query<&SceneInstance>,
	just_added_scenes: Query<(), (With<SceneRoot>, Without<SceneInstance>)>,
	just_added_meshes: Query<(), Added<Mesh3d>>,
	nav_mesh_events: MessageReader<AssetEvent<NavMesh<ThreeD>>>,
	scatter_done: Res<ScatterDoneProbably>,
	scatter_root: Query<&ScatterRoot>,
) {
	if !(just_added_meshes.is_empty() && just_added_scenes.is_empty()) {
		return;
	}
	if !nav_mesh_events.is_empty() {
		return;
	}
	if !scatter_root.is_empty() && !scatter_done.0 {
		return;
	}

	for scene_instance in scene_instances.iter() {
		if !scene_spawner.instance_is_ready(**scene_instance) {
			return;
		}
	}
	next_screen.set(Screen::Gameplay);
}

#[derive(Resource, Reflect, Debug, Default, Clone, Copy, Deref, DerefMut)]
#[reflect(Resource)]
struct ScatterDoneProbably(bool);

fn on_scatter_done(
	_done: On<ScatterFinished<InstancedWindAffectedMaterial>>,
	mut scatter_done: ResMut<ScatterDoneProbably>,
) {
	scatter_done.0 = true;
}

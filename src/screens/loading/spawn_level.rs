//! The loading screen that appears when the game is starting, but still spawning the level.

use bevy::{prelude::*, scene::SceneInstance};
use bevy_landmass::{NavMesh, coords::ThreeD};

use crate::{
	gameplay::level::{Level, load_level_assets, spawn_level},
	screens::{Screen, loading::LoadingScreenUiNode},
	theme::{palette::SCREEN_BACKGROUND, prelude::*},
};

use super::LoadingScreen;

pub(super) fn plugin(app: &mut App) {
	app.add_systems(
		OnEnter(LoadingScreen::Level),
		(
			load_level_assets,
			// cursed
			spawn_level_loading_screen.run_if(|| false),
		),
	);
	app.add_systems(
		Update,
		(spawn_level, advance_to_gameplay_screen)
			.chain()
			.run_if(in_state(LoadingScreen::Level)),
	);
}

fn spawn_level_loading_screen(mut commands: Commands) {
	commands.spawn((
		LoadingScreenUiNode,
		widget::ui_root("Loading Screen"),
		BackgroundColor(SCREEN_BACKGROUND),
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
	level: Query<(), With<Level>>,
) {
	// Don't advance until spawn_level has actually spawned the level.
	if level.is_empty() {
		return;
	}
	if !(just_added_meshes.is_empty() && just_added_scenes.is_empty()) {
		return;
	}
	if !nav_mesh_events.is_empty() {
		return;
	}

	for scene_instance in scene_instances.iter() {
		if !scene_spawner.instance_is_ready(**scene_instance) {
			return;
		}
	}
	next_screen.set(Screen::Gameplay);
}

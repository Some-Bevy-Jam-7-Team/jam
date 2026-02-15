//! The loading screen that appears when the game is starting, but still spawning the level.

use bevy::{prelude::*, scene::SceneInstance};
use bevy_feronia::prelude::{HeightMapState, ScatterRoot, ScatterState};
use bevy_landmass::{NavMesh, coords::ThreeD};

use super::LoadingScreen;
use crate::font::VARIABLE_FONT;
use crate::gameplay::level::{CurrentLevel, spawn_landscape, spawn_level};
use crate::scatter::ScatterDone;
use crate::theme::palette::HEADER_TEXT;
use crate::{screens::Screen, theme::prelude::*};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(
		OnEnter(LoadingScreen::Level),
		(spawn_level_loading_screen, spawn_landscape, spawn_level),
	)
	.add_systems(
		OnEnter(LoadingScreen::Shaders),
		(spawn_landscape, spawn_level),
	)
	.add_systems(OnExit(Screen::Gameplay), reset_scatter_state)
	.add_systems(OnEnter(Screen::Title), reset_current_level);

	app.add_systems(
		Update,
		advance_to_gameplay_screen.run_if(in_state(LoadingScreen::Level)),
	)
	.add_observer(on_scatter_done);
}

fn reset_current_level(mut current_level: ResMut<CurrentLevel>) {
	*current_level = CurrentLevel::default();
}

fn reset_scatter_state(
	mut ns_scatter: ResMut<NextState<ScatterState>>,
	mut ns_height_map: ResMut<NextState<HeightMapState>>,
) {
	ns_scatter.set(ScatterState::default());
	ns_height_map.set(HeightMapState::default());
}

fn spawn_level_loading_screen(mut commands: Commands) {
	commands.spawn((
		widget::ui_root("Loading Screen"),
		DespawnOnExit(LoadingScreen::Level),
		children![(
			Name::new("Spawning level text"),
			Text("Spawning Level".into()),
			TextFont {
				font: VARIABLE_FONT,
				font_size: 24.0,
				weight: FontWeight(800),
				..default()
			},
			TextColor(HEADER_TEXT),
		)],
	));
}

#[derive(Component)]
struct ScatterReadyToAdvance;

fn advance_to_gameplay_screen(
	mut next_screen: ResMut<NextState<Screen>>,
	scene_spawner: Res<SceneSpawner>,
	scene_instances: Query<&SceneInstance>,
	just_added_scenes: Query<(), (With<SceneRoot>, Without<SceneInstance>)>,
	just_added_meshes: Query<(), Added<Mesh3d>>,
	nav_mesh_events: MessageReader<AssetEvent<NavMesh<ThreeD>>>,
	_: Single<(), With<ScatterReadyToAdvance>>,
) {
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

fn on_scatter_done(
	_: On<ScatterDone>,
	mut cmd: Commands,
	scatter_root: Single<Entity, With<ScatterRoot>>,
) {
	cmd.entity(*scatter_root).insert(ScatterReadyToAdvance);
}

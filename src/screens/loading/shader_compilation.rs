//! A loading screen during which game assets are loaded.
//! This reduces stuttering, especially for audio on Wasm.

use bevy::prelude::*;

use super::LoadingScreen;
use crate::gameplay::level::AdvanceLevel;
use crate::{
	shader_compilation::{LoadedPipelineCount, all_pipelines_loaded},
	theme::{palette::SCREEN_BACKGROUND, prelude::*},
};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(
		OnEnter(LoadingScreen::Shaders),
		(spawn_or_skip_shader_compilation_loading_screen,),
	);

	app.add_systems(
		Update,
		(
			update_loading_shaders_label,
			enter_spawn_level_screen.run_if(all_pipelines_loaded),
		)
			.chain()
			.run_if(in_state(LoadingScreen::Shaders)),
	);
}

fn spawn_or_skip_shader_compilation_loading_screen(
	mut commands: Commands,
	loaded_pipeline_count: Res<LoadedPipelineCount>,
	mut next_screen: ResMut<NextState<LoadingScreen>>,
) {
	if loaded_pipeline_count.is_done() {
		next_screen.set(LoadingScreen::Level);
		return;
	}
	commands.spawn((
		widget::ui_root("Loading Screen"),
		BackgroundColor(SCREEN_BACKGROUND),
		DespawnOnExit(LoadingScreen::Shaders),
		children![(widget::label("Compiling shaders..."), LoadingShadersLabel)],
	));
}

fn enter_spawn_level_screen(mut cmd: Commands) {
	cmd.trigger(AdvanceLevel);
}

#[derive(Component, Reflect)]
#[reflect(Component)]
struct LoadingShadersLabel;

fn update_loading_shaders_label(
	mut query: Query<&mut Text, With<LoadingShadersLabel>>,
	loaded_pipeline_count: Res<LoadedPipelineCount>,
) {
	for mut text in query.iter_mut() {
		text.0 = format!(
			"Compiling shaders: {} / {}",
			loaded_pipeline_count.0,
			LoadedPipelineCount::TOTAL_PIPELINES
		);
	}
}

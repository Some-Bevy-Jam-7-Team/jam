//! [Yarnspinner](https://github.com/YarnSpinnerTool/YarnSpinner-Rust) handles dialogue.

use bevy::prelude::*;

use bevy_yarnspinner::{events::DialogueCompleted, prelude::*};

use crate::{
	gameplay::{
		objectives::{
			complete_dialogue_objective, create_dialogue_objective, create_dialogue_subobjective,
			get_dialogue_current_objective,
		},
		scripting::{
			despawn_entity, interact_with_entity, read_bool_from_entity, set_value_on_entity,
			toggle_bool_on_entity,
		},
	},
	props::specific::intro_crt::set_intro_crt_emote,
	screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
	app.add_plugins((
		// In Wasm, we need to load the dialogue file manually. If we're not targeting Wasm, we can just use `YarnSpinnerPlugin::default()` instead.
		YarnSpinnerPlugin::with_yarn_sources(vec![
			YarnFileSource::file("dialogue/intro_crt.yarn"),
			YarnFileSource::file("dialogue/day_two_crt.yarn"),
			YarnFileSource::file("dialogue/intro_npc.yarn"),
			YarnFileSource::file("dialogue/commune.yarn"),
			YarnFileSource::file("dialogue/day_two_npc.yarn"),
		])
		.with_localizations(Localizations {
			base_localization: "en".into(),
			translations: vec![],
		}),
	));
	app.add_systems(OnEnter(Screen::Gameplay), setup_dialogue_runner);
	app.add_systems(
		OnExit(Screen::Gameplay),
		abort_all_dialogues_when_leaving_gameplay,
	);
}

fn setup_dialogue_runner(mut commands: Commands, yarn_project: Res<YarnProject>) {
	let mut dialogue_runner = yarn_project.create_dialogue_runner(&mut commands);
	dialogue_runner
		.commands_mut()
		.add_command(
			"complete_objective",
			commands.register_system(complete_dialogue_objective),
		)
		.add_command(
			"create_objective",
			commands.register_system(create_dialogue_objective),
		)
		.add_command(
			"create_subobjective",
			commands.register_system(create_dialogue_subobjective),
		)
		.add_command("despawn_entity", commands.register_system(despawn_entity))
		.add_command("set_value", commands.register_system(set_value_on_entity))
		.add_command(
			"toggle_value",
			commands.register_system(toggle_bool_on_entity),
		)
		.add_command(
			"llmanager_emote",
			commands.register_system(set_intro_crt_emote),
		)
		.add_command(
			"interact_with",
			commands.register_system(interact_with_entity),
		);
	dialogue_runner
		.library_mut()
		.add_function(
			"get_current_objective",
			commands.register_system(get_dialogue_current_objective),
		)
		.add_function(
			"is_bool_set",
			commands.register_system(read_bool_from_entity),
		);
	commands.spawn((
		DespawnOnExit(Screen::Gameplay),
		Name::new("Dialogue Runner"),
		dialogue_runner,
	));
}

fn abort_all_dialogues_when_leaving_gameplay(
	q_dialogue_runner: Query<Entity, With<DialogueRunner>>,
	mut commands: Commands,
) {
	for dialogue_runner in q_dialogue_runner.iter() {
		commands
			.entity(dialogue_runner)
			.trigger(|entity| DialogueCompleted { entity });
	}
}

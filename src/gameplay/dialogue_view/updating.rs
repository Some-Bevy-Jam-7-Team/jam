use super::DialogueViewSystemSet;
use super::option_selection::OptionSelection;
use super::setup::{DialogueContinueNode, DialogueNameNode, UiRootNode};
use super::typewriter::Typewriter;
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use bevy_yarnspinner::{events::*, prelude::*};

use crate::audio::SfxPool;

#[derive(Component)]
struct VoiceAudio;

pub(super) fn ui_updating_plugin(app: &mut App) {
	app.init_resource::<AutoContinueTimer>();

	app.add_systems(
		Update,
		(continue_dialogue, auto_continue_dialogue)
			.chain()
			.run_if(resource_exists::<Typewriter>)
			.after(YarnSpinnerSystemSet)
			.in_set(DialogueViewSystemSet),
	)
	.add_message::<SpeakerChangeEvent>()
	.register_type::<SpeakerChangeEvent>();

	app.add_observer(show_dialog);
	app.add_observer(hide_dialog);
	app.add_observer(present_line);
	app.add_observer(present_options);
}

const AUTO_CONTINUE_DELAY_SECS: f32 = 4.0;

#[derive(Resource, Deref, DerefMut)]
struct AutoContinueTimer(Timer);

impl Default for AutoContinueTimer {
	fn default() -> Self {
		Self(Timer::from_seconds(
			AUTO_CONTINUE_DELAY_SECS,
			TimerMode::Once,
		))
	}
}

/// Signals that a speaker has changed.
/// A speaker starts speaking when a new line is presented with a [`PresentLine`] event which has a character name.
/// A speaker stops speaking when the line is fully displayed on the screen, which happens over the course of a few seconds
#[derive(Debug, Eq, PartialEq, Hash, Reflect, Message)]
#[reflect(Debug, PartialEq, Hash)]
#[non_exhaustive]
pub struct SpeakerChangeEvent {
	/// The name of the character who is or was speaking.
	pub character_name: String,
	/// If `true`, the character just started speaking. Otherwise, they just stopped.
	pub speaking: bool,
}

fn show_dialog(_: On<DialogueStarted>, mut visibility: Single<&mut Visibility, With<UiRootNode>>) {
	**visibility = Visibility::Inherited;
}

fn hide_dialog(
	_: On<DialogueCompleted>,
	mut root_visibility: Single<&mut Visibility, With<UiRootNode>>,
	mut commands: Commands,
	voice_query: Query<Entity, With<VoiceAudio>>,
) {
	**root_visibility = Visibility::Hidden;
	for entity in &voice_query {
		commands.entity(entity).despawn();
	}
}

fn present_line(
	event: On<PresentLine>,
	mut speaker_change_events: MessageWriter<SpeakerChangeEvent>,
	mut typewriter: ResMut<Typewriter>,
	name_node: Single<Entity, With<DialogueNameNode>>,
	mut text_writer: TextUiWriter,
	asset_server: Res<AssetServer>,
	mut commands: Commands,
	voice_query: Query<Entity, With<VoiceAudio>>,
) {
	// Stop any previously playing voice line.
	for entity in &voice_query {
		commands.entity(entity).despawn();
	}

	// Play voice audio for this line.
	let id = event
		.line
		.id
		.0
		.strip_prefix("line:")
		.unwrap_or(&event.line.id.0);
	let path = format!("audio/voice/{id}.ogg");
	let handle = asset_server.load::<AudioSample>(path);
	commands.spawn((SamplePlayer::new(handle), SfxPool, VoiceAudio));

	let name = if let Some(name) = event.line.character_name() {
		speaker_change_events.write(SpeakerChangeEvent {
			character_name: name.to_string(),
			speaking: true,
		});
		name.to_string()
	} else {
		String::new()
	};
	*text_writer.text(*name_node, 0) = name;
	typewriter.set_line(&event.line);
}

fn present_options(event: On<PresentOptions>, mut commands: Commands) {
	let option_selection = OptionSelection::from_option_set(&event.options);
	commands.insert_resource(option_selection);
}

fn continue_dialogue(
	keys: Res<ButtonInput<KeyCode>>,
	mouse_buttons: Res<ButtonInput<MouseButton>>,
	touches: Res<Touches>,
	mut dialogue_runners: Query<&mut DialogueRunner>,
	mut typewriter: ResMut<Typewriter>,
	option_selection: Option<Res<OptionSelection>>,
	mut root_visibility: Single<&mut Visibility, With<UiRootNode>>,
	mut continue_visibility: Single<
		&mut Visibility,
		(With<DialogueContinueNode>, Without<UiRootNode>),
	>,
) {
	let explicit_continue = keys.just_pressed(KeyCode::Space)
		|| keys.just_pressed(KeyCode::Enter)
		|| mouse_buttons.just_pressed(MouseButton::Left)
		|| touches.any_just_pressed();
	if explicit_continue && !typewriter.is_finished() {
		typewriter.fast_forward();
		return;
	}
	if (explicit_continue || typewriter.last_before_options) && option_selection.is_none() {
		for mut dialogue_runner in dialogue_runners.iter_mut() {
			if !dialogue_runner.is_waiting_for_option_selection() && dialogue_runner.is_running() {
				dialogue_runner.continue_in_next_update();
				**root_visibility = Visibility::Hidden;
				**continue_visibility = Visibility::Hidden;
			}
		}
	}
}

fn auto_continue_dialogue(
	mut dialogue_runners: Query<&mut DialogueRunner>,
	typewriter: Res<Typewriter>,
	time: Res<Time>,
	mut timer: ResMut<AutoContinueTimer>,
) {
	if typewriter.is_finished() && !typewriter.last_before_options {
		timer.tick(time.delta());
		if timer.just_finished() {
			for mut dialogue_runner in dialogue_runners.iter_mut() {
				if !dialogue_runner.is_waiting_for_option_selection()
					&& dialogue_runner.is_running()
				{
					dialogue_runner.continue_in_next_update();
				}
			}
			timer.reset();
		}
	} else {
		timer.reset();
	}
}

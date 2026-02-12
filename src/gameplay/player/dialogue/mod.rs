//! Player dialogue handling. This module starts the Yarn Spinner dialogue when the player starts interacting with an NPC.

use bevy::prelude::*;

use bevy_yarnspinner::prelude::*;

use crate::{
	PostPhysicsAppSystems, gameplay::interaction::InteractEvent, screens::Screen,
	third_party::bevy_yarnspinner::YarnNode,
};

use super::camera::PlayerCameraParent;

pub(super) fn plugin(app: &mut App) {
	app.init_resource::<DialogueSpeaker>();

	app.configure_sets(
		Update,
		(
			DialogueSystems::UpdateOpportunity,
			DialogueSystems::UpdateUi,
		)
			.chain()
			.in_set(PostPhysicsAppSystems::ChangeUi),
	);

	app.add_systems(
		Update,
		stop_dialogue_far_from_speaker
			.in_set(DialogueSystems::UpdateOpportunity)
			.run_if(in_state(Screen::Gameplay)),
	);
	app.add_observer(interact_with_dialogue);
}

#[derive(Debug, SystemSet, Hash, Eq, PartialEq, Clone, Copy)]
pub(super) enum DialogueSystems {
	UpdateOpportunity,
	UpdateUi,
}

/// A resource that tracks the current speaker of the dialogue, if any.
#[derive(Resource, Default, Reflect, Deref)]
pub(crate) struct DialogueSpeaker(pub(crate) Option<Entity>);

fn interact_with_dialogue(
	trigger: On<InteractEvent>,
	q_yarn_node: Query<&YarnNode>,
	mut dialogue_runner: Single<&mut DialogueRunner>,
	mut speaker: ResMut<DialogueSpeaker>,
) {
	if let Ok(node) = q_yarn_node.get(trigger.0) {
		if dialogue_runner.try_start_node(&node.yarn_node).is_ok() {
			speaker.0 = Some(trigger.0);
		}
	}
}

/// Stops dialogue if the player is too far from the speaker.
fn stop_dialogue_far_from_speaker(
	player: Single<&GlobalTransform, With<PlayerCameraParent>>,
	transforms: Query<&GlobalTransform>,
	mut dialogue_runner: Single<&mut DialogueRunner>,
	speaker: Res<DialogueSpeaker>,
) {
	const MAX_DIALOGUE_DISTANCE: f32 = 4.0;

	let Some(speaker_transform) = speaker
		.0
		.and_then(|speaker_entity| transforms.get(speaker_entity).ok())
	else {
		return;
	};

	let distance = player
		.translation()
		.distance(speaker_transform.translation());

	if distance > MAX_DIALOGUE_DISTANCE {
		dialogue_runner.stop();
	}
}

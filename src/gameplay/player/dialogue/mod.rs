//! Player dialogue handling. This module starts the Yarn Spinner dialogue when the player starts interacting with an NPC.

use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

use bevy_yarnspinner::prelude::*;

use crate::{
	PostPhysicsAppSystems,
	screens::Screen,
	third_party::{
		avian3d::CollisionLayer,
		bevy_yarnspinner::{YarnNode, is_dialogue_running},
	},
};

mod ui;

use super::{Player, camera::PlayerCamera, input::Interact, pickup::is_holding_prop};

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
		(
			check_for_dialogue_opportunity
				.run_if(not(is_dialogue_running).and(not(is_holding_prop))),
			stop_dialogue_far_from_speaker,
		)
			.chain()
			.in_set(DialogueSystems::UpdateOpportunity)
			.run_if(in_state(Screen::Gameplay)),
	);
	app.add_observer(interact_with_dialogue);

	app.add_plugins(ui::plugin);
}

#[derive(Debug, SystemSet, Hash, Eq, PartialEq, Clone, Copy)]
pub(super) enum DialogueSystems {
	UpdateOpportunity,
	UpdateUi,
}

fn check_for_dialogue_opportunity(
	player: Single<&GlobalTransform, With<PlayerCamera>>,
	player_collider: Single<Entity, With<Player>>,
	mut interaction_prompt: Single<&mut InteractionPrompt>,
	q_yarn_node: Query<&YarnNode>,
	spatial_query: SpatialQuery,
) {
	let camera_transform = player.compute_transform();
	const MAX_INTERACTION_DISTANCE: f32 = 3.0;
	let hit = spatial_query.cast_ray(
		camera_transform.translation,
		camera_transform.forward(),
		MAX_INTERACTION_DISTANCE,
		true,
		&SpatialQueryFilter::from_mask(CollisionLayer::Character)
			.with_excluded_entities([*player_collider]),
	);
	let node = hit
		.and_then(|hit| q_yarn_node.get(hit.entity).ok())
		.cloned();
	if interaction_prompt.node != node {
		interaction_prompt.node = node;
	}
	if let Some(hit) = hit {
		interaction_prompt.entity = Some(hit.entity);
	}
}

#[derive(Component, Default, Reflect)]
#[reflect(Component, Default)]
struct InteractionPrompt {
	node: Option<YarnNode>,
	entity: Option<Entity>,
}

/// A resource that tracks the current speaker of the dialogue, if any.
#[derive(Resource, Default, Reflect)]
struct DialogueSpeaker(Option<Entity>);

fn interact_with_dialogue(
	_on: On<Start<Interact>>,
	mut interaction_prompt: Single<&mut InteractionPrompt>,
	mut dialogue_runner: Single<&mut DialogueRunner>,
	mut speaker: ResMut<DialogueSpeaker>,
) {
	let Some(node) = interaction_prompt.node.take() else {
		return;
	};

	speaker.0 = Some(interaction_prompt.entity.take().unwrap());
	dialogue_runner.start_node(&node.yarn_node);
}

/// Stops dialogue if the player is too far from the speaker.
fn stop_dialogue_far_from_speaker(
	player: Single<&GlobalTransform, With<PlayerCamera>>,
	transforms: Query<&GlobalTransform>,
	mut dialogue_runner: Single<&mut DialogueRunner>,
	speaker: Res<DialogueSpeaker>,
) {
	const MAX_DIALOGUE_DISTANCE: f32 = 10.0;

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

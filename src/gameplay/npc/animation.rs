//! NPC animation handling.

use std::time::Duration;

use avian3d::prelude::LinearVelocity;
use bevy::prelude::*;
use bevy_ahoy::CharacterControllerState;

use crate::{
	PostPhysicsAppSystems,
	animation::{AnimationState, AnimationStateTransition},
	gameplay::animation::AnimationPlayers,
	screens::Screen,
};

use super::assets::NpcAssets;

pub(super) fn plugin(app: &mut App) {
	app.add_systems(
		Update,
		play_animations
			.run_if(in_state(Screen::Gameplay))
			.in_set(PostPhysicsAppSystems::PlayAnimations),
	);
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct NpcAnimations {
	idle: AnimationNodeIndex,
	walk: AnimationNodeIndex,
	run: AnimationNodeIndex,
	dance: AnimationNodeIndex,
}

pub(crate) fn setup_npc_animations(
	add: On<Add, AnimationPlayers>,
	q_anim_players: Query<&AnimationPlayers>,
	mut commands: Commands,
	assets: Res<NpcAssets>,
	mut graphs: ResMut<Assets<AnimationGraph>>,
	gltfs: Res<Assets<Gltf>>,
) {
	let gltf = gltfs.get(&assets.model).unwrap();
	let anim_players = q_anim_players.get(add.entity).unwrap();
	for anim_player in anim_players.iter() {
		let (graph, indices) = AnimationGraph::from_clips([
			gltf.named_animations.get("run").unwrap().clone(),
			gltf.named_animations.get("idle").unwrap().clone(),
			gltf.named_animations.get("walk").unwrap().clone(),
			gltf.named_animations.get("dance").unwrap().clone(),
		]);
		let [run_index, idle_index, walk_index, dance_index] = indices.as_slice() else {
			unreachable!()
		};
		let graph_handle = graphs.add(graph);

		let animations = NpcAnimations {
			idle: *idle_index,
			walk: *walk_index,
			run: *run_index,
			dance: *dance_index,
		};
		let transitions = AnimationTransitions::new();
		commands.entity(anim_player).insert((
			animations,
			AnimationGraphHandle(graph_handle),
			transitions,
		));
	}
}

/// Managed by [`play_animations`]
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum NpcAnimationState {
	Standing,
	Airborne,
	Walking(f32),
	Running(f32),
}

fn play_animations(
	mut query: Query<(
		&mut AnimationState<NpcAnimationState>,
		&LinearVelocity,
		&CharacterControllerState,
		&AnimationPlayers,
	)>,
	mut q_animation: Query<(
		&NpcAnimations,
		&mut AnimationPlayer,
		&mut AnimationTransitions,
	)>,
) {
	for (mut animating_state, velocity, state, anim_players) in &mut query {
		let mut iter = q_animation.iter_many_mut(anim_players.iter());
		while let Some((animations, mut anim_player, mut transitions)) = iter.fetch_next() {
			match animating_state.update_by_discriminant({
				let speed = velocity.length();
				if state.grounded.is_none() {
					NpcAnimationState::Airborne
				} else if speed > 7.0 {
					NpcAnimationState::Running(speed)
				} else if speed > 0.01 {
					NpcAnimationState::Walking(speed)
				} else {
					NpcAnimationState::Standing
				}
			}) {
				AnimationStateTransition::Maintain { state: _ } => {}
				AnimationStateTransition::Alter {
					// We don't need the old state here, but it's available for transition
					// animations.
					old_state: _,
					state,
				} => match state {
					NpcAnimationState::Airborne => {
						transitions
							.play(&mut anim_player, animations.run, Duration::from_millis(200))
							.repeat();
					}
					NpcAnimationState::Standing => {
						transitions
							.play(
								&mut anim_player,
								animations.idle,
								Duration::from_millis(500),
							)
							.repeat();
					}
					NpcAnimationState::Walking(_speed) => {
						transitions
							.play(
								&mut anim_player,
								animations.walk,
								Duration::from_millis(300),
							)
							.repeat();
					}
					NpcAnimationState::Running(_speed) => {
						transitions
							.play(&mut anim_player, animations.run, Duration::from_millis(400))
							.repeat();
					}
				},
			}
		}
	}
}

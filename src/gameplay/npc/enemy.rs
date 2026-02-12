use std::f32::consts::TAU;

use avian3d::prelude::{ColliderOf, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use bevy_bae::prelude::*;
use rand::{Rng, rng};

use crate::{
	gameplay::{
		npc::ai::{Agent, NpcWalkTargetOf},
		player::Player,
	},
	third_party::avian3d::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(FixedUpdate, update_sensors.before(BaeSystems::ExecutePlan));
}

fn update_sensors(
	mut commands: Commands,
	spatial: SpatialQuery,
	mut enemies: Query<(Entity, &GlobalTransform, &mut Props, &mut EnemyAiState)>,
	player: Single<(Entity, &Transform), With<Player>>,
	colliders: Query<&ColliderOf>,
	time: Res<Time>,
) {
	let (player_entity, player_transform) = player.into_inner();
	for (entity, transform, mut props, mut state) in enemies.iter_mut() {
		state.walk_timer.tick(time.delta());
		if !props.get::<bool>("alert") {
			let dist_sq = transform
				.translation()
				.distance_squared(player_transform.translation);
			const MAX_DIST: f32 = 30.0;
			if dist_sq < MAX_DIST * MAX_DIST
				&& let Ok(dir) = Dir3::new(player_transform.translation - transform.translation())
				&& spatial
					.cast_ray(
						transform.translation(),
						dir,
						MAX_DIST,
						true,
						&SpatialQueryFilter::from_mask([
							CollisionLayer::Default,
							CollisionLayer::Prop,
							CollisionLayer::PlayerCharacter,
						]),
					)
					.is_some_and(|hit| {
						colliders
							.get(hit.entity)
							.is_ok_and(|rb| rb.body == player_entity)
					}) {
				props.set("alert", true);
			}
		}
		if props.get::<bool>("alert") {
			const MELEE_RANGE: f32 = 1.0;
			if transform
				.translation()
				.distance_squared(player_transform.translation)
				< MELEE_RANGE * MELEE_RANGE
			{
				if !props.get::<bool>("in_melee_range") {
					commands.entity(entity).trigger(UpdatePlan::from);
					props.set("in_melee_range", true);
				}
			} else {
				if props.get::<bool>("in_melee_range") {
					commands.entity(entity).trigger(UpdatePlan::from);
					props.set("in_melee_range", false);
				}
			}
		}
	}
}

pub(crate) fn melee_enemy_htn() -> impl Bundle {
	(
		EnemyAiState::default(),
		Plan::new(),
		Select,
		tasks![
			(
				conditions![Condition::eq("alert", false)],
				Operator::new(walk_randomly),
			),
			(
				conditions![Condition::eq("in_melee_range", true)],
				Operator::new(melee_attack),
			),
			Operator::new(go_to_player),
		],
	)
}

fn walk_randomly(
	In(input): In<OperatorInput>,
	mut npcs: Query<&Agent>,
	transforms: Query<&GlobalTransform>,
	mut states: Query<&EnemyAiState>,
	spatial: SpatialQuery,
	mut commands: Commands,
) -> OperatorStatus {
	let Ok(state) = states.get_mut(input.entity) else {
		return OperatorStatus::Failure;
	};

	let Ok(agent) = npcs.get_mut(input.entity) else {
		return OperatorStatus::Failure;
	};
	let Ok(transform) = transforms.get(agent.entity()) else {
		return OperatorStatus::Failure;
	};

	if state.walk_timer.is_finished() {
		let yaw = rng().random_range(0.0..TAU);
		let dir = Dir3::new_unchecked(Vec3::NEG_Z.rotate_y(yaw));
		const MAX_WALK_DIST: f32 = 10.0;
		let target_dist = spatial
			.cast_ray(
				transform.translation(),
				dir,
				MAX_WALK_DIST,
				true,
				&SpatialQueryFilter::from_mask([
					CollisionLayer::Default,
					CollisionLayer::PlayerCharacter,
					CollisionLayer::Prop,
				]),
			)
			.map_or(MAX_WALK_DIST, |hit| (hit.distance - 0.1).max(0.0));
		let target_pos = transform.translation() + dir * target_dist;
		agent.entity();

		commands
			.entity(input.entity)
			.with_related::<NpcWalkTargetOf>(Transform::from_translation(target_pos));
	}
	OperatorStatus::Ongoing
}

fn melee_attack(
	In(input): In<OperatorInput>,
	mut enemy: Query<&mut EnemyAiState>,
) -> OperatorStatus {
	let Ok(mut enemy) = enemy.get_mut(input.entity) else {
		return OperatorStatus::Failure;
	};
	enemy.punching = true;
	OperatorStatus::Ongoing
}

fn go_to_player(
	In(input): In<OperatorInput>,
	mut commands: Commands,
	player: Single<&Transform, With<Player>>,
) -> OperatorStatus {
	commands
		.entity(input.entity)
		.with_related::<NpcWalkTargetOf>(**player);
	OperatorStatus::Ongoing
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub(crate) struct EnemyAiState {
	pub(crate) walk_timer: Timer,
	pub(crate) punching: bool,
}

impl Default for EnemyAiState {
	fn default() -> Self {
		Self {
			walk_timer: Timer::from_seconds(rng().random_range(4.0..6.0), TimerMode::Repeating),
			punching: false,
		}
	}
}

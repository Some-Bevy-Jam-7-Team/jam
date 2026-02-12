use std::f32::consts::TAU;

use avian3d::prelude::{ColliderOf, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use bevy_bae::prelude::*;
use bevy_landmass::{Archipelago3d, FromAgentRadius as _, PointSampleDistance3d};
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
	spatial: SpatialQuery,
	mut enemies: Query<(&GlobalTransform, &mut Props, &mut EnemyAiState)>,
	player: Single<(Entity, &Transform), With<Player>>,
	colliders: Query<&ColliderOf>,
	time: Res<Time>,
) {
	let (player_entity, player_transform) = player.into_inner();
	for (transform, mut props, mut state) in enemies.iter_mut() {
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
					.is_none_or(|hit| {
						colliders
							.get(hit.entity)
							.is_ok_and(|rb| rb.body == player_entity)
					}) {
				props.set("alert", true);
			}
		}
	}
}

pub(crate) fn enemy_htn() -> impl Bundle {
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
				conditions![Condition::eq("alert", true)],
				Operator::new(attack_player),
			),
		],
	)
}

fn walk_randomly(
	In(input): In<OperatorInput>,
	mut npcs: Query<&Agent>,
	transforms: Query<&GlobalTransform>,
	archipelago: Single<&Archipelago3d>,
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

		let target_pos_for_real = match archipelago
			.sample_point(target_pos, &PointSampleDistance3d::from_agent_radius(10.0))
		{
			Ok(target) => target.point(),
			Err(err) => {
				error!(position_sampling_error = ?err);
				return OperatorStatus::Failure;
			}
		};
		commands
			.entity(input.entity)
			.with_related::<NpcWalkTargetOf>(Transform::from_translation(target_pos_for_real));
	}
	OperatorStatus::Success
}

fn attack_player(In(_input): In<OperatorInput>) -> OperatorStatus {
	// TODO: Implement lol
	OperatorStatus::Success
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct EnemyAiState {
	walk_timer: Timer,
}

impl Default for EnemyAiState {
	fn default() -> Self {
		Self {
			walk_timer: Timer::from_seconds(rng().random_range(4.0..6.0), TimerMode::Repeating),
		}
	}
}

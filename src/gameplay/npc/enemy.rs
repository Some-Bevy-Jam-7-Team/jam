use std::f32::consts::TAU;

use avian3d::prelude::{SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use bevy_bae::prelude::*;
use bevy_landmass::{Archipelago3d, FromAgentRadius as _, PointSampleDistance3d};
use bevy_trenchbroom::prelude::*;
use rand::{Rng, rng};

use crate::{
	gameplay::npc::ai::{Agent, NpcWalkTargetOf},
	third_party::avian3d::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
	app.add_observer(add_walker);
}

#[base_class]
#[derive(Default)]
pub struct Walker {
	is_walker: bool,
}

fn add_walker(add: On<Insert, Walker>, walker: Query<&Walker>, mut commands: Commands) {
	let Ok(walker) = walker.get(add.entity) else {
		return;
	};
	if walker.is_walker {
		commands.entity(add.entity).insert(enemy_htn());
	}
}

pub(crate) fn enemy_htn() -> impl Bundle {
	(
		EnemyAiState::default(),
		Plan::new(),
		Select,
		tasks![(Operator::new(walk_randomly),),],
	)
}

fn walk_randomly(
	In(input): In<OperatorInput>,
	mut npcs: Query<&Agent>,
	transforms: Query<&GlobalTransform>,
	archipelago: Single<&Archipelago3d>,
	mut states: Query<&mut EnemyAiState>,
	spatial: SpatialQuery,
	mut commands: Commands,
	time: Res<Time>,
) -> OperatorStatus {
	let Ok(mut state) = states.get_mut(input.entity) else {
		return OperatorStatus::Failure;
	};

	let Ok(agent) = npcs.get_mut(input.entity) else {
		return OperatorStatus::Failure;
	};
	let Ok(transform) = transforms.get(agent.entity()) else {
		return OperatorStatus::Failure;
	};

	state.walk_timer.tick(time.delta());
	if state.walk_timer.is_finished() {
		let yaw = rng().random_range(0.0..TAU);
		let dir = Dir3::new_unchecked(Vec3::NEG_Z.rotate_y(yaw));
		const MAX_WALK_DIST: f32 = 10.0;
		let walk_dist = rng().random_range(0.5..MAX_WALK_DIST);
		let target_dist = spatial
			.cast_ray(
				transform.translation(),
				dir,
				walk_dist,
				true,
				&SpatialQueryFilter::from_mask([
					CollisionLayer::Default,
					CollisionLayer::PlayerCharacter,
					CollisionLayer::Prop,
				]),
			)
			.map_or(walk_dist, |hit| (hit.distance - 0.1).max(0.0));
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

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct EnemyAiState {
	walk_timer: Timer,
}

impl Default for EnemyAiState {
	fn default() -> Self {
		Self {
			walk_timer: Timer::from_seconds(rng().random_range(6.0..10.0), TimerMode::Repeating),
		}
	}
}

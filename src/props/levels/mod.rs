use std::time::Duration;

use crate::audio::SpatialPool;
use crate::{gameplay::objectives::*, third_party::avian3d::CollisionLayer};
use avian3d::prelude::*;
use bevy::ecs::error::Result;
use bevy::prelude::*;
use bevy_seedling::prelude::*;
use bevy_trenchbroom::prelude::*;

use crate::{
	gameplay::level::LevelAssets,
	props::logic_entity::ObjectiveEntity,
	timer::{GenericTimer, TimerFinished},
};

pub(super) fn plugin(app: &mut App) {
	app.add_observer(setup_break_room);
}

fn setup_break_room(add: On<Add, BreakRoomSensor>, mut commands: Commands) -> Result {
	let entity = add.entity;
	commands
		.entity(entity)
		.insert((
			GenericTimer::<BreakRoomTimer>::new(Timer::new(
				Duration::from_secs(5),
				TimerMode::Once,
			))
			.with_active(false),
			CollisionLayers::new([CollisionLayer::Sensor], [CollisionLayer::PlayerCharacter]),
		))
		.observe(tell_to_eat)
		.observe(kick_out);
	commands
		.spawn((
			Name::new("Objective: work_6".to_string()),
			ObjectiveEntity {
				targetname: "work_6".into(),
				target: None,
				objective_order: 6.0,
			},
			Objective::new("Increase Shareholder Value"),
		))
		.observe(
			move |_add: On<Add, ObjectiveCompleted>,
			      mut timer: Query<&mut GenericTimer<BreakRoomTimer>>|
			      -> Result {
				let mut timer = timer.get_mut(entity)?;

				timer.set_active(true);
				Ok(())
			},
		);
	Ok(())
}

fn kick_out(
	finished: On<TimerFinished<BreakRoomTimer>>,
	mut commands: Commands,
	objectives: Query<(Entity, &ObjectiveEntity)>,
	level_assets: Res<LevelAssets>,
	transform: Query<&GlobalTransform>,
) {
	if let Some((entity, _)) = objectives
		.iter()
		.find(|(_, ObjectiveEntity { targetname, .. })| targetname == "work_7")
	{
		let translation = transform
			.get(finished.entity)
			.map(|t| t.translation())
			.unwrap_or_default();
		commands.spawn((
			SamplePlayer {
				sample: level_assets.break_room_alarm.clone(),
				repeat_mode: RepeatMode::RepeatMultiple {
					num_times_to_repeat: 2,
				},
				..default()
			},
			SpatialPool,
			Transform::from_translation(translation),
		));
		commands.entity(entity).insert(ObjectiveCompleted);
	}
}

struct BreakRoomTimer;

#[solid_class(base(Transform, Visibility))]
#[require(Sensor, CollisionEventsEnabled)]
pub(crate) struct BreakRoomSensor;

fn tell_to_eat(
	_collision: On<CollisionStart>,
	mut commands: Commands,
	objectives: Query<&ObjectiveEntity>,
	current_objective: Res<CurrentObjective>,
) -> Result<(), BevyError> {
	let Some(current_objective) = **current_objective else {
		return Ok(());
	};
	let Ok(ObjectiveEntity { targetname, .. }) = objectives.get(current_objective) else {
		return Ok(());
	};

	if targetname == "work_5" {
		commands
			.entity(current_objective)
			.insert(ObjectiveCompleted);
	}
	Ok(())
}

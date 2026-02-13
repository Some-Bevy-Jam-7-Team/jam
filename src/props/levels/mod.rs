use crate::audio::SpatialPool;
use crate::gameplay::TargetName;
use crate::gameplay::interaction::InteractEvent;
use crate::gameplay::objectives::*;
use crate::props::logic_entity::SensorEntity;
use avian3d::prelude::*;
use bevy::ecs::error::Result;
use bevy::prelude::*;
use bevy_seedling::prelude::*;

use crate::gameplay::level::LevelAssets;

pub(super) fn plugin(app: &mut App) {
	app.add_observer(setup_break_room);
}

fn setup_break_room(add: On<Add, SensorEntity>, mut commands: Commands) -> Result {
	let entity = add.entity;
	commands
		.entity(entity)
		.observe(tell_to_eat)
		.observe(kick_out);
	Ok(())
}

fn kick_out(
	trigger: On<InteractEvent>,
	mut commands: Commands,
	level_assets: Res<LevelAssets>,
	transform: Query<&GlobalTransform>,
) {
	let translation = transform
		.get(trigger.0)
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
}

fn tell_to_eat(
	_collision: On<CollisionStart>,
	mut commands: Commands,
	objectives: Query<&TargetName>,
	current_objective: Res<CurrentObjective>,
) -> Result<(), BevyError> {
	let Some(current_objective) = **current_objective else {
		return Ok(());
	};
	let Ok(targetname) = objectives.get(current_objective) else {
		return Ok(());
	};

	if **targetname == "work_5" {
		commands
			.entity(current_objective)
			.insert(ObjectiveCompleted);
	}
	Ok(())
}

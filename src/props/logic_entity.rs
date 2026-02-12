use bevy::prelude::*;

use bevy_trenchbroom::prelude::*;

use crate::gameplay::{
	TargetName, TargetnameEntityIndex,
	objectives::{Objective, SubObjectiveOf},
};

pub(super) fn plugin(app: &mut App) {
	app.add_observer(uninitialise_objectives)
		.add_systems(Update, initialise_objectives);
}

fn uninitialise_objectives(add: On<Insert, ObjectiveEntity>, mut commands: Commands) {
	commands.entity(add.entity).insert(UnitialisedObjective);
}

fn initialise_objectives(
	uninit_objectives: Populated<(Entity, &ObjectiveEntity), With<UnitialisedObjective>>,
	objectives: Query<(), With<Objective>>,
	entity_index: Res<TargetnameEntityIndex>,
	mut commands: Commands,
) {
	for (entity, obj) in uninit_objectives.iter() {
		if let Some(target) = obj.target.as_ref() {
			if let Some(&parent) = entity_index
				.get_entity_by_targetname(target)
				.iter()
				.filter(|entity| objectives.contains(**entity))
				.next()
			{
				commands
					.entity(entity)
					.remove::<UnitialisedObjective>()
					.insert(SubObjectiveOf { objective: parent });
			}
		} else {
			commands.entity(entity).remove::<UnitialisedObjective>();
		}
	}
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct UnitialisedObjective;

/// An entity describing the identity of an objective
#[point_class(base(TargetName, Objective))]
#[derive(Default)]
pub(crate) struct ObjectiveEntity {
	/// The objective, if any, that this is a subobjective of
	/// DO NOT MUTATE, IT WON'T UPDATE
	pub target: Option<String>,
	/// The ordering of the objective, bigger = later
	pub objective_order: f32,
}

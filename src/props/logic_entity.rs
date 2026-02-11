use bevy::prelude::*;

use bevy_trenchbroom::prelude::*;

use crate::gameplay::objectives::{Objective, SubObjectiveOf};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(Update, initialise_objectives);
}

fn initialise_objectives(
	uninit_objectives: Populated<(Entity, &ObjectiveEntity), With<UnitialisedObjective>>,
	objectives: Query<(Entity, &ObjectiveEntity)>,
	mut commands: Commands,
) {
	for (entity, obj) in uninit_objectives.iter() {
		if let Some(target) = obj.target.as_ref() {
			for (parent, parent_candidate) in objectives.iter() {
				if *target == parent_candidate.targetname {
					commands
						.entity(entity)
						.remove::<UnitialisedObjective>()
						.insert(SubObjectiveOf { objective: parent });
				}
			}
		} else {
			commands.entity(entity).remove::<UnitialisedObjective>();
		}
	}
}

#[base_class]
struct UnitialisedObjective;

/// An entity describing the identity of an objective
#[point_class(base(Objective, UnitialisedObjective))]
#[derive(Default)]
pub(crate) struct ObjectiveEntity {
	/// The name by which other entities (such as an objective completor or subobjectives) refer to this entity
	pub targetname: String,
	/// The objective, if any, that this is a subobjective of
	pub target: Option<String>,
	/// The ordering of the objective, bigger = later
	pub objective_order: f32,
}

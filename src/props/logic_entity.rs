use bevy::prelude::*;

use bevy_trenchbroom::prelude::*;

use crate::{
	gameplay::{
		TargetName, TargetnameEntityIndex,
		objectives::{Objective, SubObjectiveOf},
	},
	props::interactables::InteractableEntity,
	reflection::ReflAppExt,
};

pub(super) fn plugin(app: &mut App) {
	app.register_dynamic_component::<ObjectiveEntity>()
		.register_dynamic_component::<YarnNode>()
		.add_observer(uninitialise_objectives)
		.add_observer(talk_ify_yarnnode)
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
				.find(|entity| objectives.contains(**entity))
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

/// An entity describing a dialogue node or a script
/// To activate the script, launch an [`InteractEvent`]
/// Either by having the entity itself be interactable, or by relaying the event.
/// ## See Also
/// [`InteractableEntity::interaction_relay`]
#[point_class(base(TargetName))]
#[derive(Eq, PartialEq, Clone, Debug)]
pub(crate) struct YarnNode {
	#[class(must_set)]
	pub(crate) yarn_node: String,
	/// Whether this node should avoid the restrictions placed upon dialogue.
	pub(crate) is_non_dialogue: bool,
}

impl Default for YarnNode {
	fn default() -> Self {
		Self {
			yarn_node: "".to_string(),
			is_non_dialogue: false,
		}
	}
}

fn talk_ify_yarnnode(
	on: On<Add, YarnNode>,
	interactable_query: Query<&InteractableEntity>,
	mut commands: Commands,
) {
	if let Ok(interaction) = interactable_query.get(on.entity) {
		commands
			.entity(on.entity)
			.insert(interaction.add_override("Talk"));
	}
}

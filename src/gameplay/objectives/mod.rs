use bevy::prelude::*;
use bevy_trenchbroom::prelude::*;

use crate::{
	gameplay::interaction::{InteractEvent, InteractableObject},
	props::logic_entity::ObjectiveEntity,
};

pub(crate) mod ui;

pub(super) fn plugin(app: &mut App) {
	app.add_plugins(ui::plugin);

	app.add_observer(update_current_objective);
	//	app.add_systems(Update, find_current_objective);
	app.add_observer(watch_for_completors);
	app.add_systems(PostUpdate, complete_parent_objectives);
}

/// Marker for entities that complete subobjectives on interact
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
#[require(InteractableObject(Some("Complete Objective".to_string())))]
pub struct ObjectiveCompletor {
	/// `ObjectiveEntity::targetname` of the objective completed by this completor
	pub target: String,
}

#[derive(Component)]
pub struct CurrentObjective;

/// A game objective.
#[derive(Default)]
#[base_class]
pub struct Objective {
	/// The description of the objective.
	pub description: String,
}

impl Objective {
	/// Creates a new [`Objective`] with the given description.
	pub fn new(description: impl Into<String>) -> Self {
		Self {
			description: description.into(),
		}
	}
}

/// Marker component for completed objectives.
#[derive(Component, Debug, Default)]
pub struct ObjectiveCompleted;

/// A relationship component linking a sub-objective to its parent objective.
#[derive(Component, Debug)]
#[relationship(relationship_target = SubObjectives)]
pub struct SubObjectiveOf {
	/// The parent objective entity.
	#[relationship]
	pub objective: Entity,
}

/// A relationship target component holding all sub-objectives of a parent objective.
#[derive(Component, Debug, Default, Deref)]
#[relationship_target(relationship = SubObjectiveOf)]
pub struct SubObjectives(Vec<Entity>);

#[derive(Component)]
#[relationship_target(relationship = PreviousObjective)]
pub struct NextObjective(Entity);

#[derive(Component)]
#[relationship(relationship_target = NextObjective)]
pub struct PreviousObjective(pub Entity);

fn watch_for_completors(
	trigger: On<InteractEvent>,
	objective_query: Query<(Entity, &ObjectiveEntity)>,
	completor_query: Query<&ObjectiveCompletor>,
	mut commands: Commands,
) {
	if let Ok(completor) = completor_query.get(trigger.0) {
		for (entity, objective) in objective_query.iter() {
			if objective.targetname == completor.target {
				commands.entity(entity).insert(ObjectiveCompleted);
			}
		}
	}
}

pub(crate) fn create_dialogue_objective(
	In((description, previous)): In<(String, Option<String>)>,
	mut commands: Commands,
	objectives: Query<(Entity, &Objective), Without<SubObjectiveOf>>,
) {
	if let Some(previous) = previous
		&& let Some((previous, _)) = objectives
			.iter()
			.find(|(_, objective)| objective.description == previous)
	{
		let objective = commands.spawn(Objective::new(description)).id();
		commands.entity(previous).insert(NextObjective(objective));
	} else {
		commands.spawn((CurrentObjective, Objective::new(description)));
	}
}

pub(crate) fn add_dialogue_objective_to_current(
	In(description): In<String>,
	mut commands: Commands,
	current_objective: Option<Single<Entity, With<CurrentObjective>>>,
) {
	if let Some(objective) = current_objective {
		commands.spawn((
			Objective::new(description),
			SubObjectiveOf {
				objective: *objective,
			},
		));
	}
}

pub(crate) fn complete_dialogue_objective(
	In(description): In<String>,
	mut commands: Commands,
	objectives: Query<(Entity, &Objective)>,
) {
	if let Some((objective, _)) = objectives
		.iter()
		.find(|(_, objective)| objective.description == description)
	{
		commands.entity(objective).insert(ObjectiveCompleted);
	}
}

pub(crate) fn get_dialogue_current_objective(
	current_objective: Option<Single<&Objective, With<CurrentObjective>>>,
) -> String {
	current_objective
		.map(|objective| objective.description.clone())
		.unwrap_or_default()
}

fn update_current_objective(
	add: On<Add, ObjectiveCompleted>,
	mut commands: Commands,
	objectives: Query<&NextObjective, With<CurrentObjective>>,
) {
	if let Ok(&NextObjective(next_objective)) = objectives.get(add.entity) {
		commands.entity(add.entity).try_remove::<CurrentObjective>();
		commands.entity(next_objective).try_insert(CurrentObjective);
	}
}

/// Marks parent objectives as completed when all their sub-objectives are completed.
// TODO (Jondolf): I wanted to handle this with an observer, but had problems where
//                 siblings of completed sub-objectives were not yet spawned, and thus it
//                 would incorrectly mark the parent objective as completed.
fn complete_parent_objectives(
	new_completed_query: Query<&SubObjectiveOf, Added<ObjectiveCompleted>>,
	sub_objectives_query: Query<&SubObjectives>,
	completed_query: Query<(), With<ObjectiveCompleted>>,
	mut commands: Commands,
) {
	for sub_objective_of in new_completed_query.iter() {
		// Check if all sibling sub-objectives are completed.
		if let Ok(sub_objectives) = sub_objectives_query.get(sub_objective_of.objective)
			&& completed_query.iter_many(sub_objectives.iter()).count() == sub_objectives.len()
		{
			// Mark the parent objective as completed.
			commands
				.entity(sub_objective_of.objective)
				.try_insert(ObjectiveCompleted);
		}
	}
}

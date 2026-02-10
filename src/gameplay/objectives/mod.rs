use bevy::prelude::*;

use crate::{gameplay::objectives::ui::spawn_objective_ui, screens::Screen};

pub(crate) mod ui;

pub(super) fn plugin(app: &mut App) {
	app.add_plugins(ui::plugin);

	app.add_systems(
		OnEnter(Screen::Gameplay),
		spawn_test_objectives.after(spawn_objective_ui),
	);
	// app.add_observer(set_current_objective);
	app.add_observer(update_current_objective);
	app.add_systems(PostUpdate, complete_parent_objectives);
}

#[derive(Component)]
pub struct CurrentObjective;

/// A game objective.
#[derive(Component, Debug, Default)]
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

fn spawn_test_objectives(mut commands: Commands) {
	// Spawn a top-level objective.
	commands
		.spawn((
			Objective::new("Task 1"),
			related!(SubObjectives[
				Objective::new("Task 1.1"),
				Objective::new("Task 1.2"),
				(Objective::new("Task 1.3"), ObjectiveCompleted)
			]),
			related!(NextObjective[
				Objective::new("Task 2"),
				related!(SubObjectives[
					(Objective::new("Task 2.1"), ObjectiveCompleted),
					(
						Objective::new("Task 2.2"),
						related!(SubObjectives[
							Objective::new("Task 2.2.1"),
							Objective::new("Task 2.2.2"),
						]),
					),
					Objective::new("Task 2.3")
				])
			]),
		))
		// If you want to hate ui remove this.
		.insert(CurrentObjective);
}

fn update_current_objective(
	add: On<Add, ObjectiveCompleted>,
	mut commands: Commands,
	objectives: Query<&NextObjective, With<CurrentObjective>>,
) {
	if let Ok(&NextObjective(next_objective)) = objectives.get(add.entity) {
		commands.entity(add.entity).remove::<CurrentObjective>();
		commands.entity(next_objective).insert(CurrentObjective);
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
				.insert(ObjectiveCompleted);
		}
	}
}

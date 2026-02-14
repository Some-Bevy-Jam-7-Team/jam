#[cfg(feature = "dev")]
use bevy::input::common_conditions::input_just_pressed;
use bevy::{
	ecs::{lifecycle::HookContext, world::DeferredWorld},
	prelude::*,
};
use bevy_trenchbroom::prelude::*;

use crate::{
	gameplay::{TargetName, TargetnameEntityIndex, interaction::InteractEvent},
	props::logic_entity::ObjectiveEntity,
	screens::Screen,
};

pub(crate) mod ui;

pub(super) fn plugin(app: &mut App) {
	app.add_plugins(ui::plugin);
	app.init_resource::<CurrentObjective>();

	app.add_systems(
		Update,
		(update_current_objective, trigger_all_objectives_done).chain(),
	);
	app.add_observer(watch_for_completions);
	app.add_systems(PostUpdate, complete_parent_objectives);
	#[cfg(feature = "dev")]
	app.add_systems(
		Update,
		(|mut commands: Commands| {
			commands.trigger(AllObjectivesDone);
		})
		.run_if(input_just_pressed(KeyCode::F10)),
	);
}

#[derive(Resource, Reflect, Debug, Deref, Default, PartialEq)]
#[reflect(Resource)]
pub struct CurrentObjective(Option<Entity>);

/// A game objective.
#[derive(Default)]
#[base_class]
#[component(on_add=Objective::on_add)]
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

	fn on_add(mut world: DeferredWorld, ctx: HookContext) {
		if world.is_scene_world() {
			return;
		}
		world
			.commands()
			.entity(ctx.entity)
			.insert(DespawnOnExit(Screen::Gameplay));
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

fn watch_for_completions(
	trigger: On<InteractEvent>,
	objective_query: Query<(), With<Objective>>,
	mut commands: Commands,
) {
	if objective_query.contains(trigger.0) {
		commands.entity(trigger.0).insert(ObjectiveCompleted);
	}
}

pub(crate) fn create_dialogue_objective(
	In((identifier, description, order)): In<(String, String, f32)>,
	mut commands: Commands,
) {
	commands.spawn((
		Name::new(format!("Objective: {identifier}")),
		TargetName::new(identifier),
		ObjectiveEntity {
			target: None,
			objective_order: order,
		},
		Objective::new(description),
	));
}

pub(crate) fn create_dialogue_subobjective(
	In((identifier, description, parent_identifier)): In<(String, String, String)>,
	mut commands: Commands,
) {
	commands.spawn((
		Name::new(format!("Subobjective: {identifier} of {parent_identifier}")),
		TargetName::new(identifier),
		ObjectiveEntity {
			target: Some(parent_identifier),
			objective_order: 0.0,
		},
		Objective::new(description),
	));
}

pub(crate) fn complete_dialogue_objective(
	In(identifier): In<String>,
	mut commands: Commands,
	objective_query: Query<(), With<Objective>>,
	entity_name_index: Res<TargetnameEntityIndex>,
) {
	for entity in entity_name_index
		.get_entity_by_targetname(&identifier)
		.iter()
		.filter(|entity| objective_query.contains(**entity))
	{
		commands.entity(*entity).insert(ObjectiveCompleted);
	}
}

pub(crate) fn get_dialogue_current_objective(
	current_objective: Res<CurrentObjective>,
	objective_query: Query<&TargetName>,
) -> String {
	(**current_objective)
		.and_then(|entity| objective_query.get(entity).ok())
		.map(|objective| (**objective).clone())
		.unwrap_or_default()
}

#[derive(Event)]
pub(crate) struct AllObjectivesDone;

fn trigger_all_objectives_done(
	not_done_objectives: Query<
		(Entity, &ObjectiveEntity),
		(Without<ObjectiveCompleted>, Without<SubObjectiveOf>),
	>,
	done_objectives: Query<
		(Entity, &ObjectiveEntity),
		(With<ObjectiveCompleted>, Without<SubObjectiveOf>),
	>,
	mut commands: Commands,
) {
	if not_done_objectives.is_empty() && !done_objectives.is_empty() {
		commands.trigger(AllObjectivesDone);
	}
}

fn update_current_objective(
	objectives: Query<
		(Entity, &ObjectiveEntity),
		(Without<ObjectiveCompleted>, Without<SubObjectiveOf>),
	>,
	mut current_objective: ResMut<CurrentObjective>,
) {
	let minimum = objectives
		.iter()
		.min_by(|(_, a), (_, b)| a.objective_order.total_cmp(&b.objective_order))
		.map(|(entity, _)| entity);
	current_objective.set_if_neq(CurrentObjective(minimum));
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

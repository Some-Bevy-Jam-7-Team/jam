use bevy::prelude::*;

use crate::{
	gameplay::objectives::{
		CurrentObjective, Objective, ObjectiveCompleted, SubObjectiveOf, SubObjectives,
	},
	screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
	app.add_observer(on_spawn_objective);
	app.add_observer(on_complete_objective);

	app.add_systems(OnEnter(Screen::Gameplay), spawn_objective_ui);
	app.add_systems(Update, update_objective_description_ui);
}

/// The UI node that holds all objectives.
#[derive(Component, Debug)]
pub struct ObjectiveUi;

/// A UI node representing a single objective.
#[derive(Component, Debug)]
pub struct ObjectiveNode;

/// A UI node representing a list of sub-objectives under a parent objective.
#[derive(Component, Debug)]
pub struct SubObjectiveListNode;

/// Links an [`Objective`] to a specific UI node in the world.
#[derive(Component, Debug)]
pub struct ObjectiveOfNode {
	pub node: Entity,
}

/// Spawns the main objective UI node.
pub fn spawn_objective_ui(mut commands: Commands) {
	commands.spawn((
		Name::new("Objective UI"),
		crate::ui_layout::RootWidget,
		DespawnOnExit(Screen::Gameplay),
		ObjectiveUi,
		Node {
			padding: UiRect::all(Val::Px(10.0)),
			flex_direction: FlexDirection::Column,
			row_gap: Val::Px(5.0),
			..default()
		},
		BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
	));
}

/// Creates a UI node for an objective with the given description.
fn objective_node(description: impl Into<String>, depth: usize) -> impl Bundle {
	(
		ObjectiveNode,
		Node {
			padding: UiRect::all(Val::Px(2.0)),
			flex_direction: FlexDirection::Column,
			row_gap: Val::Px((5.0 - depth as f32 * 0.5).max(2.0)),
			..default()
		},
		children![(
			Text::new(description),
			TextFont::from_font_size((16.0 - depth as f32 * 2.0).max(10.0)),
		)],
	)
}

/// Creates a UI node for a list of objectives.
fn sub_objective_list_node(depth: usize) -> impl Bundle {
	(
		SubObjectiveListNode,
		Node {
			flex_direction: FlexDirection::Column,
			padding: UiRect::left(Val::Px(20.0)),
			row_gap: Val::Px((5.0 - depth as f32 * 0.5).max(2.0)),
			..default()
		},
	)
}

/// Spawns a UI node for a new current objective.
fn on_spawn_objective(add: On<Add, CurrentObjective>, mut commands: Commands) {
	commands.run_system_cached_with(spawn_objective, add.entity);
}

// This is gross, but we use exclusive `World` access to make structural changes immediate.
// Using commands would really complicate this, since the logic depends on previously spawned nodes.
fn spawn_objective(
	In(objective_entity): In<Entity>,
	world: &mut World,
	objective_ui: &mut QueryState<Entity, With<ObjectiveUi>>,
	sub_objective_query: &mut QueryState<&SubObjectives>,
	sub_objective_of_query: &mut QueryState<&SubObjectiveOf>,
) {
	let Ok(objective_ui) = objective_ui.query(world).single() else {
		return;
	};

	let Some(objective_description) = world
		.get::<Objective>(objective_entity)
		.map(|o| o.description.clone())
	else {
		return;
	};

	let objective_node_entity = world
		.spawn((
			ChildOf(objective_ui),
			objective_node(objective_description, 0),
		))
		.id();

	world.entity_mut(objective_entity).insert(ObjectiveOfNode {
		node: objective_node_entity,
	});

	let descendants = sub_objective_query
		.query(world)
		.iter_descendants(objective_entity)
		.collect::<Vec<Entity>>();

	for sub_objective_entity in descendants {
		let Some(sub_objective_of) = world.get::<SubObjectiveOf>(sub_objective_entity) else {
			continue;
		};

		// This is a sub-objective; find the parent objective's UI node.
		let Some(parent_objective_node) = world.get::<ObjectiveOfNode>(sub_objective_of.objective)
		else {
			continue;
		};

		// Get the children of the parent objective node.
		let Some(children) = world.get::<Children>(parent_objective_node.node) else {
			continue;
		};

		// Determine the depth of this sub-objective in the hierarchy.
		let depth = sub_objective_of_query
			.query(world)
			.iter_ancestors(sub_objective_entity)
			.count();

		// Find or create the objective list node under the parent objective node.
		let objective_list_node_entity = if children.len() == 1 {
			// No objective list node yet; create one.
			world
				.spawn((
					ChildOf(parent_objective_node.node),
					sub_objective_list_node(depth),
				))
				.id()
		} else {
			// There is already an objective list node; use it.
			children[1]
		};

		let sub_objective_description = world
			.get::<Objective>(sub_objective_entity)
			.map(|o| o.description.clone())
			.unwrap();
		let objective_node_entity = world
			.spawn((
				ChildOf(objective_list_node_entity),
				objective_node(sub_objective_description, depth),
			))
			.id();

		world
			.entity_mut(sub_objective_entity)
			.insert(ObjectiveOfNode {
				node: objective_node_entity,
			});
	}
}

/// Updates the objective UI and completes parent objectives when an objective is completed.
fn on_complete_objective(
	completed: On<Insert, (ObjectiveCompleted, ObjectiveOfNode)>,
	objective_node_query: Query<&ObjectiveOfNode, With<ObjectiveCompleted>>,
	sub_objective_query: Query<&SubObjectiveOf>,
	child_query: Query<&Children>,
	mut commands: Commands,
) {
	// Update the objective UI to show the objective as completed.
	let Ok(children) = objective_node_query
		.get(completed.entity)
		.and_then(|node| child_query.get(node.node))
	else {
		return;
	};

	let text_entity = children[0];
	commands
		.entity(text_entity)
		.try_insert((Strikethrough, StrikethroughColor(Color::WHITE)));

	// Remove the sub-objectives from the world.
	if let Some(sub_objective_list) = children.get(1) {
		commands.entity(*sub_objective_list).despawn();
	}

	// If this objective is a top-level objective, despawn its node when completed.
	if !sub_objective_query.contains(completed.entity)
		&& let Ok(objective_of_node) = objective_node_query.get(completed.entity)
	{
		commands.entity(objective_of_node.node).try_despawn();
	}
}

/// Updates the objective description UI when an objective's description changes.
fn update_objective_description_ui(
	objectives: Query<(&Objective, &ObjectiveOfNode), Changed<Objective>>,
	mut text_query: Query<&mut Text>,
	child_query: Query<&Children>,
) {
	for (objective, objective_of_node) in objectives.iter() {
		let Ok(children) = child_query.get(objective_of_node.node) else {
			continue;
		};

		let Ok(mut text) = text_query.get_mut(children[0]) else {
			continue;
		};

		text.0 = objective.description.clone();
	}
}

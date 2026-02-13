use avian3d::prelude::{
	CollisionEnd, CollisionEventsEnabled, CollisionLayers, CollisionStart, Position, Sensor
};
use bevy::{
	ecs::{lifecycle::HookContext, world::DeferredWorld},
	prelude::*,
};

use bevy_trenchbroom::prelude::*;

use crate::{
	PostPhysicsAppSystems,
	gameplay::{
		TargetName, TargetnameEntityIndex,
		interaction::InteractEvent,
		objectives::{Objective, SubObjectiveOf},
		player::Player,
		scripting::ReflectionSystems,
	},
	props::interactables::InteractableEntity,
	reflection::ReflAppExt,
	third_party::avian3d::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
	app.register_dynamic_component::<ObjectiveEntity>()
		.register_dynamic_component::<YarnNode>()
		.register_dynamic_component::<TimerEntity>()
		.register_dynamic_component::<LogicSetter>()
		.register_dynamic_component::<LogicToggler>()
		.register_dynamic_component::<LogicDespawn>()
		.add_observer(interact_timers)
		.add_observer(uninitialise_objectives)
		.add_observer(talk_ify_yarnnode)
		.add_observer(on_sensor_start)
		.add_observer(on_sensor_end)
		.add_observer(run_setter)
		.add_observer(run_toggle)
		.add_observer(run_despawn)
		.add_observer(interact_teleport)
		.add_systems(
			Update,
			(
				initialise_objectives,
				tick_timers.in_set(PostPhysicsAppSystems::TickTimers),
			),
		);
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
/// Activates (completes) on [`InteractEvent`]
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
/// Either by having the entity itself be interactable, or by relaying the event.
/// Activates on [`InteractEvent`]
/// ## See Also
/// [`InteractableEntity::interaction_relay`]
#[point_class(base(TargetName))]
#[derive(Eq, PartialEq, Clone, Debug)]
pub(crate) struct YarnNode {
	/// Title of the yarn script that should be executed when this node is interacted with.
	#[class(must_set)]
	pub(crate) yarn_node: String,
	/// Whether this node should avoid the restrictions placed upon dialogue. TODO!
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

/// An entity describing a timer which triggers [`InteractEvent`] after some time.
/// Can also be used as a timed relay.
/// Activates on [`InteractEvent`]
#[point_class(base(TargetName))]
#[derive(PartialEq, Clone, Debug, Default)]
pub(crate) struct TimerEntity {
	/// How long this timer takes to finish (in seconds)
	pub timer_length: f32,
	/// How long this timer has already been going for
	pub timer_elapsed: f32,
	/// Entities to interact with upon timer completion
	pub timer_on_finish: Option<String>,
	/// Whether this timer is currently ticking, disabled upon completion, activated upon interaction
	pub timer_active: bool,
	/// Whether this timer should start again after it finishes
	pub timer_repeating: bool,
}

fn interact_timers(trigger: On<InteractEvent>, mut timer_query: Query<&mut TimerEntity>) {
	if let Ok(mut timer) = timer_query.get_mut(trigger.0) {
		timer.timer_active = true;
	}
}

fn tick_timers(
	mut timer_query: Query<&mut TimerEntity>,
	mut commands: Commands,
	entity_index: Res<TargetnameEntityIndex>,
	time: Res<Time>,
) {
	let dt = time.delta_secs();
	for mut timer in timer_query.iter_mut() {
		let mut timer_activated = false;
		if timer.timer_active {
			timer.timer_elapsed += dt;
			if timer.timer_elapsed >= timer.timer_length {
				timer_activated = true;
				if timer.timer_repeating {
					if timer.timer_length > 0.0 && timer.timer_length.is_finite() {
						timer.timer_elapsed =
							f32::rem_euclid(timer.timer_elapsed, timer.timer_length);
						if timer.timer_elapsed >= timer.timer_length {
							timer.timer_elapsed = 0.0;
						}
					}
				} else {
					timer.timer_elapsed = 0.0;
					timer.timer_active = false;
				}
			}
		}
		if timer_activated {
			if let Some(target) = &timer.timer_on_finish {
				for &entity in entity_index.get_entity_by_targetname(target) {
					commands.trigger(InteractEvent(entity));
				}
			}
		}
	}
}

/// A sensor entity describing an area that reacts to the presence of entities
#[solid_class(base(TargetName, Transform, Visibility))]
#[component(on_insert=SensorEntity::on_insert)]
#[component(immutable)]
#[derive(Default)]
pub(crate) struct SensorEntity {
	/// Entities to interact with when collision starts
	sensor_on_collision_start: Option<String>,
	/// Entities to interact with when collision ends
	sensor_on_collision_end: Option<String>,
	/// Whether the sensor detects the player
	sensor_detect_player: bool,
	/// Whether the sensor detects characters (like NPCs)
	sensor_detect_character: bool,
	/// Whether the sensor detects props
	sensor_detect_props: bool,
	/// Whether the sensor is disabled and will not respond to events
	sensor_disabled: bool,
}

impl SensorEntity {
	pub fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
		if world.is_scene_world() {
			return;
		}
		if let Some(values) = world.get::<SensorEntity>(ctx.entity) {
			let mut filter_layers: Vec<_> = vec![];
			if values.sensor_detect_player {
				filter_layers.push(CollisionLayer::PlayerCharacter);
			}
			if values.sensor_detect_character {
				filter_layers.push(CollisionLayer::Character);
			}
			if values.sensor_detect_props {
				filter_layers.push(CollisionLayer::Prop);
			}
			let sensor_disabled = values.sensor_disabled;
			let _ = values;

			world.commands().entity(ctx.entity).insert((
				Sensor,
				CollisionLayers::new([CollisionLayer::Sensor], filter_layers),
			));
			if sensor_disabled {
				world
					.commands()
					.entity(ctx.entity)
					.remove::<CollisionEventsEnabled>();
			} else {
				world
					.commands()
					.entity(ctx.entity)
					.insert(CollisionEventsEnabled);
			}
		}
	}
}

fn on_sensor_start(
	on: On<CollisionStart>,
	sensor_query: Query<&SensorEntity>,
	entity_index: Res<TargetnameEntityIndex>,
	mut commands: Commands,
) {
	if let Ok(sensor) = sensor_query.get(on.collider1) {
		if !sensor.sensor_disabled {
			if let Some(targetname) = &sensor.sensor_on_collision_start {
				for &entity in entity_index.get_entity_by_targetname(targetname) {
					commands.trigger(InteractEvent(entity));
				}
			}
		}
	}
}

fn on_sensor_end(
	on: On<CollisionEnd>,
	sensor_query: Query<&SensorEntity>,
	entity_index: Res<TargetnameEntityIndex>,
	mut commands: Commands,
) {
	if let Ok(sensor) = sensor_query.get(on.collider1) {
		if !sensor.sensor_disabled {
			if let Some(targetname) = &sensor.sensor_on_collision_end {
				for &entity in entity_index.get_entity_by_targetname(targetname) {
					commands.trigger(InteractEvent(entity));
				}
			}
		}
	}
}

/// An entity describing a change in properties of another entity.
///
/// Activates on [`InteractEvent`]
#[point_class(base(TargetName))]
#[derive(PartialEq, Clone, Debug, Default)]
pub(crate) struct LogicSetter {
	/// targetname of entity that this setter changes
	pub logic_setter_target: String,
	/// Name of the property to set
	pub logic_field_to_set: String,
	/// Value string of the property to set
	pub logic_value_to_set: String,
}

fn run_setter(
	trigger: On<InteractEvent>,
	setter_query: Query<&LogicSetter>,
	reflection_systems: Res<ReflectionSystems>,
	mut commands: Commands,
) {
	if let Ok(setter) = setter_query.get(trigger.0) {
		commands.run_system_with(
			reflection_systems.get_set_value_system(),
			(
				setter.logic_setter_target.clone(),
				setter.logic_field_to_set.clone(),
				setter.logic_value_to_set.clone(),
			),
		);
	}
}

/// An entity describing a change in properties of another entity.
///
/// Activates on [`InteractEvent`]
#[point_class(base(TargetName))]
#[derive(PartialEq, Clone, Debug, Default)]
pub(crate) struct LogicToggler {
	/// targetname of entity that this setter changes
	pub logic_toggle_target: String,
	/// Name of the property to toggle
	pub logic_field_to_toggle: String,
}

fn run_toggle(
	trigger: On<InteractEvent>,
	toggler_query: Query<&LogicToggler>,
	reflection_systems: Res<ReflectionSystems>,
	mut commands: Commands,
) {
	if let Ok(toggle) = toggler_query.get(trigger.0) {
		commands.run_system_with(
			reflection_systems.get_toggle_value_system(),
			(
				toggle.logic_toggle_target.clone(),
				toggle.logic_field_to_toggle.clone(),
			),
		);
	}
}

/// An entity describing a change in properties of another entity.
///
/// Activates on [`InteractEvent`]
#[point_class(base(TargetName))]
#[derive(PartialEq, Clone, Debug, Default)]
pub(crate) struct LogicDespawn {
	/// targetname of entity that should be despawned
	pub despawn_target: String,
}

fn run_despawn(
	trigger: On<InteractEvent>,
	despawner_query: Query<&LogicDespawn>,
	reflection_systems: Res<ReflectionSystems>,
	mut commands: Commands,
) {
	if let Ok(despawn) = despawner_query.get(trigger.0) {
		commands.run_system_with(
			reflection_systems.get_despawn_entity_system(),
			despawn.despawn_target.clone(),
		);
	}
}

/// An entity for teleportation destination
#[point_class(base(TargetName, Transform))]
#[derive(PartialEq, Clone, Debug, Default)]
pub(crate) struct TeleportNode {
	/// targetname of entity that should be teleported here
	pub teleport_target: Option<String>,
	/// Whether the player should be teleported too
	pub teleport_player: bool,
	/// targetname of entity to use as the origin, setting this will make the teleport use a relative offset (this - relative_to) it adds to the entity's transform.
	pub teleport_relative_to: Option<String>,
}

fn interact_teleport(
	trigger: On<InteractEvent>,
	teleport_query: Query<(&TeleportNode, &GlobalTransform)>,
	mut transform_query: Query<(&mut Transform, Option<&mut Position>)>,
	entity_index: Res<TargetnameEntityIndex>,
	player_query: Option<Single<Entity, With<Player>>>,
) {
	if let Ok((teleport, teleport_transform)) = teleport_query.get(trigger.0) {
		let relative = if let Some(name) = teleport.teleport_relative_to.as_ref() {
			#[allow(clippy::incompatible_msrv)]
			let Some(pos) = entity_index
				.get_entity_by_targetname(name)
				.as_array::<1>()
				.and_then(|x| transform_query.get(x[0]).ok())
				.map(|(transform, _)| transform.translation)
			else {
				error!(
					"Did not find a unique relative transform entity with name {:?}",
					teleport.teleport_relative_to
				);
				return;
			};
			Some(pos)
		} else {
			None
		};
		let position_mutator = |position: &mut Vec3| {
			if let Some(pos) = relative {
				*position += teleport_transform.translation() - pos;
			} else {
				*position = teleport_transform.translation();
			}
		};
		if let Some(targetname) = &teleport.teleport_target {
			for &entity in entity_index.get_entity_by_targetname(targetname) {
				if let Ok((mut transform, position)) = transform_query.get_mut(entity) {
					transform.translation = teleport_transform.translation();
					position_mutator(&mut transform.translation);
					if let Some(mut x) = position {
						**x = teleport_transform.translation();
						position_mutator(&mut x);
					}
				}
			}
		}
		if teleport.teleport_player {
			if let Some(player_entity) = player_query {
				if let Ok((mut transform, position)) = transform_query.get_mut(*player_entity) {
					transform.translation = teleport_transform.translation();
					position_mutator(&mut transform.translation);
					if let Some(mut x) = position {
						**x = teleport_transform.translation();
						position_mutator(&mut x);
					}
				}
			}
		}
	}
}

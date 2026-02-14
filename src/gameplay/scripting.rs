//! Yarnspinner commands that allow us to do arbitrary scripting from within yarnspinner

use std::any::TypeId;

use bevy::{
	ecs::{change_detection::MutUntyped, system::SystemId},
	log,
	prelude::*,
	reflect::{DynamicStruct, ReflectFromPtr, ReflectMut, ReflectRef, TypeRegistry},
};

use crate::{
	gameplay::{TargetName, TargetnameEntityIndex, interaction::InteractEvent},
	reflection::{DynamicPropertyMap, DynamicallyModifiableType},
};

pub(super) fn plugin(app: &mut App) {
	app.init_resource::<ReflectionSystems>();
}

#[derive(Resource, Debug)]
pub(crate) struct ReflectionSystems {
	set_value_system: SystemId<In<(String, String, String)>>,
	toggle_value_system: SystemId<In<(String, String)>>,
	despawn_entity_system: SystemId<In<String>>,
}

impl FromWorld for ReflectionSystems {
	fn from_world(world: &mut World) -> Self {
		ReflectionSystems {
			set_value_system: world.register_system(set_value_on_entity),
			toggle_value_system: world.register_system(toggle_bool_on_entity),
			despawn_entity_system: world.register_system(despawn_entity),
		}
	}
}

impl ReflectionSystems {
	pub fn get_set_value_system(&self) -> SystemId<In<(String, String, String)>> {
		self.set_value_system
	}

	pub fn get_toggle_value_system(&self) -> SystemId<In<(String, String)>> {
		self.toggle_value_system
	}

	pub fn get_despawn_entity_system(&self) -> SystemId<In<String>> {
		self.despawn_entity_system
	}
}

pub(crate) fn despawn_entity(
	name: In<String>,
	entity_index: Res<TargetnameEntityIndex>,
	mut commands: Commands,
) {
	for entity in entity_index.get_entity_by_targetname(&name) {
		commands.entity(*entity).despawn();
	}
}

pub fn set_value_on_entity(input: In<(String, String, String)>, world: &mut World) {
	let (targetname, field_name, value_string) = (*input).clone();
	mutate_component_on_entity_by_names(
		&targetname,
		&field_name,
		&mut |mut reflect_borrow, dyn_type| {
			let Some(value) = dyn_type.parse_string(&value_string) else {
				warn!(
					"Failed to set a value {value_string} on entity {targetname} in field {field_name}, because it could not be parsed as {dyn_type:?}"
				);
				return;
			};
			let mut override_value = DynamicStruct::default();
			override_value.insert_boxed(&field_name, value);
			reflect_borrow
				.as_partial_reflect_mut()
				.apply(&override_value);
		},
		world,
	);
}

pub fn toggle_bool_on_entity(input: In<(String, String)>, world: &mut World) {
	let (targetname, field_name) = (*input).clone();
	mutate_component_on_entity_by_names(
		&targetname,
		&field_name,
		&mut |mut reflect_borrow, _dyn_type| {
			match reflect_borrow.as_partial_reflect_mut().reflect_mut() {
				ReflectMut::Struct(struct_data) => {
					if let Some(inner_data) = struct_data.field_mut(&field_name) {
						if let Some(value) = inner_data.try_downcast_mut::<bool>() {
							*value = !*value;
						} else if let Some(Some(value)) =
							inner_data.try_downcast_mut::<Option<bool>>()
						{
							*value = !*value;
						} else {
							warn!(
								"Could not parse field {field_name} of entity {targetname} as a bool"
							)
						}
					}
				}
				_ => {
					// Should not happen
				}
			}
		},
		world,
	);
}

pub fn interact_with_entity(
	input: In<String>,
	mut commands: Commands,
	interactables: Query<(Entity, &TargetName)>,
) {
	for (entity, name) in interactables.iter() {
		if name.targetname == input.0 {
			commands.trigger(InteractEvent(entity));
		}
	}
	warn!(
		"Failed to interact with {}: no such targetname found",
		input.0
	);
}

pub fn read_bool_from_entity(input: In<(String, String)>, world: &mut World) -> bool {
	let (targetname, field_name) = (*input).clone();
	let mut result = false;
	mutate_component_on_entity_by_names(
		&targetname,
		&field_name,
		&mut |reflect_borrow, _dyn_type| {
			match reflect_borrow.as_partial_reflect().reflect_ref() {
				ReflectRef::Struct(struct_data) => {
					if let Some(inner_data) = struct_data.field(&field_name) {
						if let Some(value) = inner_data.try_downcast_ref::<bool>() {
							if *value {
								result = true;
							}
						} else if let Some(value) = inner_data.try_downcast_ref::<Option<bool>>() {
							if value.is_some_and(|x| x) {
								result = true;
							}
						} else {
							warn!(
								"Could not parse field {field_name} of entity {targetname} as a bool"
							)
						}
					}
				}
				_ => {
					// Should not happen
				}
			}
		},
		world,
	);
	result
}

fn mutate_component_on_entity_by_names(
	targetname: &str,
	field_name: &str,
	mutator: &mut dyn FnMut(Mut<dyn Reflect>, DynamicallyModifiableType),
	world: &mut World,
) {
	world.resource_scope::<DynamicPropertyMap, ()>(
		|world, prop_index: Mut<'_, DynamicPropertyMap>| {
			let Some(&(component_id, component_type, ref field_type)) = prop_index.get(field_name)
			else {
				warn!(
					"Did not set a value in {field_name} because there isn't such a registered field."
				);
				return;
			};
			world.resource_scope::<TargetnameEntityIndex, ()>(|world, entity_index| {
				for &entity in entity_index.get_entity_by_targetname(targetname) {
					// Only modify entities which have the desired component
					if !world
						.get_entity(entity)
						.is_ok_and(|x| x.contains_id(component_id))
					{
						return;
					}
					world.resource_scope::<AppTypeRegistry, ()>(|world, type_registry| {
						let type_registry: std::sync::RwLockReadGuard<'_, TypeRegistry> =
							type_registry.read();
						world.entity_mut(entity).modify_component_by_id::<()>(
							component_id,
							|untyped| {
								// SAFETY: Surely I didn't fuck up the componentid and component_type pairing, right?
								match unsafe {
									mut_untyped_to_reflect(
										untyped,
										&type_registry,
										component_type.type_id(),
									)
								} {
									Ok(value) => mutator(value, *field_type),
									Err(error) => {
										log::error!("{:?}", error);
									}
								}
							},
						);
					});
				}
			});
		},
	);
}

// Shoutout bevy_inspector_egui for this code

// SAFETY: MutUntyped is of type with `type_id`
unsafe fn mut_untyped_to_reflect<'a>(
	value: MutUntyped<'a>,
	type_registry: &TypeRegistry,
	type_id: TypeId,
) -> Result<Mut<'a, dyn Reflect>> {
	let registration = type_registry
		.get(type_id)
		.ok_or_else(|| format!("No type registration for type ID: {type_id:?}"))?;
	let reflect_from_ptr = registration
		.data::<ReflectFromPtr>()
		.ok_or_else(|| format!("No type data for type ID: {type_id:?}"))?;

	assert_eq!(reflect_from_ptr.type_id(), type_id);

	let value = value.map_unchanged(|ptr| {
		// SAFETY: ptr is of type type_id as required in safety contract, type_id was checked above
		unsafe { reflect_from_ptr.as_reflect_mut(ptr) }
	});

	Ok(value)
}

//! Yarnspinner commands that allow us to do arbitrary scripting from within yarnspinner

use std::any::TypeId;

use bevy::{
	ecs::change_detection::MutUntyped, log, prelude::*, reflect::{DynamicStruct, ReflectFromPtr, TypeRegistry}
};
use bevy_inspector_egui::restricted_world_view::Error;

use crate::{gameplay::TargetnameEntityIndex, reflection::{DynamicPropertyMap, DynamicallyModifiableType}};

pub(super) fn plugin(_app: &mut App) {}

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
	mutate_component_on_entity_by_names(&targetname, &field_name, &|mut reflect_borrow, dyn_type| {
		let Some(value) = dyn_type.parse_string(&value_string) else {
			warn!("Failed to set a value {value_string} on entity {targetname} in field {field_name}, because it could not be parsed as {dyn_type:?}");
			return;
		};
		let mut override_value = DynamicStruct::default();
		override_value.insert_boxed(&field_name, value);
		reflect_borrow.as_partial_reflect_mut().apply(&override_value);
	}, world);
}

fn mutate_component_on_entity_by_names(targetname: &str, field_name: &str, mutator: &dyn Fn(Mut<dyn Reflect>, DynamicallyModifiableType), world: &mut World) {
	world.resource_scope::<DynamicPropertyMap, ()>(|world, prop_index: Mut<'_, DynamicPropertyMap>| {
		let Some(&(component_id, component_type, ref field_type)) = prop_index.get(field_name) else {
			warn!("Did not set a value in {field_name} because there isn't such a registered field.");
			return;
		};
		world.resource_scope::<TargetnameEntityIndex, ()>(|world, entity_index| {
			for &entity in entity_index.get_entity_by_targetname(targetname) {
				// Only modify entities which have the desired component
				if !world.entity(entity).contains_id(component_id) {
					return;
				}
				world.resource_scope::<AppTypeRegistry, ()>(|world, type_registry| {
					let type_registry: std::sync::RwLockReadGuard<'_, TypeRegistry> = type_registry.read();
					world.entity_mut(entity).modify_component_by_id::<()>(component_id, |untyped| {
						// SAFETY: Surely I didn't fuck up the componentid and component_type pairing, right?
						match unsafe { mut_untyped_to_reflect(untyped, &type_registry, component_type.type_id()) } {
							Ok(value) => {
								mutator(value, *field_type)
							}
							Err(error) => {
								log::error!("{:?}", error);
								return;
							}
						}
					});
				});
			}
		});
	});
}

// Shoutout bevy_inspector_egui for this code

// SAFETY: MutUntyped is of type with `type_id`
unsafe fn mut_untyped_to_reflect<'a>(
    value: MutUntyped<'a>,
    type_registry: &TypeRegistry,
    type_id: TypeId,
) -> Result<Mut<'a, dyn Reflect>, Error> {
    let registration = type_registry
        .get(type_id)
        .ok_or(Error::NoTypeRegistration(type_id))?;
    let reflect_from_ptr = registration
        .data::<ReflectFromPtr>()
        .ok_or(Error::NoTypeData(type_id, "ReflectFromPtr"))?;

    assert_eq!(reflect_from_ptr.type_id(), type_id);

    let value = value.map_unchanged(|ptr| {
        // SAFETY: ptr is of type type_id as required in safety contract, type_id was checked above
        unsafe { reflect_from_ptr.as_reflect_mut(ptr) }
    });

    Ok(value)
}

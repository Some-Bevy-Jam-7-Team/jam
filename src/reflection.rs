use std::any::TypeId;

use bevy::{
	ecs::component::ComponentId,
	platform::collections::HashMap,
	prelude::*,
	reflect::{TypeInfo, Typed},
};

pub(super) fn plugin(app: &mut App) {
	app.init_resource::<DynamicPropertyMap>();
}

pub trait ReflAppExt {
	/// Registers a [`DynamicComponent`] to be usable by scripting. (by adding it into [`DynamicPropertyMap`])
	/// # Note:
	/// Can be called only AFTER `reflection::plugin` has been added.
	fn register_dynamic_component<T: DynamicComponent + Component>(&mut self) -> &mut Self;
}

impl ReflAppExt for App {
	fn register_dynamic_component<T: DynamicComponent + Component>(&mut self) -> &mut Self {
		let id = self.world_mut().register_component::<T>();
		T::register(
			&mut *self.world_mut().resource_mut::<DynamicPropertyMap>(),
			id,
		);
		self
	}
}

#[derive(Debug, Clone, Copy)]
pub struct DynamicallyModifiableType {
	option: bool,
	value_type: DynamicallyModifiableTypeKind,
}

#[derive(Debug, Clone, Copy)]
pub enum DynamicallyModifiableTypeKind {
	String,
	Bool,
	F32,
}

impl DynamicallyModifiableType {
	fn from_type_id(type_id: &TypeId) -> Option<DynamicallyModifiableType> {
		if *type_id == TypeId::of::<String>() {
			return Some(DynamicallyModifiableType {
				option: false,
				value_type: DynamicallyModifiableTypeKind::String,
			});
		}
		if *type_id == TypeId::of::<bool>() {
			return Some(DynamicallyModifiableType {
				option: false,
				value_type: DynamicallyModifiableTypeKind::Bool,
			});
		}
		if *type_id == TypeId::of::<f32>() {
			return Some(DynamicallyModifiableType {
				option: false,
				value_type: DynamicallyModifiableTypeKind::F32,
			});
		}
		if *type_id == TypeId::of::<Option<String>>() {
			return Some(DynamicallyModifiableType {
				option: true,
				value_type: DynamicallyModifiableTypeKind::String,
			});
		}
		if *type_id == TypeId::of::<Option<bool>>() {
			return Some(DynamicallyModifiableType {
				option: true,
				value_type: DynamicallyModifiableTypeKind::Bool,
			});
		}
		if *type_id == TypeId::of::<Option<f32>>() {
			return Some(DynamicallyModifiableType {
				option: true,
				value_type: DynamicallyModifiableTypeKind::F32,
			});
		}
		return None;
	}

	pub fn parse_string(&self, value: &str) -> Option<Box<dyn PartialReflect>> {
		if self.option == true && *value == *"" {
			Some(match self.value_type {
				DynamicallyModifiableTypeKind::String => Option::<String>::None.to_dynamic(),
				DynamicallyModifiableTypeKind::Bool => Option::<bool>::None.to_dynamic(),
				DynamicallyModifiableTypeKind::F32 => Option::<f32>::None.to_dynamic(),
			})
		} else {
			if self.option {
				match self.value_type {
					DynamicallyModifiableTypeKind::String => Some(Some(value.to_string()).to_dynamic()),
					DynamicallyModifiableTypeKind::Bool => value.parse::<bool>().ok().map(|x| Some(x).to_dynamic()),
					DynamicallyModifiableTypeKind::F32 => value.parse::<f32>().ok().map(|x| Some(x).to_dynamic()),
				}
			} else {
				match self.value_type {
					DynamicallyModifiableTypeKind::String => Some(value.to_string().to_dynamic()),
					DynamicallyModifiableTypeKind::Bool => value.parse::<bool>().ok().map(|x| x.to_dynamic()),
					DynamicallyModifiableTypeKind::F32 => value.parse::<f32>().ok().map(|x| x.to_dynamic()),
				}
			}
		}
	}
}

#[derive(Resource, Debug, Default)]
pub struct DynamicPropertyMap {
	map: HashMap<String, (ComponentId, &'static TypeInfo, DynamicallyModifiableType)>,
}

impl DynamicPropertyMap {
	/// Gets the [`ComponentId`], [`TypeInfo`] of component, and [`TypeInfo`] of field registered to this fieldname.
	pub fn get(
		&self,
		key: &str,
	) -> Option<&(ComponentId, &'static TypeInfo, DynamicallyModifiableType)> {
		self.map.get(key)
	}
}

/// A dynamic component whose values can be potentially changed via scripting
/// needs to be registered with [`App::register_dynamic_component`]
pub trait DynamicComponent {
	fn register(prop_map: &mut DynamicPropertyMap, id: ComponentId);
}

impl<T: Typed> DynamicComponent for T {
	fn register(prop_map: &mut DynamicPropertyMap, id: ComponentId) {
		let TypeInfo::Struct(struct_info) = T::type_info() else {
			panic!(
				"You can register only structs for dynamic editing! {} isn't a struct.",
				T::type_path()
			);
		};
		for field in struct_info.iter() {
			let Some(dyn_type) = DynamicallyModifiableType::from_type_id(&field.type_id()) else {
				info!("Reflection: Skipping field {} with type {} because it is not supported", field.name(), field.type_path());
				continue;
			};
			prop_map.map.insert(
				field.name().to_string(),
				(
					id,
					T::type_info(),
					dyn_type,
				),
			);
		}
	}
}

//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::{
	app::HierarchyPropagatePlugin,
	camera::visibility::RenderLayers,
	ecs::{lifecycle::HookContext, world::DeferredWorld},
	platform::collections::HashMap,
	prelude::*,
};
use bevy_trenchbroom::prelude::*;

use crate::screens::Screen;

mod animation;
pub(crate) mod core;
pub(crate) mod crosshair;
pub(crate) mod dialogue_view;
pub(crate) mod hud;
pub(crate) mod interaction;
pub(crate) mod level;
pub(crate) mod npc;
pub(crate) mod objectives;
pub(crate) mod player;
pub(crate) mod scripting;
pub(crate) mod stomach;

pub(crate) mod fever;

pub(super) fn plugin(app: &mut App) {
	app.init_resource::<TargetnameEntityIndex>().add_plugins((
		scripting::plugin,
		animation::plugin,
		crosshair::plugin,
		dialogue_view::plugin,
		npc::plugin,
		objectives::plugin,
		player::plugin,
		stomach::plugin,
		// This plugin preloads the level,
		// so make sure to add it last.
		level::plugin,
		core::plugin,
		interaction::plugin,
		hud::plugin,
		fever::plugin,
		HierarchyPropagatePlugin::<RenderLayers>::new(PostUpdate),
	));
}

// "We have many to many relationships at home" - Sun Tzu - The quick art of polyamory with ecs characteristics.

/// Acceleration map for looking up entities by name based on their `targetname`
#[derive(Resource, Reflect, Debug, Default)]
#[reflect(Resource)]
pub struct TargetnameEntityIndex {
	targetname_to_entity: HashMap<String, Vec<Entity>>,
	entity_to_targetname: HashMap<Entity, String>,
}

#[expect(dead_code)]
impl TargetnameEntityIndex {
	pub fn get_entity_by_targetname(&self, targetname: &str) -> &[Entity] {
		self.targetname_to_entity
			.get(targetname)
			.map(|v| &**v)
			.unwrap_or(&[])
	}

	pub fn get_targetname_of_entity(&self, entity: Entity) -> Option<&str> {
		self.entity_to_targetname.get(&entity).map(|x| x.as_str())
	}

	/// Registers an entity with a given targetname, if the targetname is empty, skips it.
	pub fn register_entity(&mut self, entity: Entity, targetname: &str) {
		if targetname.is_empty() {
			return;
		}
		self.targetname_to_entity
			.entry(targetname.to_string())
			.or_default()
			.push(entity);
		self.entity_to_targetname
			.insert(entity, targetname.to_string());
	}

	/// Removes an entity from the index
	pub fn remove_entity(&mut self, entity: Entity) {
		if let Some(name) = self.entity_to_targetname.remove(&entity) {
			if let Some(list) = self.targetname_to_entity.get_mut(&name) {
				list.retain(|e| *e != entity)
			}
		}
	}

	pub fn clear(&mut self) {
		self.entity_to_targetname.clear();
		self.targetname_to_entity.clear();
	}
}

/// The targetname of an entity, it's like a [`Name`], but you can author it, and you can look up entities by it virute of [`TargetnameEntityIndex`].
/// The component is immutable, dereference it to obtain the value, insert a new one to change it.
#[base_class]
#[component(on_insert=TargetName::on_insert)]
#[component(on_remove=TargetName::on_replace)]
#[component(immutable)]
#[derive(Deref)]
pub struct TargetName {
	targetname: String,
}

impl TargetName {
	pub fn new(targetname: impl Into<String>) -> Self {
		TargetName {
			targetname: targetname.into(),
		}
	}

	fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
		if world.is_scene_world() {
			return;
		}
		if let Some(targetname) = world.get::<TargetName>(ctx.entity) {
			// Yes I know the clone to borrow is stupid but I aint figuring out how to convince borrow checker otherwise
			let targetname = targetname.targetname.clone();
			if let Some(mut resource) = world.get_resource_mut::<TargetnameEntityIndex>() {
				resource.register_entity(ctx.entity, &targetname);
			}
			world
				.commands()
				.entity(ctx.entity)
				.insert(DespawnOnExit(Screen::Gameplay));
		}
	}

	fn on_replace(mut world: DeferredWorld, ctx: HookContext) {
		if world.is_scene_world() {
			return;
		}
		if let Some(mut resource) = world.get_resource_mut::<TargetnameEntityIndex>() {
			resource.remove_entity(ctx.entity);
		}
	}
}

impl Default for TargetName {
	fn default() -> Self {
		TargetName {
			targetname: "".to_string(),
		}
	}
}

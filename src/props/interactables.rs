use bevy::{
	ecs::{lifecycle::HookContext, world::DeferredWorld},
	prelude::*,
};

use bevy_trenchbroom::prelude::*;

use crate::{
	gameplay::{
		interaction::InteractableObject, objectives::ObjectiveCompletor, stomach::EdibleProp,
	},
	reflection::ReflAppExt,
};

pub(super) fn plugin(app: &mut App) {
	app.register_dynamic_component::<InteractableEntity>();
}

/// Trenchbroom component for designing entities that can be interacted with.
#[derive(Default, Clone)]
#[base_class]
#[component(on_add = InteractableEntity::on_add)]
pub struct InteractableEntity {
	/// Whether this entity should be
	is_edible: bool,
	/// Whether this entity should have a special line of text for being interacted with or it should be inferred from being edible.
	interaction_text_override: Option<String>,
	/// What objective, if any, should be completed by this name. Should be the `targetname` of said objective.
	completes_subobjective: Option<String>,
}

impl InteractableEntity {
	pub fn on_add(mut world: DeferredWorld, ctx: HookContext) {
		if world.is_scene_world() {
			return;
		}
		if let Some(values) = world.get::<InteractableEntity>(ctx.entity).cloned() {
			if let Some(override_text) = values.interaction_text_override {
				world
					.commands()
					.entity(ctx.entity)
					.insert_if_new(InteractableObject(Some(override_text)));
			}
			if values.is_edible {
				world
					.commands()
					.entity(ctx.entity)
					.insert_if_new(EdibleProp);
			}
			if let Some(objective_name) = values.completes_subobjective {
				world
					.commands()
					.entity(ctx.entity)
					.insert_if_new(ObjectiveCompletor {
						target: objective_name,
					});
			}
		}
	}
}

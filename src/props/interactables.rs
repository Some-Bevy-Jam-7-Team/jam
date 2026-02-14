use bevy::{
	ecs::{lifecycle::HookContext, world::DeferredWorld},
	prelude::*,
};

use bevy_trenchbroom::prelude::*;

use crate::{gameplay::stomach::EdibleProp, reflection::ReflAppExt};

pub(super) fn plugin(app: &mut App) {
	app.register_dynamic_component::<InteractableEntity>();
}

/// Trenchbroom component for designing entities that can be interacted with.
#[derive(Default, Clone)]
#[base_class]
#[component(on_insert = InteractableEntity::on_insert)]
#[component(immutable)]
pub struct InteractableEntity {
	/// Whether this entity should be
	pub is_edible: bool,
	/// Whether this entity should have a special line of text for being interacted with or it should be inferred from being edible.
	pub interaction_text_override: Option<String>,
	/// What objective, if any, should be completed by this name. Should be the `targetname` of said objective.
	pub completes_subobjective: Option<String>,
	/// What entity, if any, should additionally receive [`InteractEvent`](crate::gameplay::interaction::InteractEvent) when this one activates.
	pub interaction_relay: Option<String>,
}

#[expect(dead_code)]
impl InteractableEntity {
	pub fn on_insert(mut world: DeferredWorld, ctx: HookContext) {
		if world.is_scene_world() {
			return;
		}
		if let Some(values) = world.get::<InteractableEntity>(ctx.entity).cloned() {
			if values.is_edible {
				world
					.commands()
					.entity(ctx.entity)
					.insert_if_new(EdibleProp);
			} else {
				world.commands().entity(ctx.entity).remove::<EdibleProp>();
			}
		}
	}

	pub fn new_from_text(text: String) -> Self {
		InteractableEntity {
			is_edible: false,
			interaction_text_override: Some(text),
			completes_subobjective: None,
			interaction_relay: None,
		}
	}

	/// Gets a string roughly representing the action of this entity.
	///
	pub fn get_hover_text(&self) -> Option<&str> {
		if self.interaction_text_override.is_some() {
			self.get_interaction_text_override()
		} else if self.is_edible {
			Some("Eat")
		} else if self.is_active() {
			Some("Interact")
		} else {
			None
		}
	}

	/// Whether this [`InteractableEntity`] has any non-default values that would make it warrant an interaction.
	pub fn is_active(&self) -> bool {
		self.is_edible
			|| self.interaction_text_override.is_some()
			|| self.completes_subobjective.is_some()
			|| self.interaction_relay.is_some()
	}

	pub fn get_is_edible(&self) -> bool {
		self.is_edible
	}

	pub fn get_interaction_text_override(&self) -> Option<&str> {
		self.interaction_text_override.as_deref()
	}

	pub fn get_completes_subobjective(&self) -> Option<&str> {
		self.completes_subobjective.as_deref()
	}

	pub fn get_interaction_relay(&self) -> Option<&str> {
		self.interaction_relay.as_deref()
	}

	/// Adds an override text, if there was none previously and returns the new component
	#[must_use]
	pub fn add_override(&self, text: &str) -> Self {
		let mut value = self.clone();
		if value.interaction_text_override.is_none() {
			value.interaction_text_override = Some(text.to_string());
		}
		value
	}
}

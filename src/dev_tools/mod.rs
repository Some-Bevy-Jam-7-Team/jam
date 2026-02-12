//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{dev_tools::states::log_transitions, prelude::*};

mod debug_ui;
mod input;
pub(crate) mod log_components;
mod validate_preloading;

use crate::{
	gameplay::interaction::{InteractEvent, InteractableObject},
	menus::Menu,
	screens::loading::LoadingScreen,
};

pub(super) fn plugin(app: &mut App) {
	// Log `Screen` state transitions.
	app.add_systems(
		Update,
		(log_transitions::<Menu>, log_transitions::<LoadingScreen>).chain(),
	);

	app.add_observer(interacted_entity);

	app.add_plugins((
		debug_ui::plugin,
		input::plugin,
		validate_preloading::plugin,
		log_components::plugin,
	));
}

fn interacted_entity(
	event: On<InteractEvent>,
	names: Query<&Name>,
	interactions: Query<&InteractableObject>,
) {
	info!(
		"Interacted with: {}, with name: {:?} and interaction text: {:?}",
		event.0,
		names.get(event.0).ok(),
		interactions
			.get(event.0)
			.ok()
			.map(|interaction| interaction.0.as_ref()),
	)
}

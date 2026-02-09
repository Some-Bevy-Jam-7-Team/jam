//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{dev_tools::states::log_transitions, prelude::*};

mod debug_ui;
mod input;
pub(crate) mod log_components;
mod validate_preloading;

use crate::{menus::Menu, screens::loading::LoadingScreen};

pub(super) fn plugin(app: &mut App) {
	// Log `Screen` state transitions.
	app.add_systems(
		Update,
		(log_transitions::<Menu>, log_transitions::<LoadingScreen>).chain(),
	);

	app.add_observer(clicked_entity);

	app.add_plugins((
		debug_ui::plugin,
		input::plugin,
		validate_preloading::plugin,
		log_components::plugin,
	));
}

fn clicked_entity(event: On<Pointer<Click>>, names: Query<&Name>) {
	info!(
		"Clicked on: {}, with name: {:?}",
		event.entity,
		names.get(event.entity).ok()
	)
}

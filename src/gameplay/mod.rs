//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

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
pub(crate) mod stomach;

pub(super) fn plugin(app: &mut App) {
	app.add_plugins((
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
	));
}

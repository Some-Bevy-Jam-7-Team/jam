//! The game's main screen states and transitions between them.

mod credits;
mod kaleidoscope_background;
mod level_select;
mod main;
mod pause;
mod settings;

use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
	app.init_state::<Menu>();

	app.add_plugins((
		credits::plugin,
		level_select::plugin,
		main::plugin,
		settings::plugin,
		pause::plugin,
		kaleidoscope_background::plugin,
	));
}

/// The game's main screen states.
#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Default)]
#[states(scoped_entities)]
pub(crate) enum Menu {
	#[default]
	None,
	Main,
	LevelSelect,
	Credits,
	Settings,
	Pause,
}

#![allow(warnings)]
//! Modified version of `bevy_yarnspinner_example_dialogue_view`.

#![allow(clippy::too_many_arguments, clippy::type_complexity)]
#![warn(missing_docs, missing_debug_implementations)]

use bevy::prelude::*;
use bevy_yarnspinner::prelude::YarnSpinnerPlugin;
pub use setup::UiRootNode;
pub use updating::SpeakerChangeEvent;

mod assets;
mod option_selection;
mod setup;
mod typewriter;
mod updating;

pub mod prelude {
	//! Everything you need to get starting using this example Yarn Spinner dialogue view.
	pub use super::{DialogueViewSystemSet, SpeakerChangeEvent};
}

pub(super) fn plugin(app: &mut App) {
	assert!(
		app.is_plugin_added::<YarnSpinnerPlugin>(),
		"YarnSpinnerPlugin must be added before DialogueViewPlugin"
	);
	app.add_plugins(assets::ui_assets_plugin)
		.add_plugins(setup::ui_setup_plugin)
		.add_plugins(updating::ui_updating_plugin)
		.add_plugins(typewriter::typewriter_plugin)
		.add_plugins(option_selection::option_selection_plugin);
}

/// The [`SystemSet`] containing all systems added by the dialogue view plugin.
/// Is run after the [`YarnSpinnerSystemSet`](bevy_yarnspinner::prelude::YarnSpinnerSystemSet).
#[derive(Debug, Default, Clone, Copy, SystemSet, Eq, PartialEq, Hash)]
pub struct DialogueViewSystemSet;

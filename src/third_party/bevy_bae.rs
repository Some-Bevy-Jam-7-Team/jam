use bevy::prelude::*;
use bevy_bae::BaePlugin;

pub(super) fn plugin(app: &mut App) {
	app.add_plugins(BaePlugin::default());
}

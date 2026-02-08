//! [Hanabi](https://github.com/djeedai/bevy_hanabi) is our GPU particle system.

use bevy::prelude::*;
use bevy_hanabi::prelude::*;

pub(super) fn plugin(app: &mut App) {
	app.add_plugins(HanabiPlugin);
}

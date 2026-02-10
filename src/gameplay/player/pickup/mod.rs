//! Player pickup handling.

use avian_pickup::prop::HeldProp;
use bevy::prelude::*;

mod collision;
mod sound;
mod ui;

pub(super) fn plugin(app: &mut App) {
	app.add_plugins((collision::plugin, sound::plugin, ui::plugin));
}

#[expect(dead_code)]
pub(crate) fn is_holding_prop(q_prop: Query<&HeldProp>) -> bool {
	!q_prop.is_empty()
}

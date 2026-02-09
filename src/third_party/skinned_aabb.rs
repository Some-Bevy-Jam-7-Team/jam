use bevy::prelude::*;
use bevy_mod_skinned_aabb::prelude::*;

pub(super) fn plugin(app: &mut App) {
	app.add_plugins(SkinnedAabbPlugin);
}

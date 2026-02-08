//! Player pickup sound effects.

use crate::gameplay::player::Player;
use avian_pickup::output::PropThrown;
use bevy::prelude::*;
use bevy_seedling::prelude::*;

use crate::{
	PostPhysicsAppSystems, audio::SpatialPool, gameplay::player::assets::PlayerAssets,
	screens::Screen,
};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(
		Update,
		play_throw_sound
			.run_if(in_state(Screen::Gameplay).and(on_message::<PropThrown>))
			.in_set(PostPhysicsAppSystems::PlaySounds),
	);
}

fn play_throw_sound(
	mut commands: Commands,
	player_assets: Res<PlayerAssets>,
	player_transform: Single<&Transform, With<Player>>,
) {
	let sound = player_assets.throw_sound.clone();

	commands.spawn((
		**player_transform,
		SamplePlayer::new(sound).with_volume(Volume::Linear(3.0)),
		SpatialPool,
	));
}

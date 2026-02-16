use bevy::prelude::*;
use bevy::transform::components::GlobalTransform;
use bevy_seedling::prelude::VolumeNode;
use firewheel::Volume;

use crate::gameplay::player::Player;
use crate::props::specific::mouth::Mouth;

#[derive(Component)]
pub(crate) struct CommunePlugin;

impl Plugin for CommunePlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			PostUpdate,
			mouth_proximity.after(TransformSystems::Propagate),
		);
	}
}

#[derive(Component)]
pub struct MouthInfluence;

fn mouth_proximity(
	mouth: Query<&GlobalTransform, With<Mouth>>,
	player: Query<&GlobalTransform, With<Player>>,
	mut volume: Single<&mut VolumeNode, With<MouthInfluence>>,
) {
	if let Ok(mouth) = mouth.single()
		&& let Ok(player) = player.single()
	{
		let distance = mouth.translation().distance(player.translation());

		let fade_distance = 20.0;
		let gain = if distance > fade_distance {
			Volume::UNITY_GAIN
		} else {
			let volume = distance / fade_distance;
			Volume::Linear(volume * volume)
		};

		volume.volume = gain;
	} else {
		volume.volume = Volume::UNITY_GAIN;
	}
}

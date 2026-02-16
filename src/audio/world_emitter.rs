use bevy::prelude::*;
use bevy_seedling::prelude::*;
use bevy_trenchbroom::prelude::*;
use rand::Rng;

use crate::audio::{SpatialPool, doppler::DopplerSound};

pub struct EmitterPlugin;

impl Plugin for EmitterPlugin {
	fn build(&self, app: &mut App) {
		app.add_observer(observe_world_emitter);
	}
}

#[point_class(base(Transform, Visibility))]
pub struct WorldEmitter {
	source: WorldSounds,
	// volume in decibels
	volume: f32,
	// the unit scale is a bit unintuitive -- it sets
	// the scale of the units, meaning larger values result
	// in smaller sound radii
	unit_scale: f32,
	random_pitch_range: f32,
	random_start_range: f32,
}

impl Default for WorldEmitter {
	fn default() -> Self {
		Self {
			source: WorldSounds::Computer,
			volume: 0.0,
			unit_scale: 4.0,
			random_pitch_range: 0.05,
			random_start_range: 15.0,
		}
	}
}

#[derive(PartialEq, Eq, Hash, Reflect, FgdType)]
enum WorldSounds {
	Corpo,
	Corpo2,
	Computer,
	Light1,
	Light2,
	Voices,
	Mouth,
}

fn observe_world_emitter(
	trigger: On<Insert, WorldEmitter>,
	emitter: Query<&WorldEmitter>,
	mut commands: Commands,
	assets: Res<AssetServer>,
) -> Result {
	let emitter = emitter.get(trigger.entity)?;
	let sound = match emitter.source {
		WorldSounds::Corpo => assets.load("audio/music/corpo slop to eat your computer to.ogg"),
		WorldSounds::Corpo2 => assets.load("audio/music/corpo slorpo feverrrrrrrr.ogg"),
		WorldSounds::Computer => assets.load("audio/sound_effects/office/computer.ogg"),
		WorldSounds::Light1 => assets.load("audio/sound_effects/office/fluorescent-light-1.ogg"),
		WorldSounds::Light2 => assets.load("audio/sound_effects/office/fluorescent-light-2.ogg"),
		WorldSounds::Voices => assets.load("audio/sound_effects/office/voices.ogg"),
		WorldSounds::Mouth => assets.load("audio/sound_effects/mouth.ogg"),
	};

	let start = if emitter.random_start_range <= 0.0 {
		0.0
	} else {
		rand::rng().random_range(0.0..emitter.random_start_range)
	};

	commands.entity(trigger.entity).insert((
		SamplePlayer::new(sound.clone())
			.looping()
			.with_volume(Volume::Decibels(emitter.volume)),
		PlaybackSettings::default()
			.remove()
			.with_play_from(PlayFrom::Seconds(start as f64)),
		DopplerSound { strength: 0.5 },
		SpatialPool,
		RandomPitch::new(emitter.random_pitch_range as f64),
		sample_effects![(
			SpatialBasicNode::default(),
			SpatialScale(Vec3::splat(emitter.unit_scale))
		)],
	));

	Ok(())
}

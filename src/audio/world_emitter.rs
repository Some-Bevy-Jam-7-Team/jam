use bevy::{platform::collections::HashMap, prelude::*};
use bevy_seedling::prelude::*;
use bevy_trenchbroom::prelude::*;
use rand::Rng;

use crate::{
	audio::{SpatialPool, doppler::DopplerSound},
	menus::Menu,
};

pub struct EmitterPlugin;

impl Plugin for EmitterPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<SoundMap>()
			.add_observer(observe_world_emitter)
			.add_systems(OnExit(Menu::None), pause_world_emitters)
			.add_systems(OnEnter(Menu::None), play_world_emitters);
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

#[derive(Resource)]
struct SoundMap(HashMap<WorldSounds, Handle<AudioSample>>);

impl FromWorld for SoundMap {
	fn from_world(world: &mut World) -> Self {
		let map = HashMap::from_iter([
			(
				WorldSounds::Corpo,
				world.load_asset("audio/music/corpo slop to eat your computer to.ogg"),
			),
			(
				WorldSounds::Computer,
				world.load_asset("audio/sound_effects/office/computer.ogg"),
			),
			(
				WorldSounds::Light1,
				world.load_asset("audio/sound_effects/office/fluorescent-light-1.ogg"),
			),
			(
				WorldSounds::Light2,
				world.load_asset("audio/sound_effects/office/fluorescent-light-2.ogg"),
			),
			(
				WorldSounds::Voices,
				world.load_asset("audio/sound_effects/office/voices.ogg"),
			),
		]);

		Self(map)
	}
}

#[derive(PartialEq, Eq, Hash, Reflect, FgdType)]
enum WorldSounds {
	Corpo,
	Computer,
	Light1,
	Light2,
	Voices,
}

fn observe_world_emitter(
	trigger: On<Insert, WorldEmitter>,
	emitter: Query<&WorldEmitter>,
	map: Res<SoundMap>,
	mut commands: Commands,
) -> Result {
	let emitter = emitter.get(trigger.entity)?;
	let sound = map
		.0
		.get(&emitter.source)
		.ok_or("Failed to find world sound")?;

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

fn pause_world_emitters(emitters: Query<&mut PlaybackSettings, With<WorldEmitter>>) {
	for mut emitter in emitters {
		emitter.pause();
	}
}

fn play_world_emitters(emitters: Query<&mut PlaybackSettings, With<WorldEmitter>>) {
	for mut emitter in emitters {
		if !*emitter.play {
			emitter.play_from = PlayFrom::Resume;
			emitter.play();
		}
	}
}

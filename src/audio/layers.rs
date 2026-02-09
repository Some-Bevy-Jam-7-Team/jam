use bevy::prelude::*;
use bevy_seedling::{
	pool::{CompletionReason, Sampler},
	prelude::*,
	sample::QueuedSample,
};

use crate::audio::{MusicPool, animation::play_at};

pub fn plugin(app: &mut App) {
	app.add_systems(PostUpdate, LayeredMusic::update_layers)
		.add_observer(stop_music)
		.add_observer(play_music)
		.add_observer(add_active)
		.add_observer(remove_active);
}

#[derive(Component, Reflect)]
pub struct LayeredMusic {
	/// Controls the number of active layers, expressed from 0 to 1.
	pub amount: f32,
}

impl LayeredMusic {
	const HYSTERESIS: f32 = 0.05;

	fn iter_layers(
		&self,
		layers: impl ExactSizeIterator<Item = (Entity, bool)>,
		mut updater: impl FnMut(Entity, Option<bool>),
	) -> Result {
		let num_layers = layers.len() as f32;
		let mut layers = layers.enumerate();
		let (_, (first_entity, first_active)) = layers
			.next()
			.ok_or("Music layers must have at least one child")?;

		if !first_active {
			updater(first_entity, Some(true));
		}

		for (i, (entity, is_active)) in layers {
			let threshold = i as f32 / num_layers;

			let next_state = if is_active {
				self.amount > (threshold - Self::HYSTERESIS)
			} else {
				self.amount >= threshold
			};

			if next_state != is_active {
				updater(entity, Some(next_state));
			}
		}

		Ok(())
	}

	fn update_layers(
		music: Query<(&Self, &Children), With<ActiveMusic>>,
		layers: Query<Has<ActiveLayer>>,
		mut commands: Commands,
	) -> Result {
		for (amount, children) in music {
			amount.iter_layers(
				children
					.iter()
					.map(|e| (e, layers.get(e).unwrap_or_default())),
				|entity, new_state| match new_state {
					Some(true) => {
						commands.entity(entity).insert(ActiveLayer);
					}
					Some(false) => {
						commands.entity(entity).remove::<ActiveLayer>();
					}
					_ => {}
				},
			)?;
		}

		Ok(())
	}
}

#[derive(Component)]
pub struct ActiveMusic;

#[derive(Component)]
pub struct Layer(pub Handle<AudioSample>);

#[derive(Component)]
struct ActiveLayer;

#[derive(Component)]
pub struct Intro {
	pub sample: Handle<AudioSample>,
	/// The duration of the intro in seconds.
	///
	/// The main layers will be delayed by this amount when queued for playback.
	pub duration: DurationSeconds,
}

/// Trigger playback of layered music.
#[derive(EntityEvent)]
pub struct PlayLayeredMusic(pub Entity);

/// Halt playback of layered music.
#[derive(EntityEvent)]
pub struct StopLayeredMusic(pub Entity);

fn play_music(
	trigger: On<PlayLayeredMusic>,
	music: Query<(&LayeredMusic, Option<&Intro>, &Children)>,
	layers: Query<&Layer>,
	time: Res<Time<Audio>>,
	mut commands: Commands,
) -> Result {
	let (amount, intro, children) = music.get(trigger.0)?;
	let default_volume = Volume::Decibels(9.0);

	commands.entity(trigger.0).insert(ActiveMusic);

	match intro {
		Some(intro) => {
			commands.entity(trigger.0).insert((
				MusicPool,
				SamplePlayer::new(intro.sample.clone()).with_volume(default_volume),
				PlaybackSettings::default().remove(),
			));

			for entity in children.iter() {
				let layer = layers.get(entity)?;
				let (player, settings, events) = play_at(
					SamplePlayer::new(layer.0.clone())
						.looping()
						.with_volume(default_volume),
					&time,
					intro.duration.0,
				);

				commands.entity(entity).insert((
					MusicPool,
					player,
					settings.remove(),
					events,
					sample_effects![VolumeNode::from_linear(0.0)],
				));
			}
		}
		None => {
			for entity in children.iter() {
				let layer = layers.get(entity)?;
				commands.entity(entity).insert((
					MusicPool,
					SamplePlayer::new(layer.0.clone())
						.looping()
						.with_volume(default_volume),
					PlaybackSettings::default().remove(),
					sample_effects![VolumeNode::from_linear(0.0)],
				));
			}
		}
	}

	amount.iter_layers(children.iter().map(|e| (e, false)), |entity, new_state| {
		if let Some(true) = new_state {
			commands.entity(entity).insert(ActiveLayer);
		}
	})?;

	Ok(())
}

// TODO: we should probably fade out or have some graceful stop
fn stop_music(
	trigger: On<StopLayeredMusic>,
	music: Query<(&Children, Has<SamplePlayer>), With<LayeredMusic>>,
	layers: Query<Has<SamplePlayer>>,
	mut commands: Commands,
) -> Result {
	let (children, has_sample_player) = music.get(trigger.0)?;

	if has_sample_player {
		commands.trigger(PlaybackCompletion {
			entity: trigger.0,
			reason: CompletionReason::PlaybackInterrupted,
		});
	}

	commands.entity(trigger.0).remove::<ActiveMusic>();

	for entity in children.iter() {
		if layers.get(entity)? {
			commands
				.entity(entity)
				.despawn_related::<SampleEffects>()
				.remove_with_requires::<(SamplePlayer, Sampler, QueuedSample, AudioEvents)>();
		}
		commands.entity(entity).remove::<ActiveLayer>();
	}

	Ok(())
}

fn add_active(
	trigger: On<Add, ActiveLayer>,
	layer: Query<&SampleEffects>,
	mut volume: Query<(&VolumeNode, &mut AudioEvents)>,
) -> Result {
	let layer = layer.get(trigger.entity)?;
	let (node, mut events) = volume.get_effect_mut(layer)?;

	node.fade_to(Volume::Linear(1.0), DurationSeconds(1.0), &mut events);
	Ok(())
}

fn remove_active(
	trigger: On<Remove, ActiveLayer>,
	layer: Query<&SampleEffects>,
	mut volume: Query<(&VolumeNode, &mut AudioEvents)>,
) -> Result {
	let Ok(layer) = layer.get(trigger.entity) else {
		return Ok(());
	};
	let (node, mut events) = volume.get_effect_mut(layer)?;

	node.fade_to(Volume::Linear(0.0), DurationSeconds(1.0), &mut events);
	Ok(())
}

use bevy::prelude::*;
use bevy_seedling::firewheel::nodes::svf::SvfNode;
use bevy_seedling::prelude::*;

/// Sets up all the components required to start audio playback at a particular time.
pub fn play_at(
	player: SamplePlayer,
	time: &Time<Audio>,
	delay: f64,
) -> (SamplePlayer, PlaybackSettings, AudioEvents) {
	let mut events = AudioEvents::new(time);
	let settings = PlaybackSettings {
		play: Notify::new(false),
		..Default::default()
	};
	settings.play_at(None, time.delay(DurationSeconds(delay)), &mut events);

	(player, settings, events)
}

pub trait AnimateCutoff {
	fn animate_cutoff(
		&self,
		target: f32,
		duration: f64,
		time: &Time<Audio>,
		events: &mut AudioEvents,
	);
}

impl<const C: usize> AnimateCutoff for SvfNode<C> {
	fn animate_cutoff(
		&self,
		frequency: f32,
		duration: f64,
		time: &Time<Audio>,
		events: &mut AudioEvents,
	) {
		events.schedule_tween(
			time.now(),
			time.now() + DurationSeconds(duration),
			*self,
			{
				let mut target = *self;
				target.cutoff_hz = frequency;
				target
			},
			30,
			|a, b, t| {
				let t = if a.cutoff_hz >= b.cutoff_hz {
					1.0 - (1.0 - t).powi(2)
				} else {
					t * t
				};

				let mut new_value = *a;
				new_value.cutoff_hz = a.cutoff_hz.interpolate_stable(&b.cutoff_hz, t);
				new_value
			},
		);
	}
}

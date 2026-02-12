use bevy::prelude::*;

use crate::gameplay::core::*;

#[derive(EntityEvent)]
pub struct FeverTick {
	pub entity: Entity,
}

pub fn source(
	time: Res<Time>,
	q_fever_source: Query<&FeverSource>,
	mut q_affected: Query<(&mut Temperature, &FeverSources, &mut FeverSourceTimer)>,
) {
	for (mut temp, fever_sources) in
		q_affected
			.iter_mut()
			.filter_map(|(temp, fever_sources, mut timer)| {
				timer.tick(time.delta());
				timer.is_finished().then(|| {
					(
						temp,
						fever_sources
							.iter()
							.filter_map(|e| q_fever_source.get(e).ok()),
					)
				})
			}) {
		for fever_source in fever_sources {
			**temp *= **fever_source;
		}
	}
}

pub fn fever(
	mut cmd: Commands,
	mut q_affected: Query<(
		Entity,
		&mut FeverTimer,
		&Temperature,
		&TemperatureThreshold,
		&mut Health,
		&FeverDamage,
		&BaseTemperature,
		&MaxTemperature,
	)>,
	time: Res<Time>,
) {
	for (entity, mut timer, temp, threshold, mut health, dmg, base, max) in &mut q_affected {
		if **temp > **base {
			cmd.entity(entity).insert(Feverish);
		} else {
			cmd.entity(entity).remove::<Feverish>();
		}

		timer.tick(time.delta());
		if !timer.is_finished() {
			continue;
		}

		cmd.trigger(FeverTick { entity });

		// Remove when UI exists
		println!("fever:{:?}, health:{:?}", temp, health);

		if **temp > **threshold {
			**health -= (**temp - **threshold) * **dmg;
			**health = health.max(0.);
		}

		if **temp > **max {
			// TODO die?
		}
	}
}

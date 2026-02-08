use bevy::prelude::*;

use crate::gameplay::core::*;

pub fn source(
	time: Res<Time>,
	mut query: Query<(&mut Temperature, &mut FeverSource, &mut FeverSourceTimer)>,
) {
	let dt = time.delta_secs();
	for (mut temp, rate, mut timer) in &mut query {
		timer.tick(time.delta());
		if !timer.is_finished() {
			continue;
		}
		**temp += **rate * dt;
	}
}

pub fn fever(
	mut cmd: Commands,
	mut units: Query<(
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
	for (entity, mut timer, temp, threshold, mut health, dmg, base, max) in &mut units {
		if **temp > **base {
			cmd.entity(entity).insert(Feverish);
		} else {
			cmd.entity(entity).remove::<Feverish>();
		}

		timer.tick(time.delta());
		if !timer.is_finished() {
			continue;
		}

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

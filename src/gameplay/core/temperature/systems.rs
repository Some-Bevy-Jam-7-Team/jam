use avian3d::prelude::*;
use bevy::prelude::*;

use crate::gameplay::core::*;
use crate::gameplay::stomach::Stomach;

/// Simulates thermal transfer by weighting global temperature with
/// collision-based and eaten temperature sources.
pub fn temp(
	time: Res<Time>,
	mut units: Query<(
		&mut Temperature,
		&BaseTemperature,
		&TemperatureSensors,
		Option<&Conductivity>,
		Option<&DepthSensitivity>,
	)>,
	global_temp: Res<GlobalTemperature>,
	q_sensors: Query<&CollidingEntities, With<TemperatureSensor>>,
	env_temps: Query<&EnvironmentTemperature>,
	collisions: Collisions,
	stomach: Single<&Stomach>,
) {
	let delta_seconds = time.delta_secs();

	for (mut temp, temp_base, sensors, conductivity, depth_sens) in &mut units {
		let depth_sens = depth_sens.cloned().unwrap_or_default();
		let (temp_weighted, total_weight) = sensors
			.iter()
			.filter_map(|sensor| Some((sensor, q_sensors.get(sensor).ok()?)))
			.flat_map(|(sensor, hits)| hits.iter().map(move |hit| (sensor, hit)))
			.filter_map(|(sensor, hit)| {
				let temp = env_temps.get(*hit).ok()?;

				let penetration = collisions
					.get(sensor, *hit)
					.and_then(|pair| pair.find_deepest_contact())
					.map(|p| p.penetration)
					.unwrap_or(0.0);

				let weight = 1.0 + (penetration * *depth_sens).max(0.0);

				Some((temp, weight))
			})
			.chain(
				stomach
					.contents
					.iter()
					.filter_map(|e| env_temps.get(*e).ok().map(|t| (t, *depth_sens))),
			)
			.fold(
				(**global_temp, 1.0),
				|(acc_temp, acc_weight), (env_temp, weight)| {
					(acc_temp + (**env_temp * weight), acc_weight + weight)
				},
			);

		let temp_env = temp_weighted / total_weight;

		// Newton's law of cooling
		let k = conductivity.cloned().unwrap_or_default();

		// rate = k * dt
		let rate = (*k * delta_seconds).min(1.);

		// temp += (Target - Current) * (k * dt)
		let temp_final = **temp + (temp_env - **temp) * rate;

		// Prevent the temperature from dropping too low, i.e., below body temp.
		let freezing = **temp < **temp_base;
		let too_low = temp_final < **temp_base;
		if too_low && !freezing {
			**temp = **temp_base;
		} else {
			**temp = temp_final;
		}
	}
}

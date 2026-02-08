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
        &Children,
        Option<&Conductivity>,
        &Stomach,
    )>,
    global_temp: Res<GlobalTemperature>,
    sensors: Query<&CollidingEntities, With<TemperatureSensor>>,
    env_temps: Query<&EnvironmentTemperature>,
    collisions: Collisions,
) {
    let delta_seconds = time.delta_secs();

    for (mut temp, temp_base, children, conductivity, stomach) in &mut units {
        let (temp_weighted, total_weight) = children
            .iter()
            .filter_map(|child| sensors.get(child).ok().map(|hits| (child, hits)))
            .flat_map(|(child, hits)| hits.iter().map(move |hit| (child, hit)))
            .filter_map(|(child, hit)| {
                let temp = env_temps.get(*hit).ok()?;

                let penetration = collisions
                    .get(child, *hit)
                    .and_then(|pair| pair.find_deepest_contact())
                    .map(|p| p.penetration)
                    .unwrap_or(0.0);

                // Might have to adjust depth sensitivity (10x) and play with higher env temps instead.
                let weight = 1.0 + (penetration * 10.0).max(0.0);

                Some((temp, weight))
            })
            .chain(
                stomach
                    .contents
                    .iter()
                    .filter_map(|e| env_temps.get(*e).ok().map(|t| (t, 1.))),
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

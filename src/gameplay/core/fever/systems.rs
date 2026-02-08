use bevy::prelude::*;

use crate::gameplay::core::*;

pub fn tick_source(time: Res<Time>, mut query: Query<(&mut Temperature, &FeverSource)>) {
    let dt = time.delta_secs();
    for (mut temp, rate) in &mut query {
        **temp += **rate * dt;
    }
}

pub fn tick_fever(
    mut cmd: Commands,
    time: Res<Time>,
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
        }

        if **temp > **max {
            // TODO die?
        }
    }
}

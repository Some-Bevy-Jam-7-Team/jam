use bevy::prelude::*;

use crate::gameplay::core::*;

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
        timer.tick(time.delta());

        if !timer.is_finished() {
            continue;
        }

        // Remove when UI exists
        println!("fever:{:?}, health:{:?}", temp, health);

        if **temp > **threshold {
            **health -= (**temp - **threshold) * **dmg;
        }

        if **temp > **base {
            cmd.entity(entity).insert(Feverish);
        } else {
            cmd.entity(entity).remove::<Feverish>();
        }

        if **temp > **max {
            // TODO die?
        }
    }
}

use bevy::prelude::*;

use crate::gameplay::core::*;

pub fn tick_fever(
    time: Res<Time>,
    mut units: Query<(
        &mut FeverTimer,
        &Temperature,
        &MaxTemperature,
        &mut Health,
        &FeverDamage,
    )>,
) {
    for (mut timer, temp, max_temp, mut health, dmg) in &mut units {
        timer.tick(time.delta());

        if !timer.is_finished() {
            continue;
        }

        // Remove when UI exists
        println!("fever:{:?}, health:{:?}", temp, health);

        if **temp > **max_temp {
            **health -= (**temp - **max_temp) * **dmg;
        }
    }
}

use crate::gameplay::core::{fever::tick_fever, temperature::systems::tick_temperature, *};

pub struct FeverPlugin;

impl Plugin for FeverPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalTemperature>()
            .add_systems(FixedUpdate, (tick_fever, tick_temperature).chain());
    }
}

use crate::gameplay::core::{fever::systems::*, temperature::systems::*, *};

pub struct FeverPlugin;

impl Plugin for FeverPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalTemperature>()
            .add_systems(FixedUpdate, (tick_source, tick_temp, tick_fever).chain());
    }
}

use bevy::app::prelude::*;

use super::{systems::*, *};

pub struct TemperaturePlugin;

impl Plugin for TemperaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalTemperature>()
            .add_systems(FixedUpdate, temp);
    }
}

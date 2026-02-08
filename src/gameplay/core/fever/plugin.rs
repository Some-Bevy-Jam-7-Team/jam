use bevy::app::prelude::*;

use crate::gameplay::core::{fever::systems::*, temperature::*, *};

pub struct FeverPlugin;

impl Plugin for FeverPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<TemperaturePlugin>() {
            app.add_plugins(TemperaturePlugin);
        }

        app.add_systems(FixedUpdate, (source, fever).chain());
    }
}

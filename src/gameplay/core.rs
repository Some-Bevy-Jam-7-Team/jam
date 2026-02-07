/// Can be split up and renamed later.
use bevy::prelude::*;
use std::time::Duration;

/// Temperature of an entity
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct Temperature(pub f32);

impl Default for Temperature {
    fn default() -> Self {
        Self(37.0)
    }
}

/// Max temperature that a unit can handle before it takes damage.
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct MaxTemperature(pub f32);

impl Default for MaxTemperature {
    fn default() -> Self {
        Self(40.0)
    }
}

/// Base damage of a unit/entity (could be modified by temperature/fever).
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct Damage(pub f32);

impl Default for Damage {
    fn default() -> Self {
        Self(10.)
    }
}

/// Global temperature of the world
#[derive(Resource, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Resource)]
pub struct GlobalTemperature(pub f32);

impl Default for GlobalTemperature {
    fn default() -> Self {
        Self(20.)
    }
}

/// Temperature of objects/entities/space in the environment, affecting the temperature of units around them.
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct EnvironmentTemperature(pub f32);

impl Default for EnvironmentTemperature {
    fn default() -> Self {
        Self(40.)
    }
}

/// Base health of a unit/entity (could be modified by temperature).
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct Health(pub f32);

impl Default for Health {
    fn default() -> Self {
        Self(100.)
    }
}

/// Timer used to manage the `fever over time` effect. Allows for slowing down or speeding up the effect.
#[derive(Component, Debug, Deref, DerefMut, Clone, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct FeverTimer(pub Timer);

impl Default for FeverTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(1), TimerMode::Repeating))
    }
}

/// Timer used to manage the Temperature of units, i.e., adjusting based on global/env temp.
#[derive(Component, Debug, Deref, DerefMut, Clone, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct TemperatureTimer(pub Timer);

pub struct FeverPlugin;

impl Plugin for FeverPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalTemperature>()
            .add_systems(FixedUpdate, (tick_fever, tick_temperature).chain());
    }
}

fn tick_temperature(
    time: Res<Time>,
    mut units: Query<(&mut TemperatureTimer, &mut Temperature)>,
    global_temperature: Res<GlobalTemperature>,
    environment_temperatures: Query<&EnvironmentTemperature>,
) {
    // TODO look up collisions with EnvironmentTemperature entities
    for (mut timer, mut temp) in units.iter_mut() {
        timer.tick(time.delta());

        if timer.is_finished() {
            // cool down / heat up depending on global/env temperature
        }
    }
}

fn tick_fever(
    time: Res<Time>,
    mut units: Query<(&mut FeverTimer, &Temperature, &MaxTemperature, &mut Health)>,
) {
    for (mut timer, temp, max_temp, mut health) in units.iter_mut() {
        timer.tick(time.delta());

        if timer.is_finished() && **temp > **max_temp {
            **health -= 1.;
        }
    }
}

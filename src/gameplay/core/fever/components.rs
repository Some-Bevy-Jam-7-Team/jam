use crate::gameplay::core::*;
use std::time::Duration;

/// Makes a unit affected by fever.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
#[require(
    Temperature,
    MaxTemperature,
    BaseTemperature,
    TemperatureThreshold,
    Health,
    FeverTimer,
    FeverDamage
)]
pub struct Fever;

/// Marker component for units that are currently feverish (temp higher than base temp).
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct Feverish;

/// Amount of damage caused by fewer per-tick times degrees Celsius.
///
/// E.g., When 5 deg above max temp and damage is 10, 1 tick is 50 damage.
#[derive(Component, Debug, Deref, DerefMut, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct FeverDamage(pub f32);

impl Default for FeverDamage {
    fn default() -> Self {
        Self(5.)
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

/// A rate that represents an internal source of fever/heat that raises the entity's temperature over time.
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct FeverSource(pub f32);

impl Default for FeverSource {
    fn default() -> Self {
        Self(1.0)
    }
}

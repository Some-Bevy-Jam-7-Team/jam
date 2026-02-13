use crate::gameplay::core::*;
use std::time::Duration;

/// Makes a unit affected by fever.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Clone, Debug, Component)]
#[require(Temperature, Health, FeverTimer, FeverDamage, FeverSources)]
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
		Self(1.)
	}
}

/// Timer used to manage the `fever over time` effect, i.e., the interval in which fever applies damage.
///
/// Allows for slowing down or speeding up the effect.
#[derive(Component, Debug, Deref, DerefMut, Clone, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct FeverTimer(pub Timer);

impl Default for FeverTimer {
	fn default() -> Self {
		Self(Timer::new(Duration::from_secs(1), TimerMode::Repeating))
	}
}

/// Timer used to manage the `fever source` effect, i.e., the rate of a debuff that increases fever.
///
/// Allows for slowing down or speeding up the effect.
#[derive(Component, Debug, Deref, DerefMut, Clone, Reflect)]
#[reflect(Clone, Debug, Component)]
pub struct FeverSourceTimer(pub Timer);

impl Default for FeverSourceTimer {
	fn default() -> Self {
		Self(Timer::new(Duration::from_secs(1), TimerMode::Repeating))
	}
}

/// The parent entity with [`Fever`] that this [`FeverSource`] belongs to.
#[derive(Component, Debug, Deref, Reflect, Clone, Copy)]
#[reflect(Component, Clone, Debug)]
#[relationship(relationship_target = FeverSources)]
pub struct FeverSourceOf(#[relationship] pub Entity);

#[derive(Component, Debug, Clone, Deref, DerefMut, Reflect, Default)]
#[reflect(Clone, Debug, Component, Default)]
#[relationship_target(relationship = FeverSourceOf)]
pub struct FeverSources(Vec<Entity>);

/// A rate that represents an internal source of fever/heat that raises the entity's temperature over time.
///
/// Defaults to 1% per tick.
#[derive(Component, Debug, Clone, Copy, Deref, DerefMut, Reflect)]
#[reflect(Clone, Debug, Component)]
#[require(Temperature, FeverSourceTimer)]
pub struct FeverSource(pub f32);

impl Default for FeverSource {
	fn default() -> Self {
		Self(1.01)
	}
}

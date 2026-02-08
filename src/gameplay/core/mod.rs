use bevy::prelude::*;
use plugin::FeverPlugin;

pub mod components;
pub mod fever;
pub mod plugin;
pub mod temperature;

pub use components::*;
pub use fever::*;
pub use temperature::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(FeverPlugin);
}

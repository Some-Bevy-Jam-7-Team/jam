pub mod components;
pub mod plugin;
pub mod resources;
pub mod systems;

use bevy::app::App;

pub use components::*;
pub use plugin::*;
pub use resources::*;

pub fn plugin(app: &mut App) {
	app.add_plugins(TemperaturePlugin);
}

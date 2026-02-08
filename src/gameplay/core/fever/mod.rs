pub mod components;
pub mod plugin;
pub mod systems;

use bevy::app::App;

pub use components::*;
pub use plugin::*;
pub use systems::*;

pub fn plugin(app: &mut App) {
    app.add_plugins(FeverPlugin);
}

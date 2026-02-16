use bevy::app::App;

mod grass;
pub mod postprocess;

pub fn plugin(app: &mut App) {
	postprocess::plugin(app);
	grass::plugin(app);
}

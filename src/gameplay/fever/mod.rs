use bevy::app::App;

pub mod postprocess;

pub fn plugin(app: &mut App) {
	postprocess::plugin(app)
}

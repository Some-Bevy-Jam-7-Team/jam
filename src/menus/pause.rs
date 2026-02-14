//! The pause menu.

use std::any::Any as _;

use crate::{
	gameplay::{crosshair::CrosshairState, player::input::BlocksInput},
	menus::Menu,
	screens::Screen,
	theme::widget,
	ui_layout::RootWidget,
};
use bevy::{input::common_conditions::input_just_pressed, prelude::*};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);
	app.add_systems(
		Update,
		go_back.run_if(in_state(Menu::Pause).and(input_just_pressed(KeyCode::Escape))),
	);
}

fn spawn_pause_menu(
	mut commands: Commands,
	mut crosshair: Single<&mut CrosshairState>,
	mut time: ResMut<Time<Virtual>>,
	mut blocks_input: ResMut<BlocksInput>,
) {
	commands.spawn((
		DespawnOnExit(Menu::Pause),
		RootWidget,
		GlobalZIndex(3),
		widget::header("Game paused!"),
	));
	commands.spawn((
		DespawnOnExit(Menu::Pause),
		RootWidget,
		GlobalZIndex(3),
		widget::button("Unpause", close_pause_menu),
	));
	commands.spawn((
		DespawnOnExit(Menu::Pause),
		RootWidget,
		GlobalZIndex(3),
		widget::button("Settings", open_settings_menu),
	));
	commands.spawn((
		DespawnOnExit(Menu::Pause),
		RootWidget,
		GlobalZIndex(3),
		widget::button("Quit to title", quit_to_title),
	));
	crosshair
		.wants_free_cursor
		.insert(spawn_pause_menu.type_id());
	blocks_input.insert(spawn_pause_menu.type_id());
	time.pause();
}

fn open_settings_menu(_on: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::Settings);
}

fn close_pause_menu(
	_on: On<Pointer<Click>>,
	mut next_menu: ResMut<NextState<Menu>>,
	crosshair: Single<&mut CrosshairState>,
	time: ResMut<Time<Virtual>>,
	blocks_input: ResMut<BlocksInput>,
) {
	println!("UNPAUSE");
	next_menu.set(Menu::None);
	unpause(crosshair, time, blocks_input);
}

fn quit_to_title(
	_on: On<Pointer<Click>>,
	mut next_screen: ResMut<NextState<Screen>>,
	crosshair: Single<&mut CrosshairState>,
	time: ResMut<Time<Virtual>>,
	blocks_input: ResMut<BlocksInput>,
) {
	println!("QUIT");
	next_screen.set(Screen::Title);
	unpause(crosshair, time, blocks_input);
}

fn go_back(
	mut next_menu: ResMut<NextState<Menu>>,
	crosshair: Single<&mut CrosshairState>,
	time: ResMut<Time<Virtual>>,
	blocks_input: ResMut<BlocksInput>,
) {
	println!("go_back");
	next_menu.set(Menu::None);
	unpause(crosshair, time, blocks_input);
}

fn unpause(
	mut crosshair: Single<&mut CrosshairState>,
	mut time: ResMut<Time<Virtual>>,
	mut blocks_input: ResMut<BlocksInput>,
) {
	crosshair
		.wants_free_cursor
		.remove(&spawn_pause_menu.type_id());
	blocks_input.remove(&spawn_pause_menu.type_id());
	time.unpause();
}

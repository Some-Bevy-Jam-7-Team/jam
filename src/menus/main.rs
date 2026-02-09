//! The main menu (seen on the title screen).
use bevy::{
	prelude::*,
	window::{CursorGrabMode, CursorOptions},
};

use crate::{menus::Menu, theme::widget};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(OnEnter(Menu::Main), spawn_main_menu);
}

fn spawn_main_menu(mut commands: Commands, mut cursor_options: Single<&mut CursorOptions>) {
	cursor_options.grab_mode = CursorGrabMode::None;
	commands.spawn((
		DespawnOnExit(Menu::Main),
		crate::ui_layout::RootWidget,
		widget::button("Play", open_level_select),
	));
	commands.spawn((
		DespawnOnExit(Menu::Main),
		crate::ui_layout::RootWidget,
		widget::button("Settings", open_settings_menu),
	));
	commands.spawn((
		DespawnOnExit(Menu::Main),
		crate::ui_layout::RootWidget,
		widget::button("Credits", open_credits_menu),
	));
	#[cfg(not(target_family = "wasm"))]
	commands.spawn((
		DespawnOnExit(Menu::Main),
		crate::ui_layout::RootWidget,
		widget::button("Exit", exit_app),
	));
}

fn open_level_select(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::LevelSelect);
}

fn open_settings_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::Settings);
}

fn open_credits_menu(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::Credits);
}

#[cfg(not(target_family = "wasm"))]
fn exit_app(_: On<Pointer<Click>>, mut app_exit: MessageWriter<AppExit>) {
	app_exit.write(AppExit::Success);
}

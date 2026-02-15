//! The title screen that appears after the splash screen.

use bevy::prelude::*;
use bevy_seedling::sample::SamplePlayer;
use firewheel::Volume;

use crate::{audio::MusicPool, menus::Menu, screens::Screen};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(OnEnter(Screen::Title), (open_main_menu, spawn_gloop));
	app.add_systems(OnExit(Screen::Title), close_menu);
}

fn open_main_menu(mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::Main);
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::None);
}

fn spawn_gloop(mut commands: Commands, assets: Res<AssetServer>) {
	commands.spawn((
		DespawnOnEnter(Screen::Gameplay),
		SamplePlayer::new(assets.load("audio/music/gloopy.ogg"))
			.looping()
			.with_volume(Volume::Decibels(6.0)),
		MusicPool,
	));
}

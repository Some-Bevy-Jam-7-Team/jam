//! A credits menu.

use crate::ui_layout::RootWidget;
use crate::{menus::Menu, theme::prelude::*};
use bevy::{
	ecs::spawn::SpawnIter, input::common_conditions::input_just_pressed, prelude::*, ui::Val::*,
};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu);
	app.add_systems(
		Update,
		go_back.run_if(in_state(Menu::Credits).and(input_just_pressed(KeyCode::Escape))),
	);
}

fn spawn_credits_menu(mut commands: Commands) {
	commands.spawn((
		RootWidget,
		DespawnOnExit(Menu::Credits),
		GlobalZIndex(2),
		children![
			widget::header("Created by"),
			created_by(),
			widget::header("Assets"),
			assets(),
			widget::button("Back", go_back_on_click),
		],
	));
}

fn created_by() -> impl Bundle {
	grid(vec![
		["Joe Shmoe", "Implemented alligator wrestling AI"],
		["Jane Doe", "Made the music for the alien invasion"],
	])
}

fn assets() -> impl Bundle {
	grid(vec![
		[
			"Bevy logo",
			"All rights reserved by the Bevy Foundation, permission granted for splash screen use when unmodified",
		],
		["Button SFX", "CC0 by Jaszunio15"],
		["Ambient music and Footstep SFX", "CC0 by NOX SOUND"],
		[
			"Throw SFX",
			"FilmCow Royalty Free SFX Library License Agreement by Jason Steele",
		],
		[
			"Fox model",
			"CC0 1.0 Universal by PixelMannen (model), CC BY 4.0 International by tomkranis (Rigging & Animation), CC BY 4.0 International by AsoboStudio and scurest (Conversion to glTF)",
		],
		[
			"Player model",
			"You can use it commercially without the need to credit me by Drillimpact",
		],
		["Vocals", "CC BY 4.0 by Dillon Becker"],
		["Night Sky HDRI 001", "CC0 by ambientCG"],
		[
			"Dark Mod assets",
			"CC BY-NC-SA 3.0 by The Dark Mod Team, converted to Bevy-friendly assets by Jan Hohenheim",
		],
		[
			"Rock",
			"CC0 Rock Moss Set 01 by Kless Gyzen https://polyhaven.com/a/rock_moss_set_01",
		],
		["Fluorescent Light 1", "CC0 by EverydaySounds"],
		["Fluorescent Light 2", "CC0 by kyles"],
		["Floppy Disk", "CC0 by BigSoundBank"],
		["Door sounds", "CC0 by BigSoundBank"],
		["More stuffs", "TODO :)"],
	])
}

fn grid(content: Vec<[&'static str; 2]>) -> impl Bundle {
	(
		Name::new("Grid"),
		Node {
			display: Display::Grid,
			row_gap: Px(10.0),
			column_gap: Px(30.0),
			grid_template_columns: RepeatedGridTrack::px(2, 400.0),
			..default()
		},
		Children::spawn(SpawnIter(content.into_iter().flatten().enumerate().map(
			|(i, text)| {
				(
					widget::label_small(text),
					Node {
						justify_self: if i % 2 == 0 {
							JustifySelf::End
						} else {
							JustifySelf::Start
						},
						..default()
					},
				)
			},
		))),
	)
}

fn go_back_on_click(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::Main);
}

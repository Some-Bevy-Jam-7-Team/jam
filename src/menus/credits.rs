//! A credits menu.

use crate::audio::MusicPool;
use crate::ui_layout::RootWidget;
use crate::{menus::Menu, theme::prelude::*};
use bevy::{
	ecs::spawn::SpawnIter, input::common_conditions::input_just_pressed, prelude::*, ui::Val::*,
};
use bevy_seedling::sample::SamplePlayer;

pub(super) fn plugin(app: &mut App) {
	app.add_systems(OnEnter(Menu::Credits), spawn_credits_menu);
	app.add_systems(
		Update,
		go_back.run_if(in_state(Menu::Credits).and(input_just_pressed(KeyCode::Escape))),
	);
}

fn spawn_credits_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
	commands.spawn((
		MusicPool,
		SamplePlayer::new(asset_server.load("audio/music/sexy horse.ogg")),
		DespawnOnExit(Menu::Credits),
	));

	commands.spawn((
		DespawnOnExit(Menu::Credits),
		GlobalZIndex(2),
		Node {
			position_type: PositionType::Absolute,
			width: Percent(100.0),
			height: Percent(100.0),
			justify_content: JustifyContent::Center,
			align_items: AlignItems::Center,
			..default()
		},
		children![
			(
				Node {
					width: Vw(50.0),
					flex_direction: FlexDirection::Column,
					row_gap: Px(10.0),
					justify_content: JustifyContent::Center,
					align_items: AlignItems::Center,
					padding: UiRect::all(Px(15.0)),
					..default()
				},
				children![
					widget::header("Crafted by"),
					created_by(),
					widget::header("Assets"),
					assets()
				]
			),
			(
				Node {
					width: Vw(50.0),
					flex_direction: FlexDirection::Column,
					row_gap: Px(10.0),
					justify_content: JustifyContent::Start,
					align_items: AlignItems::Center,
					padding: UiRect::all(Px(15.0)),
					..default()
				},
				children![widget::header("Voice Acting"), voice_acting()]
			)
		],
	));

	commands.spawn((
		RootWidget,
		DespawnOnExit(Menu::Credits),
		GlobalZIndex(2),
		children![widget::button("Back", go_back_on_click)],
	));
}

fn created_by() -> impl Bundle {
	grid(vec![
		[
			"Jan Hohenheim",
			"project mischief, writing, game design, level design, dev-tooling, level design, project oiling, texturing, modelling, asset management, moral support, voice acting, cool guy",
		],
		[
			"IQuick",
			"fever dream haver, joke production, game design, gameplay code, project oiling, dev-tooling, the harbringer of Next Generation UI and Reactivity, writing, level design, credits writing, \"cool guy\"",
		],
		[
			"Freyja Moth",
			"level design, writing, voice acting, objective programming, the (metaphorical) mother of David the Anarcho Sydney guy, cool girl(?)",
		],
		[
			"Nico",
			"gameplay code, shaders, polish, grass and shroom gardener, bug-hunting, project oiling, asset management, cool guy",
		],
		[
			"Joe",
			"SFX programming, asset management, bug-hunting, cool guy",
		],
		[
			"vero",
			"music, voice acting, game design, shaders, rendering expert, parkour expert, 100_000x dev, -100_000x dev, cool lizard",
		],
		[
			"Joona Aalto",
			"physics, shaders, UI, voice acting, gloop reductor, door hinge expert, bug-hunting, cool guy",
		],
		[
			"Corvus Prudens",
			"music, voice acting, audio engineer, asset management, cool crow",
		],
		[
			"cereal",
			"music, voice acting, amoral support, cool corvidae",
		],
		["burningEmber", "voice acting, cool gal"],
		[
			"IWonderWhatThisAPIDoes",
			"modelling, texturing, voice acting, bug-hunting, cool guy",
		],
		["Dylan", "Broke his leg, we wish him well, cool guy"],
		[
			"willow, forest, ebony, and meadow",
			"emotional support cats",
		],
		[
			"Billie, Ratlas, Ratilla, Ratrick, and Bifidus",
			"emotional support rats",
		],
	])
}

fn voice_acting() -> impl Bundle {
	grid(vec![
		["LLManager", "Corvus Prudens"],
		["Jan", "Joe"],
		["Jannet", "Jan Hohenheim"],
		["Jannick", "Jan Hohenheim"],
		["Janitor", "Joona Aalto"],
		["Janissary", "Joona Aalto"],
		["Janibal Lectern", "Joona Aalto"],
		["Janderstülz", "IWonderWhatThisAPIDoes"],
		["Janthorpe", "burningEmber"],
		["Janniri", "Joona Aalto"],
		["Jandice", "IWonderWhatThisAPIDoes"],
		["Janateer", "Joona Aalto"],
		["John", "Joona Aalto"],
		["CCTV", "Joona Aalto"],
		["Rowdy Teen", "burningEmber"],
		["Glossy [FIGURE]", "Jan Hohenheim"],
		["Mind the Gap", "cereal"],
		["Posh Lady", "burningEmber"],
		["Geezer 1", "burningEmber"],
		["Geezer 2", "burningEmber"],
		["Salivatorus Dhalee", "cereal"],
		["Cultist", "Joona Aalto"],
		["Novice", "Joona Aalto"],
		["The Library", "Jan Hohenheim"],
		["Jeff", "Jan Hohenheim"],
		["Mark", "Jan Hohenheim"],
		["Jimmy", "Jan Hohenheim"],
		["Mushroom Lidya", "Freyja Moth"],
		["Dave", "Jan Hohenheim"],
		["Storm", "Jan Hohenheim"],
		["The Mouth", "Jan Hohenheim and vero"],
		["Jumping Sounds", "vero"],
		["Stomach Sounds", "vero"],
		["Eating and Vomiting Sounds", "vero"],
		["Generator Interaction Sounds", "Joe"],
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
		["Night Sky HDRI 001", "CC0 by ambientCG"],
		[
			"Dark Mod assets",
			"CC BY-NC-SA 3.0 by The Dark Mod Team, converted to Bevy-friendly assets by Jan Hohenheim",
		],
		["Rock", "CC0 Rock Moss Set 01 by Kless Gyzen"],
		[
			"CRT Monitor",
			"CC BY 4.0 by Ewan Lejkowski, simplified by Joona Aalto",
		],
		[
			"Computer Keyboard",
			"CC BY 4.0 by MelGibzon, simplified by Joona Aalto",
		],
		[
			"Computer Mouse",
			"CC BY 4.0 by anila.shakya, simplified by Joona Aalto",
		],
		["Fluorescent Light 1", "CC0 by EverydaySounds"],
		["Fluorescent Light 2", "CC0 by kyles"],
		["Floppy Disk", "CC0 by BigSoundBank"],
		["Door sounds", "CC0 by BigSoundBank"],
	])
}

fn grid(content: Vec<[&'static str; 2]>) -> impl Bundle {
	(
		Name::new("Grid"),
		Node {
			display: Display::Grid,
			row_gap: Px(6.0),
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

//! Cruelty Squad-inspired level selection screen.
//!
//! Each visual section is its own spawn function so other devs
//! can pull individual pieces into different contexts.

use bevy::{
	input::common_conditions::input_just_pressed,
	prelude::*,
	ui::Val::*,
	window::{CursorGrabMode, CursorOptions},
};

use crate::{
	menus::Menu,
	screens::Screen,
	theme::{interaction::InteractionPalette, palette::SCREEN_BACKGROUND, widget},
};

const NEON_GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
const GRID_BG: Color = Color::srgb(0.12, 0.12, 0.14);
const LOCKED_BG: Color = Color::srgb(0.15, 0.15, 0.15);
const LOCKED_TEXT: Color = Color::srgb(0.4, 0.4, 0.4);
const PANEL_BORDER: Color = Color::srgb(0.8, 0.0, 0.0);
const OBJECTIVE_BG: Color = Color::srgb(0.05, 0.1, 0.05);
const LORE_BG: Color = Color::srgb(0.06, 0.04, 0.08);
const HEADER_GREEN: Color = Color::srgb(0.0, 1.0, 0.0);
const ENTER_BG: Color = Color::srgb(0.0, 0.5, 0.0);
const ENTER_HOVER: Color = Color::srgb(0.0, 0.7, 0.0);
const ENTER_PRESS: Color = Color::srgb(0.0, 0.3, 0.0);

// --- Level data ---

struct LevelInfo {
	name: &'static str,
	objective: &'static str,
	description: &'static str,
	preview_color: Color,
	locked: bool,
}

const LEVELS: &[LevelInfo] = &[
	LevelInfo {
		name: "LEVEL 1",
		objective: "Placeholder objective for level 1",
		description: "Placeholder description for level 1. This text will be replaced with actual level details.",
		preview_color: Color::srgb(0.6, 0.2, 0.2),
		locked: false,
	},
	LevelInfo {
		name: "LEVEL 2",
		objective: "Placeholder objective for level 2",
		description: "Placeholder description for level 2. This text will be replaced with actual level details.",
		preview_color: Color::srgb(0.7, 0.3, 0.1),
		locked: true,
	},
	LevelInfo {
		name: "LEVEL 3",
		objective: "Placeholder objective for level 3",
		description: "Placeholder description for level 3. This text will be replaced with actual level details.",
		preview_color: Color::srgb(0.2, 0.4, 0.7),
		locked: true,
	},
	LevelInfo {
		name: "LEVEL 4",
		objective: "Placeholder objective for level 4",
		description: "Placeholder description for level 4. This text will be replaced with actual level details.",
		preview_color: Color::srgb(0.3, 0.5, 0.2),
		locked: true,
	},
	LevelInfo {
		name: "LEVEL 5",
		objective: "Placeholder objective for level 5",
		description: "Placeholder description for level 5. This text will be replaced with actual level details.",
		preview_color: Color::srgb(0.8, 0.7, 0.2),
		locked: true,
	},
	LevelInfo {
		name: "LEVEL 6",
		objective: "Placeholder objective for level 6",
		description: "Placeholder description for level 6. This text will be replaced with actual level details.",
		preview_color: Color::srgb(0.1, 0.6, 0.6),
		locked: true,
	},
];

// --- Components ---

#[derive(Component)]
struct LevelSquare(usize);

#[derive(Component)]
struct PreviewPanel;

#[derive(Component)]
struct DescriptionText;

#[derive(Component)]
struct ObjectiveText;

#[derive(Component)]
struct LevelNameText;

#[derive(Component)]
struct EnterLevelButton;

#[derive(Component)]
struct SelectedBorder(usize);

// --- Resources ---

#[derive(Resource, Default)]
struct SelectedLevel(usize);

// --- Plugin ---

pub(super) fn plugin(app: &mut App) {
	app.add_systems(OnEnter(Menu::LevelSelect), spawn_level_select);
	app.add_systems(OnExit(Menu::LevelSelect), cleanup_selected_level);
	app.add_systems(
		Update,
		(
			handle_level_click,
			update_selection_visuals,
			handle_enter_level,
		)
			.run_if(in_state(Menu::LevelSelect)),
	);
	app.add_systems(
		Update,
		go_back.run_if(in_state(Menu::LevelSelect).and(input_just_pressed(KeyCode::Escape))),
	);
}

fn cleanup_selected_level(mut commands: Commands) {
	commands.remove_resource::<SelectedLevel>();
}

/// Top-level level-select screen. Composes all sub-panels.
fn spawn_level_select(mut commands: Commands) {
	commands.init_resource::<SelectedLevel>();
	let level = &LEVELS[0];

	commands
		.spawn((
			widget::ui_root("Level Select"),
			BackgroundColor(SCREEN_BACKGROUND),
			GlobalZIndex(2),
			DespawnOnExit(Menu::LevelSelect),
		))
		.with_children(|root| {
			spawn_header(root);

			// Main 3-column layout
			root.spawn((
				Name::new("Main Layout"),
				Node {
					width: Percent(95.0),
					height: Percent(75.0),
					flex_direction: FlexDirection::Row,
					column_gap: Px(15.0),
					..default()
				},
			))
			.with_children(|columns| {
				spawn_level_grid(columns);
				spawn_preview_panel(columns, level);
				spawn_right_column(columns, level);
			});

			spawn_bottom_buttons(root);
		});
}

/// `<<LEVEL SELECT>>` header text.
fn spawn_header(parent: &mut ChildSpawnerCommands) {
	parent.spawn((
		Name::new("Level Select Header"),
		Text("<<LEVEL SELECT>>".into()),
		TextFont::from_font_size(36.0),
		TextColor(HEADER_GREEN),
		Node {
			margin: UiRect {
				left: Px(15.0),
				bottom: Px(5.0),
				..default()
			},
			..default()
		},
	));
}

/// 2x3 CSS-Grid of level squares (left column).
fn spawn_level_grid(parent: &mut ChildSpawnerCommands) {
	parent
		.spawn((
			Name::new("Level Grid"),
			Node {
				display: Display::Grid,
				width: Px(220.0),
				padding: UiRect::all(Px(10.0)),
				row_gap: Px(8.0),
				column_gap: Px(8.0),
				grid_template_columns: RepeatedGridTrack::px(2, 95.0),
				grid_template_rows: RepeatedGridTrack::px(3, 70.0),
				..default()
			},
			BackgroundColor(GRID_BG),
			BorderColor::from(Color::srgb(0.3, 0.3, 0.0)),
		))
		.with_children(|grid| {
			for idx in 0..LEVELS.len() {
				spawn_level_square(grid, idx);
			}
		});
}

/// Single clickable level square inside the grid.
fn spawn_level_square(parent: &mut ChildSpawnerCommands, idx: usize) {
	let level = &LEVELS[idx];
	let is_selected = idx == 0;
	let bg = if level.locked {
		LOCKED_BG
	} else {
		level.preview_color
	};

	parent
		.spawn((
			Name::new(format!("Level Square {}", idx)),
			Button,
			LevelSquare(idx),
			SelectedBorder(idx),
			BackgroundColor(bg),
			Node {
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				border: UiRect::all(Px(3.0)),
				..default()
			},
			BorderColor::from(if is_selected {
				NEON_GREEN
			} else {
				Color::srgb(0.2, 0.2, 0.2)
			}),
		))
		.with_children(|sq| {
			let text = if level.locked {
				"???".to_string()
			} else {
				format!("L{}", idx + 1)
			};
			let color = if level.locked {
				LOCKED_TEXT
			} else {
				Color::WHITE
			};
			sq.spawn((
				Text(text),
				TextFont::from_font_size(20.0),
				TextColor(color),
				Pickable::IGNORE,
			));
		});
}

/// Center column: level name, preview rect, objective box.
fn spawn_preview_panel(parent: &mut ChildSpawnerCommands, level: &LevelInfo) {
	parent
		.spawn((
			Name::new("Center Column"),
			Node {
				flex_grow: 1.0,
				flex_direction: FlexDirection::Column,
				row_gap: Px(10.0),
				..default()
			},
		))
		.with_children(|center| {
			// Level name header
			center.spawn((
				Name::new("Level Name"),
				LevelNameText,
				Text(level.name.into()),
				TextFont::from_font_size(28.0),
				TextColor(NEON_GREEN),
				Node {
					margin: UiRect::left(Px(5.0)),
					..default()
				},
			));

			// Preview rectangle
			center.spawn((
				Name::new("Preview Panel"),
				PreviewPanel,
				BackgroundColor(level.preview_color),
				Node {
					width: Percent(100.0),
					height: Px(250.0),
					border: UiRect::all(Px(3.0)),
					..default()
				},
				BorderColor::from(PANEL_BORDER),
			));

			spawn_objective_box(center, level);
		});
}

/// Green-bordered objective text box.
fn spawn_objective_box(parent: &mut ChildSpawnerCommands, level: &LevelInfo) {
	parent
		.spawn((
			Name::new("Objective Box"),
			BackgroundColor(OBJECTIVE_BG),
			Node {
				width: Percent(100.0),
				padding: UiRect::all(Px(12.0)),
				border: UiRect::all(Px(2.0)),
				margin: UiRect::top(Px(-3.0)),
				..default()
			},
			BorderColor::from(NEON_GREEN),
		))
		.with_children(|obj| {
			obj.spawn((
				Name::new("Objective Label"),
				Text(format!("OBJECTIVE: {}", level.objective)),
				TextFont::from_font_size(16.0),
				TextColor(NEON_GREEN),
				ObjectiveText,
			));
		});
}

/// Right column: portrait placeholder + description/lore panel.
fn spawn_right_column(parent: &mut ChildSpawnerCommands, level: &LevelInfo) {
	parent
		.spawn((
			Name::new("Right Column"),
			Node {
				width: Px(200.0),
				flex_direction: FlexDirection::Column,
				row_gap: Px(8.0),
				..default()
			},
		))
		.with_children(|right| {
			spawn_portrait(right);
			spawn_description_panel(right, level);
		});
}

/// Portrait box (placeholder — wire up eye material or an image here).
fn spawn_portrait(parent: &mut ChildSpawnerCommands) {
	parent.spawn((
		Name::new("Portrait"),
		BackgroundColor(Color::srgb(0.1, 0.08, 0.12)),
		Node {
			width: Px(180.0),
			height: Px(120.0),
			border: UiRect::all(Px(3.0)),
			margin: UiRect {
				left: Px(5.0),
				top: Px(20.0),
				..default()
			},
			..default()
		},
		BorderColor::from(PANEL_BORDER),
	));
}

/// Description/lore text panel beneath the portrait.
fn spawn_description_panel(parent: &mut ChildSpawnerCommands, level: &LevelInfo) {
	parent
		.spawn((
			Name::new("Description Panel"),
			BackgroundColor(LORE_BG),
			Node {
				width: Px(190.0),
				min_height: Px(150.0),
				padding: UiRect::all(Px(10.0)),
				border: UiRect {
					left: Px(2.0),
					right: Px(2.0),
					top: Px(0.0),
					bottom: Px(2.0),
				},
				margin: UiRect::left(Px(5.0)),
				..default()
			},
			BorderColor::from(Color::srgb(0.4, 0.2, 0.5)),
		))
		.with_children(|lore_panel| {
			lore_panel.spawn((
				Name::new("Description Text"),
				DescriptionText,
				Text(level.description.into()),
				TextFont::from_font_size(13.0),
				TextColor(Color::srgb(0.7, 0.6, 0.8)),
			));
		});
}

/// Bottom row: Enter Level + Back buttons.
fn spawn_bottom_buttons(parent: &mut ChildSpawnerCommands) {
	parent
		.spawn((
			Name::new("Bottom Buttons"),
			Node {
				width: Percent(95.0),
				flex_direction: FlexDirection::Row,
				justify_content: JustifyContent::SpaceBetween,
				margin: UiRect::top(Px(10.0)),
				..default()
			},
		))
		.with_children(|bottom| {
			spawn_enter_button(bottom);
			spawn_back_button(bottom);
		});
}

/// Green `<<ENTER LEVEL>>` button.
fn spawn_enter_button(parent: &mut ChildSpawnerCommands) {
	parent
		.spawn((
			Name::new("Enter Level Button"),
			Button,
			EnterLevelButton,
			BackgroundColor(ENTER_BG),
			InteractionPalette {
				none: ENTER_BG,
				hovered: ENTER_HOVER,
				pressed: ENTER_PRESS,
			},
			Node {
				width: Px(250.0),
				height: Px(50.0),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				border: UiRect::all(Px(2.0)),
				..default()
			},
			BorderColor::from(NEON_GREEN),
		))
		.with_children(|btn| {
			btn.spawn((
				Text("<<ENTER LEVEL>>".into()),
				TextFont::from_font_size(24.0),
				TextColor(NEON_GREEN),
				Pickable::IGNORE,
			));
		});
}

/// Red `<<BACK>>` button.
fn spawn_back_button(parent: &mut ChildSpawnerCommands) {
	parent
		.spawn((
			Name::new("Back Button"),
			Button,
			BackgroundColor(Color::srgb(0.5, 0.1, 0.1)),
			InteractionPalette {
				none: Color::srgb(0.5, 0.1, 0.1),
				hovered: Color::srgb(0.7, 0.15, 0.15),
				pressed: Color::srgb(0.3, 0.05, 0.05),
			},
			Node {
				width: Px(200.0),
				height: Px(50.0),
				align_items: AlignItems::Center,
				justify_content: JustifyContent::Center,
				border: UiRect::all(Px(2.0)),
				margin: UiRect::right(Px(7.0)),
				..default()
			},
			BorderColor::from(Color::srgb(0.8, 0.2, 0.2)),
		))
		.observe(go_back_on_click)
		.with_children(|btn| {
			btn.spawn((
				Text("<<BACK>>".into()),
				TextFont::from_font_size(24.0),
				TextColor(Color::srgb(0.9, 0.9, 0.9)),
				Pickable::IGNORE,
			));
		});
}

// --- Systems ---

fn handle_level_click(
	interaction_query: Query<(&Interaction, &LevelSquare), Changed<Interaction>>,
	mut selected: ResMut<SelectedLevel>,
) {
	for (interaction, square) in &interaction_query {
		if *interaction == Interaction::Pressed {
			selected.0 = square.0;
		}
	}
}

fn update_selection_visuals(
	selected: Res<SelectedLevel>,
	mut border_query: Query<
		(&SelectedBorder, &mut BorderColor, &mut BackgroundColor),
		Without<PreviewPanel>,
	>,
	mut preview_query: Query<&mut BackgroundColor, With<PreviewPanel>>,
	mut name_query: Query<
		&mut Text,
		(
			With<LevelNameText>,
			Without<ObjectiveText>,
			Without<DescriptionText>,
		),
	>,
	mut objective_query: Query<
		&mut Text,
		(
			With<ObjectiveText>,
			Without<LevelNameText>,
			Without<DescriptionText>,
		),
	>,
	mut lore_query: Query<
		&mut Text,
		(
			With<DescriptionText>,
			Without<LevelNameText>,
			Without<ObjectiveText>,
		),
	>,
) {
	if !selected.is_changed() {
		return;
	}

	let idx = selected.0;
	let level = &LEVELS[idx];

	for (border, mut border_color, mut bg_color) in &mut border_query {
		if border.0 == idx {
			*border_color = NEON_GREEN.into();
		} else {
			*border_color = Color::srgb(0.2, 0.2, 0.2).into();
		}
		let lev = &LEVELS[border.0];
		if lev.locked {
			*bg_color = LOCKED_BG.into();
		} else {
			*bg_color = lev.preview_color.into();
		}
	}

	for mut bg in &mut preview_query {
		if level.locked {
			*bg = LOCKED_BG.into();
		} else {
			*bg = level.preview_color.into();
		}
	}

	for mut text in &mut name_query {
		if level.locked {
			text.0 = "???".into();
		} else {
			text.0 = level.name.into();
		}
	}

	for mut text in &mut objective_query {
		if level.locked {
			text.0 = "OBJECTIVE: ???".into();
		} else {
			text.0 = format!("OBJECTIVE: {}", level.objective);
		}
	}

	for mut text in &mut lore_query {
		if level.locked {
			text.0 = "Locked level. Complete previous levels to unlock.".into();
		} else {
			text.0 = level.description.into();
		}
	}
}

fn handle_enter_level(
	interaction_query: Query<&Interaction, (Changed<Interaction>, With<EnterLevelButton>)>,
	selected: Res<SelectedLevel>,
	mut next_screen: ResMut<NextState<Screen>>,
	mut next_menu: ResMut<NextState<Menu>>,
	mut cursor_options: Single<&mut CursorOptions>,
) {
	for interaction in &interaction_query {
		if *interaction == Interaction::Pressed {
			let level = &LEVELS[selected.0];
			if !level.locked {
				next_screen.set(Screen::Loading);
				next_menu.set(Menu::None);
				cursor_options.grab_mode = CursorGrabMode::Locked;
			}
		}
	}
}

fn go_back_on_click(_: On<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::Main);
}

fn go_back(mut next_menu: ResMut<NextState<Menu>>) {
	next_menu.set(Menu::Main);
}

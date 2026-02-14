use crate::theme::palette::SCREEN_BACKGROUND;
use crate::theme::textures::{BUTTON_TEXTURE, TexturedUiMaterial};
use crate::ui_layout::RootWidget;

use super::assets::image_handle;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
use bevy_yarnspinner::prelude::*;

pub(super) fn ui_setup_plugin(app: &mut App) {
	app.add_systems(Startup, setup);
}

/// Marker for the [`Node`] that is the root of the UI
#[derive(Debug, Default, Component)]
pub struct UiRootNode;

#[derive(Debug, Default, Component)]
pub(super) struct DialogueNode;

#[derive(Debug, Default, Component)]
pub(super) struct DialogueNameNode;

#[derive(Debug, Default, Component)]
pub(super) struct DialogueContinueNode;

#[derive(Debug, Default, Component)]
pub(super) struct OptionsNode;

#[derive(Debug, Component)]
pub(super) struct OptionButton(pub OptionId);

fn setup(
	mut commands: Commands,
	mut materials: ResMut<Assets<TexturedUiMaterial>>,
	asset_server: Res<AssetServer>,
) {
	// root node
	commands
		.spawn((
			fmt_name("root"),
			Node {
				display: Display::Grid,
				// width: Val::Percent(100.0),
				// height: Val::Percent(100.0),
				align_content: AlignContent::End,
				justify_content: JustifyContent::SpaceAround,
				grid_auto_flow: GridAutoFlow::Row,
				grid_template_columns: vec![RepeatedGridTrack::minmax(
					1,
					MinTrackSizingFunction::Auto,
					MaxTrackSizingFunction::Px(DIALOG_WIDTH),
				)],
				..default()
			},
			GlobalZIndex(1),
			Visibility::Hidden,
			RootWidget,
			UiRootNode,
		))
		.with_children(|parent| {
			parent
				.spawn((
					fmt_name("dialogue"),
					Node {
						flex_direction: FlexDirection::Column,
						justify_content: JustifyContent::SpaceAround,
						align_items: AlignItems::FlexStart,
						padding: UiRect {
							top: Val::Px(TEXT_BORDER_TOP),
							bottom: Val::Px(TEXT_BORDER_BOTTOM),
							left: Val::Px(TEXT_BORDER_HORIZONTAL),
							right: Val::Px(TEXT_BORDER_HORIZONTAL),
						},
						border_radius: BorderRadius::all(Val::Px(10.0)),
						..default()
					},
					MaterialNode(materials.add(TexturedUiMaterial::new(
						Color::hsl(195.0, 0.2, 0.1),
						asset_server.load("textures/carpet/carpet_ambientOcclusion.png"),
						0.01,
					))),
				))
				.with_children(|parent| {
					// Speaker name
					parent.spawn((
						fmt_name("name"),
						Text::default(),
						text_style::name(),
						Node {
							margin: UiRect::bottom(Val::Px(10.0)),
							..default()
						},
						DialogueNameNode,
						Label,
					));

					// Dialog itself
					parent.spawn((
						fmt_name("text"),
						Text::default(),
						text_style::standard(),
						style::standard(),
						DialogueNode,
						Label,
					));

					// Options
					parent.spawn((
						fmt_name("options"),
						Node {
							display: Display::None,
							flex_direction: FlexDirection::Column,
							justify_content: JustifyContent::FlexEnd,
							align_items: AlignItems::FlexStart,
							margin: UiRect::top(Val::Px(20.0)),
							..default()
						},
						Visibility::Hidden,
						OptionsNode,
					));
				});

			parent.spawn((
				fmt_name("continue indicator"),
				ImageNode {
					// 27 x 27 pixels
					image: image_handle::CONTINUE_INDICATOR,
					..default()
				},
				Node {
					justify_self: JustifySelf::Center,
					align_self: AlignSelf::Center,
					margin: UiRect {
						top: Val::Px(-18.),
						bottom: Val::Px(25.),
						..default()
					},
					..default()
				},
				ZIndex(1),
				Visibility::Hidden,
				DialogueContinueNode,
			));
		});
}

fn fmt_name(name: &str) -> Name {
	Name::new(format!("Yarn Spinner example dialogue view node: {name}"))
}

pub(super) fn create_dialog_text(
	text: impl Into<String>,
	invisible: impl Into<String>,
) -> [(TextSpan, TextFont, TextColor); 2] {
	[
		(
			TextSpan(text.into()),
			text_style::standard().0,
			text_style::standard().1,
		),
		(
			TextSpan(invisible.into()),
			text_style::standard().0,
			TextColor(Color::NONE),
		),
	]
}

pub(super) fn spawn_options<'a, T>(entity_commands: &mut EntityCommands, options: T)
where
	T: IntoIterator<Item = &'a DialogueOption>,
	<T as IntoIterator>::IntoIter: 'a,
{
	entity_commands.with_children(|parent| {
		for (i, option) in options.into_iter().enumerate() {
			parent
				.spawn((
					fmt_name("option button"),
					Node {
						justify_content: JustifyContent::FlexStart,
						..default()
					},
				))
				.with_children(|parent| {
					parent
						.spawn((
							fmt_name("option text"),
							Button,
							Text::default(),
							style::options(),
							ImageNode::default().with_color(Color::NONE),
							OptionButton(option.id),
							Label,
						))
						.with_children(|parent| {
							parent
								.spawn((TextSpan(format!("{}: ", i + 1)), text_style::option_id()));
							parent.spawn((
								TextSpan(option.line.text.clone()),
								text_style::option_text(),
							));
						});
				});
		}
	});
}

const DIALOG_WIDTH: f32 = 600.0;
const TEXT_BORDER_HORIZONTAL: f32 = 60.0;
const TEXT_BORDER_TOP: f32 = 30.0;
const TEXT_BORDER_BOTTOM: f32 = TEXT_BORDER_TOP + 10.0;

mod style {
	use super::*;
	pub(super) fn standard() -> Node {
		Node {
			max_width: Val::Px(DIALOG_WIDTH - 2.0 * TEXT_BORDER_HORIZONTAL),
			..default()
		}
	}
	pub(super) fn options() -> Node {
		const INDENT_MODIFIER: f32 = 1.0;
		Node {
			margin: UiRect::horizontal(Val::Px((INDENT_MODIFIER - 1.0) * TEXT_BORDER_HORIZONTAL)),
			max_width: Val::Px(DIALOG_WIDTH - 2.0 * INDENT_MODIFIER * TEXT_BORDER_HORIZONTAL),
			..default()
		}
	}
}

mod text_style {
	use super::*;
	use crate::{
		font::{DEFAULT_FONT, VARIABLE_FONT},
		theme::palette::{HEADER_TEXT, LABEL_TEXT},
	};
	use bevy::color::palettes::css;

	pub(super) fn standard() -> (TextFont, TextColor) {
		(
			TextFont {
				font: DEFAULT_FONT,
				font_size: 20.0,
				..default()
			},
			TextColor(Color::WHITE),
		)
	}

	pub(super) fn name() -> (TextFont, TextColor) {
		(
			TextFont {
				font: VARIABLE_FONT,
				font_size: 22.0,
				weight: FontWeight(900),
				..standard().0
			},
			TextColor(Color::hsl(120.0, 1.0, 0.9)),
		)
	}

	pub(super) fn option_id() -> (TextFont, TextColor) {
		(
			TextFont {
				font: DEFAULT_FONT,
				..option_text().0
			},
			TextColor(css::ALICE_BLUE.into()),
		)
	}

	pub(super) fn option_text() -> (TextFont, TextColor) {
		(
			TextFont {
				font_size: 18.0,
				..standard().0
			},
			TextColor(LABEL_TEXT),
		)
	}
}

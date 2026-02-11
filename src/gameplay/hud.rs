use bevy::prelude::*;

use crate::{gameplay::interaction::AvailableInteraction, screens::Screen, ui_layout::RootWidget};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(OnEnter(Screen::Gameplay), spawn_interaction_text)
		.add_systems(Update, update_interaction_text);
}

/// Marker component for the [`Text`] node which displays the ability to interact
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
struct InteractionHintText;

fn spawn_interaction_text(mut commands: Commands) {
	commands.spawn((
		Text::new(""),
		TextFont::from_font_size(37.0),
		DespawnOnExit(Screen::Gameplay),
		InteractionHintText,
		RootWidget,
	));
}

fn update_interaction_text(
	mut interaction_text: Single<&mut Text, With<InteractionHintText>>,
	interaction_data: Res<AvailableInteraction>,
) {
	***interaction_text = interaction_data
		.description
		.as_ref()
		.map_or("".to_string(), |description| format!("Click:{description}"));
}

use bevy::prelude::*;

use crate::{
	gameplay::interaction::AvailableInteraction, props::interactables::InteractableEntity,
	screens::Screen, ui_layout::RootWidget,
};

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
	interactable: Query<&InteractableEntity>,
) {
	***interaction_text = interaction_data
		.target_entity
		.map_or("".to_string(), |entity| {
			interactable
				.get(entity)
				.ok()
				.map_or("Click: ???".to_string(), |interactable| {
					interactable
						.get_hover_text()
						.map_or("".to_string(), |value| format!("Click:{value}"))
				})
		});
}

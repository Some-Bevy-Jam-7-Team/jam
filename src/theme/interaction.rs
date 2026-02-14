use bevy::prelude::*;
use bevy_seedling::sample::{AudioSample, SamplePlayer};

use crate::{
	PostPhysicsAppSystems, asset_tracking::LoadResource, audio::SfxPool,
	theme::textures::ButtonMaterial,
};

pub(super) fn plugin(app: &mut App) {
	app.load_resource::<InteractionAssets>();
	app.add_systems(
		Update,
		(
			trigger_on_press,
			apply_interaction_palette,
			apply_button_interaction_palette,
			trigger_interaction_sound_effect,
		)
			.run_if(resource_exists::<InteractionAssets>)
			.in_set(PostPhysicsAppSystems::ChangeUi),
	);
}

/// Palette for widget interactions. Add this to an entity that supports
/// [`Interaction`]s, such as a button, to change its [`BackgroundColor`] based
/// on the current interaction state.
#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
pub(crate) struct InteractionPalette {
	pub(crate) none: Color,
	pub(crate) hovered: Color,
	pub(crate) pressed: Color,
}

/// Event triggered on a UI entity when the [`Interaction`] component on the same entity changes to
/// [`Interaction::Pressed`]. Observe this event to detect e.g. button presses.
#[derive(EntityEvent)]
pub(crate) struct OnPress {
	pub(crate) entity: Entity,
}

fn trigger_on_press(
	interaction_query: Query<(Entity, &Interaction), Changed<Interaction>>,
	mut commands: Commands,
) {
	for (entity, interaction) in &interaction_query {
		if matches!(interaction, Interaction::Pressed) {
			commands.trigger(OnPress { entity });
		}
	}
}

fn apply_interaction_palette(
	mut palette_query: Query<
		(&Interaction, &InteractionPalette, &mut BackgroundColor),
		Changed<Interaction>,
	>,
) {
	for (interaction, palette, mut background) in &mut palette_query {
		*background = match interaction {
			Interaction::None => palette.none,
			Interaction::Hovered => palette.hovered,
			Interaction::Pressed => palette.pressed,
		}
		.into();
	}
}

fn apply_button_interaction_palette(
	palette_query: Query<
		(
			&Interaction,
			&InteractionPalette,
			&MaterialNode<ButtonMaterial>,
		),
		Changed<Interaction>,
	>,
	mut materials: ResMut<Assets<ButtonMaterial>>,
) {
	for (interaction, palette, material_handle) in &palette_query {
		let material = materials.get_mut(&material_handle.0).unwrap();
		material.color = match interaction {
			Interaction::None => palette.none.to_srgba().to_vec4(),
			Interaction::Hovered => palette.hovered.to_srgba().to_vec4(),
			Interaction::Pressed => palette.pressed.to_srgba().to_vec4(),
		};
	}
}

#[derive(Resource, Asset, Reflect, Clone)]
pub(crate) struct InteractionAssets {
	#[dependency]
	hover: Handle<AudioSample>,
	#[dependency]
	press: Handle<AudioSample>,
}

impl InteractionAssets {
	pub(crate) const PATH_BUTTON_HOVER: &'static str = "audio/sound_effects/button_hover.ogg";
	pub(crate) const PATH_BUTTON_PRESS: &'static str = "audio/sound_effects/button_press.ogg";
}

impl FromWorld for InteractionAssets {
	fn from_world(world: &mut World) -> Self {
		let assets = world.resource::<AssetServer>();
		Self {
			hover: assets.load(Self::PATH_BUTTON_HOVER),
			press: assets.load(Self::PATH_BUTTON_PRESS),
		}
	}
}

fn trigger_interaction_sound_effect(
	interaction_query: Query<&Interaction, Changed<Interaction>>,
	interaction_assets: Res<InteractionAssets>,
	mut commands: Commands,
) {
	for interaction in &interaction_query {
		let source = match interaction {
			Interaction::Hovered => interaction_assets.hover.clone(),
			Interaction::Pressed => interaction_assets.press.clone(),
			_ => continue,
		};
		commands.spawn((SamplePlayer::new(source), SfxPool));
	}
}

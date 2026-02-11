//! Crosshair and cursor handling.
//! The crosshair is a UI element that is used to indicate the player's aim. We change the crosshair when the player is looking at a prop or an NPC.
//! This is done by registering which systems are interested in the crosshair state.

use crate::{PostPhysicsAppSystems, screens::Screen};
use assets::{CROSSHAIR_DOT_PATH, CROSSHAIR_SQUARE_PATH};
use bevy::{
	picking::input::MousePointerPositionOverride,
	platform::collections::HashSet,
	prelude::*,
	window::{CursorGrabMode, CursorOptions},
};

use std::any::{Any as _, TypeId};

pub(crate) mod assets;

pub(super) fn plugin(app: &mut App) {
	app.add_systems(
		Update,
		(
			update_pointer,
			update_crosshair.in_set(PostPhysicsAppSystems::ChangeUi),
		),
	);
	app.add_systems(OnEnter(Screen::Gameplay), spawn_crosshair);

	app.add_plugins(assets::plugin);
}

/// Show a crosshair for better aiming
fn spawn_crosshair(mut commands: Commands, assets: Res<AssetServer>) {
	commands
		.spawn((
			Name::new("Crosshair"),
			Node {
				width: Val::Percent(100.0),
				height: Val::Percent(100.0),
				justify_content: JustifyContent::Center,
				align_items: AlignItems::Center,
				..default()
			},
			DespawnOnExit(Screen::Gameplay),
			Pickable::IGNORE, // NEEDED TO UNBREAK UI
		))
		.with_children(|parent| {
			parent.spawn((
				Name::new("Crosshair Image"),
				CrosshairState::default(),
				ImageNode::new(assets.load(CROSSHAIR_DOT_PATH)),
				Pickable::IGNORE, // NEEDED TO UNBREAK UI
			));
		});
}

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component, Default)]
pub(crate) struct CrosshairState {
	pub(crate) wants_square: HashSet<TypeId>,
	pub(crate) wants_invisible: HashSet<TypeId>,
	pub(crate) wants_free_cursor: HashSet<TypeId>,
}

fn update_pointer(
	crosshair: Option<Single<&CrosshairState>>,
	window: Single<&Window>,
	mut mouse_override: ResMut<MousePointerPositionOverride>,
	mut cursor_options: Single<&mut CursorOptions>,
) {
	let crosshair_state = crosshair.as_deref();
	if crosshair_state.is_some_and(|a| a.wants_free_cursor.is_empty()) {
		cursor_options.grab_mode = CursorGrabMode::Locked;
		#[cfg(feature = "native")]
		{
			cursor_options.visible = false;
		}
		mouse_override.over_ride = Some(window.size() / 2.0);
	} else {
		cursor_options.grab_mode = CursorGrabMode::None;
		#[cfg(feature = "native")]
		{
			cursor_options.visible = true;
		}
		mouse_override.over_ride = None;
	}
}

fn update_crosshair(
	crosshair: Option<
		Single<(&mut CrosshairState, &mut ImageNode, &mut Visibility), Changed<CrosshairState>>,
	>,
	assets: Res<AssetServer>,
) {
	let Some((mut crosshair_state, mut image_node, mut visibility)) =
		crosshair.map(|c| c.into_inner())
	else {
		return;
	};
	if crosshair_state.wants_square.is_empty() {
		image_node.image = assets.load(CROSSHAIR_DOT_PATH);
	} else {
		image_node.image = assets.load(CROSSHAIR_SQUARE_PATH);
	}

	if crosshair_state.wants_free_cursor.is_empty() {
		crosshair_state
			.wants_invisible
			.remove(&update_crosshair.type_id());
	} else {
		crosshair_state
			.wants_invisible
			.insert(update_crosshair.type_id());
	}

	if crosshair_state.wants_invisible.is_empty() {
		*visibility = Visibility::Inherited;
	} else {
		*visibility = Visibility::Hidden;
	}
}

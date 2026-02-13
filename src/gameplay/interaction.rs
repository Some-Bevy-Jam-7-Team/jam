use crate::gameplay::TargetnameEntityIndex;
use crate::gameplay::player::camera::PlayerCameraParent;
use crate::gameplay::player::input::Interact;
use crate::props::interactables::InteractableEntity;
use crate::third_party::avian3d::CollisionLayer;
use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_enhanced_input::prelude::*;

/// [`Resource`] describing whether there is an interactable action available and optionally if there is a name for it.
#[derive(Resource, Default)]
pub struct AvailableInteraction {
	pub target_entity: Option<Entity>,
}

/// [`Event`] triggered when the specified entity was interacted with.
#[derive(Event)]
pub struct InteractEvent(pub Entity);

pub(super) fn plugin(app: &mut App) {
	app.init_resource::<AvailableInteraction>()
		.add_observer(interact_by_input_action);
	app.add_systems(Update, iquick_plz_do_not_kill_me);
}

fn interact_by_input_action(
	_trigger: On<Fire<Interact>>,
	resource: Res<AvailableInteraction>,
	entity_map: Option<Res<TargetnameEntityIndex>>,
	interaction_query: Query<&InteractableEntity>,
	mut commands: Commands,
) {
	if let Some(entity) = resource.target_entity {
		commands.trigger(InteractEvent(entity));

		// Also try shooting events to friends!
		if let (Some(entity_map), Ok(interactable)) = (entity_map, interaction_query.get(entity)) {
			if let Some(target) = interactable.get_interaction_relay() {
				for &related in entity_map.get_entity_by_targetname(target) {
					// Don't double-trigger
					if related != entity {
						commands.trigger(InteractEvent(related));
					}
				}
			}
		}
	}
}

fn iquick_plz_do_not_kill_me(
	spatial: SpatialQuery,
	cam: Single<&Transform, With<PlayerCameraParent>>,
	pickable: Query<(Entity, &Pickable)>,
	sensors: Query<Entity, With<Sensor>>,
	interaction_query: Query<&InteractableEntity>,
	mut resource: ResMut<AvailableInteraction>,
	collider: Query<&ColliderOf>,
) {
	let transform = cam.into_inner();
	let ignored = pickable
		.iter()
		.filter_map(|(entity, pickable)| {
			if *pickable == Pickable::IGNORE {
				Some(entity)
			} else {
				None
			}
		})
		.chain(sensors.iter());
	resource.target_entity = None;
	if let Some(hit) = spatial.cast_ray(
		transform.translation,
		transform.forward(),
		3.0,
		true,
		&SpatialQueryFilter::from_excluded_entities(ignored).with_mask([
			CollisionLayer::Default,
			CollisionLayer::Prop,
			CollisionLayer::Character,
			CollisionLayer::Dialog,
		]),
	) && let Ok(collider) = collider.get(hit.entity)
		&& let Ok(interaction) = interaction_query.get(collider.body)
		&& interaction.is_active()
	{
		resource.target_entity = Some(collider.body);
	}
}

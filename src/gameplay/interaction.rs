use crate::gameplay::player::camera::PlayerCameraParent;
use crate::gameplay::player::input::Interact;
use crate::third_party::avian3d::CollisionLayer;
use avian3d::prelude::{ColliderOf, PhysicsPickable, Sensor, SpatialQuery, SpatialQueryFilter};
use bevy::prelude::*;
use bevy::{ecs::component::Component, picking::Pickable};
use bevy_enhanced_input::prelude::Start;

/// Marker component for an entity being interactable by clicking on it.
#[derive(Component, Default, Reflect, Debug)]
#[reflect(Component)]
#[require(Pickable, PhysicsPickable)]
pub struct InteractableObject(pub Option<String>);

/// [`Resource`] describing whether there is an interactable action available and optionally if there is a name for it.
#[derive(Resource, Default)]
pub struct AvailableInteraction {
	pub target_entity: Option<Entity>,
	pub description: Option<String>,
}

/// [`Event`] triggered when the specified entity was interacted with.
#[derive(Event)]
pub struct InteractEvent(pub Entity);

pub(super) fn plugin(app: &mut App) {
	app.init_resource::<AvailableInteraction>()
		.add_observer(picking_click_observer)
		.add_observer(interact_by_input_action);
	app.add_systems(Update, iquick_plz_do_not_kill_me);
}

fn picking_click_observer(
	_trigger: On<Pointer<Click>>,
	resource: Res<AvailableInteraction>,
	mut commands: Commands,
) {
	if let Some(entity) = resource.target_entity {
		commands.trigger(InteractEvent(entity));
	}
}

fn iquick_plz_do_not_kill_me(
	spatial: SpatialQuery,
	cam: Single<&Transform, With<PlayerCameraParent>>,
	pickable: Query<(Entity, &Pickable)>,
	sensors: Query<Entity, With<Sensor>>,
	interaction_query: Query<&InteractableObject>,
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
	resource.description = None;
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
		&& interaction.0.as_ref().is_some_and(|desc| !desc.is_empty())
	{
		resource.target_entity = Some(collider.body);
		resource.description = interaction.0.clone();
	}
}

fn interact_by_input_action(
	_on: On<Start<Interact>>,
	focused_object: Res<AvailableInteraction>,
	interaction_query: Query<(), With<InteractableObject>>,
	mut commands: Commands,
) {
	if let Some(entity) = focused_object.target_entity {
		if interaction_query.contains(entity) {
			commands.trigger(InteractEvent(entity));
		}
	}
}

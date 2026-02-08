use avian3d::prelude::*;
use bevy::{camera::visibility::RenderLayers, prelude::*};
use bevy_enhanced_input::prelude::Start;

use crate::{
	gameplay::{
		player::{Player, camera::PlayerCamera, input::VomitObject},
		stomach::Stomach,
	},
	third_party::avian3d::CollisionLayer,
};

pub(super) fn plugin(app: &mut App) {
	app.add_observer(on_vomit);
	app.add_observer(try_vomit);
}

/// Event for vomiting an entity out of the stomach.
#[derive(EntityEvent, Debug)]
pub struct Vomit {
	/// The rigid body entity to vomit.
	#[event_target]
	pub body: Entity,
	/// The world position to vomit the entity to.
	pub origin: Vec3,
	/// The initial velocity to give the vomited entity.
	pub linear_velocity: Vec3,
}

fn on_vomit(
	vomit: On<Vomit>,
	mut object_query: Query<(&mut Transform, &mut LinearVelocity)>,
	mut layer_query: Query<(Option<&CollisionLayers>, Has<Mesh3d>)>,
	mut stomach: Single<&mut Stomach>,
	child_query: Query<&Children>,
	mut commands: Commands,
) {
	let Ok((mut transform, mut linear_velocity)) = object_query.get_mut(vomit.body) else {
		return;
	};

	// Move the entity to the vomit origin and set its velocity.
	transform.translation = vomit.origin;
	linear_velocity.0 = vomit.linear_velocity;
	stomach.contents.remove(&vomit.body);

	// Unlock the entity's Z translation.
	// TODO: Don't overwrite any other locked axes.
	commands.entity(vomit.body).remove::<LockedAxes>();

	// Remove the stomach collision and render layers.
	for entity in std::iter::once(vomit.body).chain(child_query.iter_descendants(vomit.body)) {
		let Ok((collision_layers, has_mesh)) = layer_query.get_mut(entity) else {
			continue;
		};

		if let Some(collision_layers) = collision_layers {
			let mut new_layers = *collision_layers;
			new_layers.memberships.remove(CollisionLayer::Stomach);
			new_layers.filters.remove(CollisionLayer::Stomach);
			commands.entity(entity).insert(new_layers);
		}

		if has_mesh {
			commands.entity(entity).remove::<RenderLayers>();
		}
	}
}

fn try_vomit(
	_vomit: On<Start<VomitObject>>,
	player_camera_transform: Single<&GlobalTransform, With<PlayerCamera>>,
	player_velocity: Single<&LinearVelocity, With<Player>>,
	stomach: Single<&Stomach>,
	vomitables: Query<(Entity, &GlobalTransform), With<RigidBody>>,
	mut commands: Commands,
) {
	let forward = player_camera_transform.forward();
	// TODO: Figure out a safe distance from the player to vomit from.
	let origin = player_camera_transform.translation() + 1.25 * forward;
	let linear_velocity = player_velocity.0 + 10.0 * forward;

	// Find the highest entity in the stomach to vomit out.
	let vomit_candidate = vomitables
		.iter_many(stomach.contents.iter())
		.max_by(|a, b| {
			a.1.translation()
				.y
				.partial_cmp(&b.1.translation().y)
				.unwrap()
		});

	if let Some((body, _)) = vomit_candidate {
		commands.trigger(Vomit {
			body,
			origin,
			linear_velocity,
		});
	}
}

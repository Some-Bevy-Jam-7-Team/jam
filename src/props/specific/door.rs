use std::f32::consts::PI;

use avian3d::prelude::*;
use bevy::{prelude::*, scene::SceneInstanceReady};

use bevy_trenchbroom::prelude::*;

use crate::{
	asset_tracking::LoadResource as _,
	third_party::{
		avian3d::CollisionLayer,
		bevy_trenchbroom::{GetTrenchbroomModelPath as _, LoadTrenchbroomModel as _},
	},
};

pub(super) fn plugin(app: &mut App) {
	app.add_observer(setup_door);
	app.load_asset::<Gltf>(Door::model_path());
}

#[point_class(base(Transform, Visibility), model("models/general/door.gltf"))]
pub(crate) struct Door;

fn setup_door(add: On<Add, Door>, asset_server: Res<AssetServer>, mut commands: Commands) {
	let model = asset_server.load_trenchbroom_model::<Door>();

	commands
		.entity(add.entity)
		.insert(SceneRoot(model))
		.observe(
			|ready: On<SceneInstanceReady>,
			 child_query: Query<&Children>,
			 name_query: Query<(Entity, &Name)>,
			 transform_helper: TransformHelper,
			 mut commands: Commands| {
				let descendants = child_query
					.iter_descendants(ready.entity)
					.collect::<Vec<_>>();

				// Find door panel entity by name.
				let Some((door_panel_entity, _)) = name_query
					.iter_many(descendants.iter())
					.find(|(_, name)| name.as_str() == "DoorPanel")
				else {
					return;
				};

				let global_transform = transform_helper
					.compute_global_transform(door_panel_entity)
					.unwrap();

				commands
					.entity(door_panel_entity)
					.remove::<ChildOf>()
					.insert((
						RigidBody::Dynamic,
						Transform::from(global_transform),
						ColliderConstructorHierarchy::new(
							ColliderConstructor::ConvexDecompositionFromMesh,
						)
						.with_default_layers(CollisionLayers::new(
							CollisionLayer::Prop,
							LayerMask::ALL,
						))
						.with_default_density(10000.0),
					));

				// Make the doorknobs children of the door panel, so they move together with the door.
				for (entity, name) in name_query.iter_many(descendants.iter()) {
					if name.as_str() == "DoorknobFront" || name.as_str() == "DoorknobBack" {
						let child_global_transform =
							transform_helper.compute_global_transform(entity).unwrap();
						commands.entity(entity).insert((
							ChildOf(door_panel_entity),
							child_global_transform.reparented_to(&global_transform),
						));
					}
				}

				// Make the original entity a static body with a collider.
				commands.entity(ready.entity).insert((
					RigidBody::Static,
					ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
				));

				// Create hinge joint between the door panel and the original entity (door frame).
				commands.spawn((
					RevoluteJoint::new(ready.entity, door_panel_entity)
						.with_hinge_axis(Vec3::Y)
						.with_local_basis2(Quat::from_rotation_y(PI))
						.with_angle_limits(-120f32.to_radians(), 120f32.to_radians())
						.with_anchor(
							global_transform.translation() - 0.425 * global_transform.right(),
						)
						.with_limit_compliance(1e-6),
					JointDamping {
						angular: 10.0,
						..default()
					},
					JointCollisionDisabled,
				));
			},
		);
}

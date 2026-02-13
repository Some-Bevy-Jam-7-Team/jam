use std::f32::consts::PI;

use avian_rerecast::ExcludeColliderFromNavmesh;
use avian3d::{dynamics::solver::joint_graph::JointGraph, prelude::*};
use bevy::{prelude::*, scene::SceneInstanceReady};

use bevy_trenchbroom::prelude::*;

use crate::{
	asset_tracking::LoadResource as _,
	gameplay::{TargetName, interaction::InteractEvent, player::camera::PlayerCameraParent},
	props::interactables::InteractableEntity,
	reflection::ReflAppExt,
	screens::Screen,
	third_party::bevy_trenchbroom::{GetTrenchbroomModelPath as _, LoadTrenchbroomModel as _},
};

pub(super) fn plugin(app: &mut App) {
	app.add_observer(setup_door);
	app.add_observer(interact_with_door);
	app.add_systems(Update, update_door_locks);
	app.load_asset::<Gltf>(Door::model_path());

	app.register_dynamic_component::<Door>();
}

#[derive(Component)]
struct DoorPanel;

#[point_class(
	base(TargetName, Transform, Visibility),
	model("models/general/door.gltf")
)]
pub(crate) struct Door {
	pub locked: bool,
	pub min_angle: f32,
	pub max_angle: f32,
}

impl Default for Door {
	fn default() -> Self {
		Self {
			locked: false,
			min_angle: -120f32.to_radians(),
			max_angle: 120f32.to_radians(),
		}
	}
}

fn setup_door(add: On<Add, Door>, asset_server: Res<AssetServer>, mut commands: Commands) {
	let model = asset_server.load_trenchbroom_model::<Door>();

	commands
		.entity(add.entity)
		.insert(SceneRoot(model))
		.observe(
			|ready: On<SceneInstanceReady>,
			 door_query: Query<&Door>,
			 child_query: Query<&Children>,
			 name_query: Query<(Entity, &Name)>,
			 transform_helper: TransformHelper,
			 mut commands: Commands| {
				let door = door_query.get(ready.entity).unwrap();

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
						DoorPanel,
						InteractableEntity::new_from_text("Clopen".to_string()),
						ExcludeColliderFromNavmesh,
						Transform::from(global_transform),
						DespawnOnExit(Screen::Gameplay),
						ColliderConstructorHierarchy::new(
							ColliderConstructor::ConvexDecompositionFromMesh,
						)
						.with_default_density(10_000.0),
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
					ExcludeColliderFromNavmesh,
					ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh),
				));

				// Create hinge joint between the door panel and the original entity (door frame).
				commands.spawn((
					RevoluteJoint::new(ready.entity, door_panel_entity)
						.with_hinge_axis(Vec3::Y)
						.with_local_basis2(Quat::from_rotation_y(PI))
						.with_angle_limits(
							if door.locked { 0.0 } else { door.min_angle },
							if door.locked { 0.0 } else { door.max_angle },
						)
						.with_anchor(
							global_transform.translation() - 0.425 * global_transform.right(),
						)
						.with_limit_compliance(1e-6)
						.with_motor(AngularMotor::new_disabled(MotorModel::SpringDamper {
							frequency: 1.5,
							damping_ratio: 1.0,
						})),
					JointDamping {
						angular: 10.0,
						..default()
					},
					JointCollisionDisabled,
					DespawnOnExit(Screen::Gameplay),
				));
			},
		);
}

/// Computes the current hinge angle of a revolute joint from the body rotations.
fn joint_angle(joint: &RevoluteJoint, rot1: Quat, rot2: Quat) -> f32 {
	let basis1 = joint.local_basis1().unwrap_or(Quat::IDENTITY);
	let basis2 = joint.local_basis2().unwrap_or(Quat::IDENTITY);

	let a1 = rot1 * basis1 * joint.hinge_axis;
	let b1 = rot1 * basis1 * joint.hinge_axis.any_orthonormal_vector();
	let b2 = rot2 * basis2 * joint.hinge_axis.any_orthonormal_vector();

	let sin_angle = b1.cross(b2).dot(a1);
	let cos_angle = b1.dot(b2);
	sin_angle.atan2(cos_angle)
}

/// Threshold in radians (~6 degrees) below which the door is considered closed.
const DOOR_CLOSED_THRESHOLD: f32 = 0.1;

fn interact_with_door(
	trigger: On<InteractEvent>,
	mut door_query: Query<&Transform, With<DoorPanel>>,
	cam: Single<&GlobalTransform, With<PlayerCameraParent>>,
	mut forces_query: Query<Forces>,
	joint_graph: Res<JointGraph>,
	mut joints: Query<&mut RevoluteJoint>,
	global_transforms: Query<&GlobalTransform>,
) {
	let entity = trigger.0;
	let Ok(door_transform) = door_query.get_mut(entity) else {
		return;
	};

	// Compute the current joint angle to determine if the door is open or closed.
	let mut current_angle = 0.0f32;
	for edge in joint_graph.joints_of(entity) {
		let Ok(joint) = joints.get(edge.entity) else {
			continue;
		};
		let (Ok(gt1), Ok(gt2)) = (
			global_transforms.get(joint.body1),
			global_transforms.get(joint.body2),
		) else {
			continue;
		};
		current_angle = joint_angle(joint, gt1.rotation(), gt2.rotation());
		break;
	}

	if current_angle.abs() > DOOR_CLOSED_THRESHOLD {
		// Door is open: enable spring motor and apply closing impulse.
		for edge in joint_graph.joints_of(entity) {
			if let Ok(mut joint) = joints.get_mut(edge.entity) {
				joint.motor.enabled = true;
			}
		}
		let torque_sign = if current_angle > 0.0 { -1.0 } else { 1.0 };
		if let Ok(mut forces) = forces_query.get_mut(entity) {
			forces.apply_angular_impulse(Vec3::Y * torque_sign * 4000.0);
		}
	} else {
		// Door is closed: disable motor, apply impulse away from player.
		for edge in joint_graph.joints_of(entity) {
			if let Ok(mut joint) = joints.get_mut(edge.entity) {
				joint.motor.enabled = false;
			}
		}

		let to_player = cam.translation() - door_transform.translation;
		let side = to_player.dot(door_transform.forward().into());
		let torque_sign = if side > 0.0 { -1.0 } else { 1.0 };

		if let Ok(mut forces) = forces_query.get_mut(entity) {
			forces.apply_angular_impulse(Vec3::Y * torque_sign * 4000.0);
		}
	}
}

fn update_door_locks(
	query: Query<(Entity, &Door), Changed<Door>>,
	mut joints: Query<&mut RevoluteJoint>,
	joint_graph: Res<JointGraph>,
) {
	for (entity, door) in query.iter() {
		for edge in joint_graph.joints_of(entity) {
			let Ok(mut joint) = joints.get_mut(edge.entity) else {
				continue;
			};
			let Some(limit) = &mut joint.angle_limit else {
				continue;
			};

			if door.locked {
				limit.min = 0.0;
				limit.max = 0.0;
			} else {
				limit.min = door.min_angle;
				limit.max = door.max_angle;
			}
		}
	}
}

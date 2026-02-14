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
	app.add_systems(
		FixedUpdate,
		(
			update_door_hinge_angle,
			close_doors,
			update_door_locks,
			stop_closing_on_door_blocked,
		)
			.chain(),
	);

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

/// Component to track the previous and current angle of the door hinge joint.
/// Used to determine if the door is opening or closing, and in which direction.
#[derive(Component, Default)]
pub struct DoorHingeAngle {
	/// Previous hinge angle in radians.
	pub previous: f32,
	/// Current hinge angle in radians.
	pub current: f32,
}

impl DoorHingeAngle {
	/// Threshold in radians below which the door is considered closed.
	pub const DOOR_CLOSED_THRESHOLD: f32 = 0.225;

	/// Returns true if the door is considered closed based on the current hinge angle.
	pub fn is_closed(&self) -> bool {
		self.current.abs() < Self::DOOR_CLOSED_THRESHOLD
	}

	/// Returns true if the door is currently closing (hinge angle decreasing in magnitude).
	pub fn is_closing(&self) -> bool {
		(self.previous.abs() - self.current.abs()) > 0.035
	}

	/// Computes the current hinge angle in radians based on the rotations
	/// of the two bodies connected by the joint.
	fn compute(joint: &RevoluteJoint, rot1: Quat, rot2: Quat) -> f32 {
		let basis1 = joint.local_basis1().unwrap_or(Quat::IDENTITY);
		let basis2 = joint.local_basis2().unwrap_or(Quat::IDENTITY);

		let a1 = rot1 * basis1 * joint.hinge_axis;
		let b1 = rot1 * basis1 * joint.hinge_axis.any_orthonormal_vector();
		let b2 = rot2 * basis2 * joint.hinge_axis.any_orthonormal_vector();

		let sin_angle = b1.cross(b2).dot(a1);
		let cos_angle = b1.dot(b2);
		sin_angle.atan2(cos_angle)
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
						Dominance(0),
						SleepingDisabled,
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
					DoorHingeAngle::default(),
					RevoluteJoint::new(ready.entity, door_panel_entity)
						.with_hinge_axis(Vec3::Y)
						.with_local_basis2(Quat::from_rotation_y(PI))
						.with_angle_limits(door.min_angle, door.max_angle)
						.with_anchor(
							global_transform.translation() - 0.425 * global_transform.right(),
						)
						.with_limit_compliance(1e6)
						.with_motor(AngularMotor::new_disabled(MotorModel::SpringDamper {
							frequency: 10.0,
							damping_ratio: 5.0,
						})),
					JointDamping {
						angular: 10.0,
						..default()
					},
					JointForces::default(),
					JointCollisionDisabled,
					DespawnOnExit(Screen::Gameplay),
				));
			},
		);
}

fn interact_with_door(
	trigger: On<InteractEvent>,
	door_transform_query: Query<&GlobalTransform, With<DoorPanel>>,
	door_query: Query<&Door>,
	cam: Single<&GlobalTransform, With<PlayerCameraParent>>,
	mut forces_query: Query<Forces>,
	joint_graph: Res<JointGraph>,
	mut joints: Query<(&mut RevoluteJoint, &DoorHingeAngle)>,
) {
	let entity = trigger.0;

	let Ok(door_transform) = door_transform_query.get(entity) else {
		return;
	};

	// We're just gonna assume the door has only one joint :3
	let Some(joint_edge) = joint_graph.joints_of(entity).next() else {
		return;
	};

	let Ok((mut joint, hinge_angle)) = joints.get_mut(joint_edge.entity) else {
		return;
	};

	let Ok(door) = door_query
		.get(joint.body1)
		.or_else(|_| door_query.get(joint.body2))
	else {
		return;
	};

	/*
	if door.locked {
		return;
	}

	 */

	if hinge_angle.is_closed() {
		// Door is closed: disable motor, apply impulse away from player.
		joint.motor.enabled = false;

		let to_player = cam.translation() - door_transform.translation();
		let side = to_player.dot(door_transform.forward().into());
		let torque_sign = if side > 0.0 { -1.0 } else { 1.0 };

		if let Ok(mut forces) = forces_query.get_mut(entity) {
			forces.apply_angular_impulse(Vec3::Y * torque_sign * 4000.0);
		}
	} else {
		// Door is open: enable spring motor and apply closing impulse.
		joint.motor.enabled = true;

		let torque_sign = if hinge_angle.current > 0.0 { -1.0 } else { 1.0 };
		if let Ok(mut forces) = forces_query.get_mut(entity) {
			forces.apply_angular_impulse(Vec3::Y * torque_sign * 4000.0);
		}
	}
}

fn update_door_hinge_angle(
	mut joints: Query<(&mut RevoluteJoint, &mut DoorHingeAngle)>,
	global_transforms: Query<&GlobalTransform>,
) {
	for (joint, mut hinge_angle) in &mut joints {
		let Ok([gt1, gt2]) = global_transforms.get_many([joint.body1, joint.body2]) else {
			continue;
		};
		hinge_angle.previous = hinge_angle.current;
		hinge_angle.current = DoorHingeAngle::compute(&joint, gt1.rotation(), gt2.rotation());
	}
}

fn close_doors(
	mut joints: Query<(&mut RevoluteJoint, &DoorHingeAngle)>,
	mut door_panel_query: Query<&mut Dominance, With<DoorPanel>>,
) {
	for (mut joint, hinge_angle) in &mut joints {
		let mut dominance = if let Ok(d) = door_panel_query.get_mut(joint.body1) {
			d
		} else if let Ok(d) = door_panel_query.get_mut(joint.body2) {
			d
		} else {
			continue;
		};

		if hinge_angle.is_closed() {
			// Set door dominance
			dominance.0 = 1;

			// Apply spring motor impulse to keep the door closed
			if hinge_angle.is_closing() {
				joint.motor.enabled = true;
			}
		} else {
			// Reset door dominance when not fully closed
			dominance.0 = 0;
		}
	}
}

fn stop_closing_on_door_blocked(
	mut joints: Query<(&mut RevoluteJoint, &JointForces)>,
	doors: Query<&Door>,
) {
	for (mut joint, forces) in &mut joints {
		let Ok(door) = doors.get(joint.body1).or_else(|_| doors.get(joint.body2)) else {
			continue;
		};
		if !door.locked && joint.motor.enabled && forces.motor_force().abs() > 240_000.0 {
			joint.motor.enabled = false;
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

			if door.locked {
				joint.motor.enabled = true;
			}
		}
	}
}

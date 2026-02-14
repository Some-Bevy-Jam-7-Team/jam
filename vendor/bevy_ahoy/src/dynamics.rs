use bevy_ecs::{intern::Interned, schedule::ScheduleLabel};

use crate::{CharacterControllerOutput, prelude::*};

pub struct AhoyDynamicPlugin {
	pub schedule: Interned<dyn ScheduleLabel>,
}

impl Plugin for AhoyDynamicPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			self.schedule,
			apply_forces.in_set(AhoySystems::ApplyForcesToDynamicRigidBodies),
		);
	}
}

fn apply_forces(
	kccs: Query<(
		&ComputedMass,
		&CharacterControllerOutput,
		Option<&Dominance>,
	)>,
	colliders: Query<&ColliderOf>,
	mut rigid_bodies: Query<(&RigidBody, Forces, Option<&Dominance>)>,
) {
	for (mass, output, dominance) in &kccs {
		let mass = mass.value();
		for touch in &output.touching_entities {
			let Ok(collider_of) = colliders.get(touch.entity) else {
				continue;
			};
			let Ok((rigid_body, mut forces, rb_dominance)) = rigid_bodies.get_mut(collider_of.body)
			else {
				continue;
			};

			let effective_dominance =
				dominance.map(|d| d.0).unwrap_or(0) + rb_dominance.map(|d| d.0).unwrap_or(0);

			if !rigid_body.is_dynamic() || effective_dominance > 0 {
				continue;
			}
			// TODO: not on step up

			let touch_dir = -touch.normal;
			let relative_velocity = touch.character_velocity - forces.linear_velocity();
			let touch_velocity = touch_dir.dot(relative_velocity) * touch_dir;
			let impulse = touch_velocity * mass;

			forces.apply_linear_impulse_at_point(impulse, touch.point);
		}
	}
}

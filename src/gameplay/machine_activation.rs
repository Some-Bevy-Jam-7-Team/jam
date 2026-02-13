use avian3d::prelude::*;
use bevy::prelude::*;

use super::{
	interaction::InteractEvent,
	objectives::ObjectiveCompletor,
	player::{
		Player,
		camera::{PlayerCameraParent, WorldModelCamera},
	},
};

pub(super) fn plugin(app: &mut App) {
	app.add_observer(on_machine_activated);
	app.add_systems(
		Update,
		(tick_screen_flash, tick_camera_shake),
	);
}

// -- Components --

#[derive(Component)]
struct ScreenFlash {
	timer: Timer,
}

impl ScreenFlash {
	fn new(duration: f32) -> Self {
		Self {
			timer: Timer::from_seconds(duration, TimerMode::Once),
		}
	}
}

#[derive(Component)]
struct CameraShake {
	timer: Timer,
	intensity: f32,
}

impl CameraShake {
	fn new(duration: f32, intensity: f32) -> Self {
		Self {
			timer: Timer::from_seconds(duration, TimerMode::Once),
			intensity,
		}
	}
}

// -- Observer --

const KNOCKBACK_FORCE: f32 = 10.0;

fn on_machine_activated(
	trigger: On<InteractEvent>,
	completors: Query<&ObjectiveCompletor>,
	machine_transforms: Query<&GlobalTransform>,
	mut player_velocity: Single<&mut LinearVelocity, With<Player>>,
	cam_parent: Single<&GlobalTransform, With<PlayerCameraParent>>,
	world_camera: Single<Entity, With<WorldModelCamera>>,
	mut commands: Commands,
) {
	let entity = trigger.0;
	let Ok(completor) = completors.get(entity) else {
		return;
	};
	if completor.target != "w1_turn_on" {
		return;
	}

	// Screen flash
	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			left: Val::Px(0.0),
			top: Val::Px(0.0),
			width: Val::Percent(100.0),
			height: Val::Percent(100.0),
			..default()
		},
		BackgroundColor(Color::WHITE),
		GlobalZIndex(i32::MAX),
		ScreenFlash::new(0.4),
	));

	// Camera shake
	commands
		.entity(*world_camera)
		.insert(CameraShake::new(0.4, 0.03));

	// Player knockback
	let machine_pos = machine_transforms.get(entity).unwrap().translation();
	let player_pos = cam_parent.translation();
	let away = (player_pos - machine_pos).with_y(0.0).normalize_or_zero();
	player_velocity.0 = away * KNOCKBACK_FORCE;
}

// -- Systems --

fn tick_screen_flash(
	mut commands: Commands,
	time: Res<Time>,
	mut query: Query<(Entity, &mut ScreenFlash, &mut BackgroundColor)>,
) {
	for (entity, mut flash, mut bg) in &mut query {
		flash.timer.tick(time.delta());
		let alpha = 1.0 - flash.timer.fraction();
		bg.0 = Color::srgba(1.0, 1.0, 1.0, alpha);
		if flash.timer.is_finished() {
			commands.entity(entity).despawn();
		}
	}
}

fn tick_camera_shake(
	mut commands: Commands,
	time: Res<Time>,
	mut query: Query<(Entity, &mut Transform, &mut CameraShake)>,
) {
	for (entity, mut transform, mut shake) in &mut query {
		shake.timer.tick(time.delta());
		if shake.timer.is_finished() {
			transform.translation = Vec3::ZERO;
			commands.entity(entity).remove::<CameraShake>();
		} else {
			let remaining = 1.0 - shake.timer.fraction();
			let t = time.elapsed_secs();
			let offset_x = (t * 127.1).sin() * shake.intensity * remaining;
			let offset_y = (t * 269.5).cos() * shake.intensity * remaining;
			transform.translation = Vec3::new(offset_x, offset_y, 0.0);
		}
	}
}

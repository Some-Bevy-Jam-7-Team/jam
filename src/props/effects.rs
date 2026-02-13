//! Utility functions for adding special effects to props or triggering special effects.

use bevy::{light::NotShadowCaster, prelude::*, scene::SceneInstanceReady};
use bevy_trenchbroom::prelude::*;

use std::iter;

use avian3d::prelude::*;
use bevy_seedling::sample::{AudioSample, SamplePlayer};

use crate::gameplay::interaction::InteractEvent;
use crate::{audio::SfxPool, gameplay::TargetName};

use crate::gameplay::player::{
	Player,
	camera::{PlayerCameraParent, WorldModelCamera},
};

pub(super) fn plugin(app: &mut App) {
	app.add_observer(on_special_effects);
	app.add_systems(Update, (tick_screen_flash, tick_camera_shake));
}

pub(crate) fn disable_shadow_casting_on_instance_ready(
	ready: On<SceneInstanceReady>,
	mut commands: Commands,
) {
	commands.entity(ready.entity).queue(disable_shadow_casting);
}

pub(crate) fn disable_shadow_casting(entity_world: EntityWorldMut) {
	let entity = entity_world.id();
	entity_world
		.into_world_mut()
		.run_system_cached_with(disable_shadow_casting_system, entity)
		.unwrap();
}

fn disable_shadow_casting_system(
	In(entity): In<Entity>,
	children: Query<&Children>,
	is_mesh: Query<&Mesh3d>,
	mut commands: Commands,
) {
	for child in iter::once(entity).chain(children.iter_descendants(entity)) {
		if is_mesh.get(child).is_ok() {
			commands.entity(child).insert(NotShadowCaster);
		}
	}
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

#[point_class(base(TargetName, Transform))]
#[derive(Default)]
pub struct SpecialEffectsNode {
	/// Sample to play
	pub sfx_sample: Option<String>,
	/// Screen-flash duration
	pub screen_flash_duration: Option<f32>,
	/// Camera-shake duration
	pub camera_shake_duration: Option<f32>,
	/// Camera-shake duration
	pub camera_shake_indensity: Option<f32>,
	/// Speed to knock-back the player from this entity with
	pub knockback_velocity: f32,
}

// -- Observer --

fn on_special_effects(
	trigger: On<InteractEvent>,
	sfx_nodes: Query<(&GlobalTransform, &SpecialEffectsNode)>,
	mut player_velocity: Single<&mut LinearVelocity, With<Player>>,
	cam_parent: Single<&GlobalTransform, With<PlayerCameraParent>>,
	world_camera: Single<Entity, With<WorldModelCamera>>,
	assets: Res<AssetServer>,
	mut commands: Commands,
) {
	let entity = trigger.0;

	let Ok((transform, sfx_node)) = sfx_nodes.get(entity) else {
		return;
	};

	// SFX
	if let Some(path) = &sfx_node.sfx_sample {
		let sfx: Handle<AudioSample> = assets.load(path);
		commands.spawn((SamplePlayer::new(sfx), SfxPool));
	}

	// Screen flash
	if let Some(time) = sfx_node.screen_flash_duration {
		if time.is_finite() && time > 0.0 {
			commands.spawn((
				Node {
					position_type: PositionType::Absolute,
					left: Val::Px(0.0),
					top: Val::Px(0.0),
					width: Val::Percent(100.0),
					height: Val::Percent(100.0),
					..default()
				},
				Pickable::IGNORE,
				BackgroundColor(Color::WHITE),
				GlobalZIndex(i32::MAX),
				ScreenFlash::new(time),
			));
		}
	}

	// Camera shake
	if let (Some(time), Some(intensity)) = (
		sfx_node.camera_shake_duration,
		sfx_node.camera_shake_indensity,
	) {
		if time.is_finite() && time > 0.0 {
			commands
				.entity(*world_camera)
				.insert(CameraShake::new(time, intensity));
		}
	}

	// Player knockback
	let machine_pos = transform.translation();
	let player_pos = cam_parent.translation();
	let away = (player_pos - machine_pos).with_y(0.0).normalize_or_zero();
	player_velocity.0 += away * sfx_node.knockback_velocity;
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

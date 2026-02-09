use avian3d::prelude::*;
use bevy::{
	anti_alias::fxaa::Fxaa,
	camera::{RenderTarget, ScalingMode, visibility::RenderLayers},
	core_pipeline::{
		prepass::{DeferredPrepass, DepthPrepass},
		tonemapping::Tonemapping,
	},
	ecs::entity::EntityHashSet,
	prelude::*,
	render::{render_resource::TextureFormat, view::Hdr},
};

use crate::{
	CameraOrder, RenderLayer,
	gameplay::player::{Player, camera::PlayerCamera},
	screens::Screen,
	third_party::avian3d::CollisionLayer,
};

pub(crate) mod eat;
pub(crate) mod vomit;

pub(super) fn plugin(app: &mut App) {
	app.add_plugins((eat::plugin, vomit::plugin));
	app.add_systems(
		OnEnter(Screen::Gameplay),
		(spawn_stomach, spawn_stomach_ui_and_render).chain(),
	);
	app.add_systems(FixedUpdate, move_stomach);
}

#[derive(Component, Debug)]
pub struct Stomach {
	pub target_size: Vec3,
	pub contents: EntityHashSet,
}

impl Default for Stomach {
	fn default() -> Self {
		Self {
			// We use a fairly large z-size, but movement is still locked in the z-axis.
			target_size: Vec3::new(2.5, 5.0, 10.0),
			contents: EntityHashSet::new(),
		}
	}
}

/// The offscreen position of the stomach.
const STOMACH_POSITION: Vec3 = Vec3::new(2000.0, 2000.0, 2000.0);

const MESH_THICKNESS: f32 = 0.25;

fn spawn_stomach(
	mut commands: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut materials: ResMut<Assets<StandardMaterial>>,
) {
	let stomach = Stomach::default();
	let mesh_thickness = MESH_THICKNESS;
	let vertical_mesh = meshes.add(Cuboid::new(
		mesh_thickness,
		stomach.target_size.y + mesh_thickness * 2.0,
		stomach.target_size.z + mesh_thickness * 2.0,
	));
	let horizontal_mesh = meshes.add(Cuboid::new(
		stomach.target_size.x + mesh_thickness * 2.0,
		mesh_thickness,
		stomach.target_size.z + mesh_thickness * 2.0,
	));
	let back_mesh = meshes.add(Cuboid::new(
		stomach.target_size.x + mesh_thickness * 2.0,
		stomach.target_size.y + mesh_thickness * 2.0,
		mesh_thickness,
	));
	let wall_material = materials.add(StandardMaterial {
		unlit: true,
		..Color::srgb(0.8, 0.1, 0.1).into()
	});
	let back_material = materials.add(StandardMaterial {
		unlit: true,
		..Color::srgb(0.4, 0.0, 0.0).into()
	});

	// TODO: Make the walls springy
	commands.spawn((
		Name::new("Stomach"),
		Stomach::default(),
		Transform::from_translation(STOMACH_POSITION),
		RigidBody::Kinematic,
		DespawnOnExit(Screen::Gameplay),
		Visibility::default(),
		children![
			(
				Name::new("Stomach Left Wall"),
				Collider::half_space(Vec3::X),
				CollisionLayers::new(CollisionLayer::Stomach, CollisionLayer::Stomach),
				Transform::from_translation(Vec3::new(-stomach.target_size.x / 2.0, 0.0, 0.0,)),
				Visibility::default(),
				children![(
					Mesh3d(vertical_mesh.clone()),
					MeshMaterial3d(wall_material.clone()),
					RenderLayers::from(RenderLayer::STOMACH),
					Transform::from_translation(Vec3::new(-mesh_thickness / 2.0, 0.0, 0.0)),
				)]
			),
			(
				Name::new("Stomach Right Wall"),
				Collider::half_space(-Vec3::X),
				CollisionLayers::new(CollisionLayer::Stomach, CollisionLayer::Stomach),
				Transform::from_translation(Vec3::new(stomach.target_size.x / 2.0, 0.0, 0.0,)),
				Visibility::default(),
				children![(
					Mesh3d(vertical_mesh),
					MeshMaterial3d(wall_material.clone()),
					RenderLayers::from(RenderLayer::STOMACH),
					Transform::from_translation(Vec3::new(mesh_thickness / 2.0, 0.0, 0.0)),
				)]
			),
			(
				Name::new("Stomach Ceiling"),
				Collider::half_space(-Vec3::Y),
				CollisionLayers::new(CollisionLayer::Stomach, CollisionLayer::Stomach),
				Transform::from_translation(Vec3::new(0.0, stomach.target_size.y / 2.0, 0.0)),
				Visibility::default(),
				children![(
					Mesh3d(horizontal_mesh.clone()),
					MeshMaterial3d(wall_material.clone()),
					RenderLayers::from(RenderLayer::STOMACH),
					Transform::from_translation(Vec3::new(0.0, mesh_thickness / 2.0, 0.0)),
				)]
			),
			(
				Name::new("Stomach Floor"),
				Collider::half_space(Vec3::Y),
				CollisionLayers::new(CollisionLayer::Stomach, CollisionLayer::Stomach),
				Transform::from_translation(Vec3::new(0.0, -stomach.target_size.y / 2.0, 0.0)),
				Visibility::default(),
				children![(
					Mesh3d(horizontal_mesh),
					MeshMaterial3d(wall_material),
					RenderLayers::from(RenderLayer::STOMACH),
					Transform::from_translation(Vec3::new(0.0, -mesh_thickness / 2.0, 0.0)),
				)]
			),
			(
				Name::new("Stomach Back Wall"),
				Collider::half_space(Vec3::Z),
				CollisionLayers::new(CollisionLayer::Stomach, CollisionLayer::Stomach),
				Transform::from_translation(Vec3::new(0.0, 0.0, -stomach.target_size.z / 2.0)),
				Visibility::default(),
				children![(
					Mesh3d(back_mesh),
					MeshMaterial3d(back_material),
					RenderLayers::from(RenderLayer::STOMACH),
					Transform::from_translation(Vec3::new(0.0, 0.0, -mesh_thickness / 2.0)),
				)]
			),
			(
				Name::new("Stomach Front Wall (Invisible)"),
				// No mesh for the front wall, so that we can see inside the stomach.
				Collider::half_space(-Vec3::Z),
				CollisionLayers::new(CollisionLayer::Stomach, CollisionLayer::Stomach),
				Transform::from_translation(Vec3::new(0.0, 0.0, stomach.target_size.z / 2.0,)),
			),
		],
	));
}

fn spawn_stomach_ui_and_render(
	mut commands: Commands,
	mut images: ResMut<Assets<Image>>,
	stomach: Single<(Entity, &Stomach)>,
) {
	let (stomach_entity, stomach) = *stomach;
	// We'll render the stomach and its contents to a texture.
	let aspect_ratio = stomach.target_size.x / stomach.target_size.y;
	let image = Image::new_target_texture(
		512,
		(512.0 / aspect_ratio) as u32,
		TextureFormat::Rgba8Unorm,
		Some(TextureFormat::Rgba8UnormSrgb),
	);
	let image_handle = images.add(image);

	// Spawn stomach camera.
	commands.spawn((
		Name::new("Stomach Camera"),
		ChildOf(stomach_entity),
		Transform::from_xyz(0.0, 0.0, 20.0),
		Camera3d::default(),
		Camera {
			// Bump the order to render on top of the world model.
			order: CameraOrder::Stomach.into(),
			..default()
		},
		Projection::Orthographic(OrthographicProjection {
			scaling_mode: ScalingMode::Fixed {
				width: stomach.target_size.x + MESH_THICKNESS * 2.0,
				height: stomach.target_size.y + MESH_THICKNESS * 2.0,
			},
			..OrthographicProjection::default_3d()
		}),
		Hdr,
		// Only render objects belonging to the stomach.
		RenderLayers::from(RenderLayer::STOMACH),
		// Render to the texture instead of the screen.
		RenderTarget::Image(image_handle.clone().into()),
		Tonemapping::TonyMcMapface,
		(DepthPrepass, Msaa::Off, DeferredPrepass, Fxaa::default()),
	));

	// Spawn stomach UI at the top right corner of the screen.
	commands.spawn((
		Name::new("Stomach UI"),
		Node {
			flex_direction: FlexDirection::Column,
			..default()
		},
		crate::ui_layout::RootWidget,
		DespawnOnExit(Screen::Gameplay),
		children![
			(
				Node {
					width: Val::Percent(100.0),
					justify_content: JustifyContent::Center,
					..default()
				},
				BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
				children![(
					// TODO: add red recording circle instead of ().
					Name::new("Stomach label"),
					Text("LIVE () STOMACH REACTION".into()),
					TextFont::from_font_size(18.0),
					TextColor(Color::BLACK),
				)]
			),
			(
				Node {
					width: Val::Px(256.0),
					height: Val::Px(256.0 / aspect_ratio),
					..default()
				},
				ImageNode {
					image: image_handle,
					..default()
				},
			)
		],
	));

	// Spawn a light to illuminate the stomach.
	commands.spawn((
		DirectionalLight {
			illuminance: 1e4,
			shadows_enabled: false,
			..default()
		},
		RenderLayers::from(RenderLayer::STOMACH),
		Transform::default().looking_to(Dir3::NEG_Z, Vec3::Y),
	));
}

fn move_stomach(
	mut stomach_velocity: Single<&mut LinearVelocity, (With<Stomach>, Without<Player>)>,
	player_camera_transform: Single<&GlobalTransform, With<PlayerCamera>>,
	player_velocity: Single<&LinearVelocity, With<Player>>,
) {
	let target_velocity = player_camera_transform.rotation().inverse() * player_velocity.0 * 0.5;
	let smoothing_factor = 0.1;
	stomach_velocity.0 = stomach_velocity.lerp(target_velocity, smoothing_factor);
	// Lock movement in the z-axis.
	stomach_velocity.z = 0.0;
}

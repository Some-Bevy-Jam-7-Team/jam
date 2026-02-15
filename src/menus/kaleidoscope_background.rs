use bevy::{
	prelude::*,
	render::render_resource::AsBindGroup,
	shader::ShaderRef,
	window::{PrimaryWindow, WindowResized},
};

use crate::screens::Screen;

pub(super) fn plugin(app: &mut App) {
	app.add_plugins(UiMaterialPlugin::<KaleidoscopeMaterial>::default());

	app.add_systems(OnEnter(Screen::Splash), spawn_kaleidoscope_background)
		.add_systems(OnEnter(Screen::Gameplay), remove_kaleidoscope_background)
		.add_systems(OnExit(Screen::Gameplay), spawn_kaleidoscope_background)
		.add_systems(Update, (resize_resolution, animation_time_system).chain());
}

fn spawn_kaleidoscope_background(
	mut commands: Commands,
	mut materials: ResMut<Assets<KaleidoscopeMaterial>>,
	window_query: Query<&Window, With<PrimaryWindow>>,
) {
	let window_size = window_query.single().map(|w| w.size());

	commands.spawn((
		Node {
			position_type: PositionType::Absolute,
			top: px(0.0),
			bottom: px(0.0),
			left: px(0.0),
			right: px(0.0),
			width: percent(100.0),
			height: percent(100.0),
			..default()
		},
		MaterialNode(materials.add(KaleidoscopeMaterial::new(
			window_size.unwrap_or(Vec2::new(1920.0, 1080.0)),
		))),
		ZIndex(-1),
	));
}

fn remove_kaleidoscope_background(
	mut commands: Commands,
	query: Query<Entity, With<MaterialNode<KaleidoscopeMaterial>>>,
) {
	for entity in query.iter() {
		commands.entity(entity).despawn();
	}
}

/// A UI material that implements a kaleidoscope shader effect.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct KaleidoscopeMaterial {
	/// Resolution of the shader's output, in pixels
	#[uniform(0)]
	pub resolution: Vec2,
	/// Time, in seconds
	#[uniform(0)]
	pub time: f32,
	/// Darkness parameter, 0 = bright, 1 = very dark
	#[uniform(0)]
	pub darkness: f32,
	/// Contrast parameter, 1–6 typical
	#[uniform(0)]
	pub contrast: f32,
	/// Highlight strength, 0–2
	#[uniform(0)]
	pub highlight_strength: f32,
	/// Glow strength, 0–3
	#[uniform(0)]
	pub glow_strength: f32,
	/// Shadow tone color
	#[uniform(0)]
	pub color_low: Vec3,
	/// Mid tone color
	#[uniform(0)]
	pub color_mid: Vec3,
	/// Highlight tone color
	#[uniform(0)]
	pub color_high: Vec3,
}

impl KaleidoscopeMaterial {
	/// Creates a new [`KaleidoscopeMaterial`] with the given resolution.
	pub fn new(resolution: Vec2) -> Self {
		Self {
			resolution,
			time: 0.0,
			darkness: 0.5,
			contrast: 3.0,
			highlight_strength: 3.0,
			glow_strength: 0.0,
			color_low: Vec3::new(0.1, 0.2, 0.3),
			color_mid: Vec3::new(0.4, 0.6, 0.8),
			color_high: Vec3::new(0.8, 0.9, 1.0),
		}
	}
}

impl UiMaterial for KaleidoscopeMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/kaleidoscope.wgsl".into()
	}
}

fn resize_resolution(
	mut resized: MessageReader<WindowResized>,
	query: Query<&MaterialNode<KaleidoscopeMaterial>>,
	mut materials: ResMut<Assets<KaleidoscopeMaterial>>,
) {
	for event in resized.read() {
		for material_node in query.iter() {
			if let Some(material) = materials.get_mut(&material_node.0) {
				material.resolution = Vec2::new(event.width, event.height) * 4.0;
			}
		}
	}
}

fn animation_time_system(
	time: Res<Time>,
	query: Query<&MaterialNode<KaleidoscopeMaterial>>,
	mut materials: ResMut<Assets<KaleidoscopeMaterial>>,
) {
	for material_node in query.iter() {
		if let Some(material) = materials.get_mut(&material_node.0) {
			material.time += time.delta_secs();
			let mut hue = material.time * 20.0;
			// Wrap hue to [0, 360)
			hue = hue - 360.0 * (hue / 360.0).floor();
			material.color_low = Srgba::from(Hsla::new(hue, 0.7, 0.01, 1.0)).to_vec3();
			material.color_mid = Srgba::from(Hsla::new(hue + 60.0, 0.8, 0.2, 1.0)).to_vec3();
			material.color_high = Srgba::from(Hsla::new(hue + 120.0, 0.7, 0.2, 1.0)).to_vec3();
		}
	}
}

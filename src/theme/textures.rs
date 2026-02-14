//! UI textures.

use bevy::{
	asset::{RenderAssetUsages, load_internal_binary_asset, uuid_handle},
	image::{CompressedImageFormats, ImageSampler, ImageType},
	prelude::*,
	render::render_resource::AsBindGroup,
	shader::ShaderRef,
};

pub(crate) const BUTTON_TEXTURE: Handle<Image> =
	uuid_handle!("005bc0cc-64c0-44ee-be79-4729bc4050bf");

const BUTTON_SHADER_ASSET_PATH: &str = "shaders/button_material.wgsl";

pub(crate) fn plugin(app: &mut App) {
	load_internal_binary_asset!(
		app,
		BUTTON_TEXTURE,
		"../../assets/textures/crate/crate_ambientOcclusion.png",
		load_image
	);

	app.add_plugins(UiMaterialPlugin::<TexturedUiMaterial>::default());
	app.add_observer(on_add_button);
	app.add_systems(Update, animate_textured_ui_material);
}

/// A UI material that applies a texture to the UI element, with some additional parameters for animation.
#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct TexturedUiMaterial {
	/// Color multiplied with the texture's color.
	#[uniform(0)]
	pub color: Vec4,
	/// Texture applied to the UI element
	#[texture(1)]
	#[sampler(2)]
	pub color_texture: Handle<Image>,
	/// Color of the border around the UI element
	#[uniform(3)]
	pub border_color: Vec4,
	#[uniform(4)]
	texture_translation: Vec2,
	#[uniform(4)]
	texture_rotation: f32,
	#[uniform(4)]
	_padding1: f32,
	#[uniform(5)]
	time: f32,
	#[uniform(5)]
	pub animation_speed: f32,
	#[uniform(5)]
	_padding2: Vec2,
}

impl Default for TexturedUiMaterial {
	fn default() -> Self {
		Self {
			color: Vec4::ONE,
			color_texture: Handle::default(),
			border_color: Vec4::ZERO,
			texture_translation: Vec2::ZERO,
			texture_rotation: 0.0,
			_padding1: 0.0,
			time: 0.0,
			animation_speed: 1.0,
			_padding2: Vec2::ZERO,
		}
	}
}

impl TexturedUiMaterial {
	pub fn new(
		color: impl Into<Color>,
		color_texture: Handle<Image>,
		animation_speed: f32,
	) -> Self {
		let color: Color = color.into();
		Self {
			color: color.to_srgba().to_vec4(),
			color_texture,
			border_color: Vec4::ZERO,
			texture_translation: Vec2::new(rand::random(), rand::random()),
			texture_rotation: rand::random_range(0.0..std::f32::consts::TAU),
			_padding1: 0.0,
			time: 0.0,
			animation_speed,
			_padding2: Vec2::ZERO,
		}
	}
}

impl UiMaterial for TexturedUiMaterial {
	fn fragment_shader() -> ShaderRef {
		BUTTON_SHADER_ASSET_PATH.into()
	}
}

fn on_add_button(
	add: On<Add, Button>,
	mut materials: ResMut<Assets<TexturedUiMaterial>>,
	mut query: Query<&mut MaterialNode<TexturedUiMaterial>>,
	time: Res<Time<Real>>,
) {
	let Ok(mut material_node) = query.get_mut(add.entity) else {
		return;
	};
	let material = TexturedUiMaterial {
		color: Vec4::ONE,
		color_texture: BUTTON_TEXTURE,
		border_color: Vec4::ZERO,
		time: time.elapsed_secs(),
		texture_translation: Vec2::new(rand::random(), rand::random()),
		texture_rotation: rand::random_range(0.0..std::f32::consts::TAU),
		animation_speed: 0.03,
		..default()
	};
	let material_handle = materials.add(material);
	material_node.0 = material_handle;
}

fn animate_textured_ui_material(
	mut materials: ResMut<Assets<TexturedUiMaterial>>,
	query: Query<&MaterialNode<TexturedUiMaterial>>,
	time: Res<Time<Real>>,
) {
	for material_node in query.iter() {
		if let Some(material) = materials.get_mut(&material_node.0) {
			material.time = time.elapsed_secs();
		}
	}
}

fn load_image(bytes: &[u8], _path: String) -> Image {
	Image::from_buffer(
		bytes,
		ImageType::Format(ImageFormat::Png),
		CompressedImageFormats::default(),
		true,
		ImageSampler::default(),
		RenderAssetUsages::default(),
	)
	.unwrap()
}

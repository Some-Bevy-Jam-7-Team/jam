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

	app.add_plugins(UiMaterialPlugin::<ButtonMaterial>::default());
	app.add_observer(on_add_button);
	app.add_systems(Update, animate_button_material);
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Default, Clone)]
pub struct ButtonMaterial {
	/// Color multiplied with the image
	#[uniform(0)]
	pub color: Vec4,
	/// Image used to represent the slider
	#[texture(1)]
	#[sampler(2)]
	pub color_texture: Handle<Image>,
	/// Color of the image's border
	#[uniform(3)]
	pub border_color: Vec4,
	#[uniform(4)]
	time: f32,
	#[uniform(4)]
	texture_translation: Vec2,
	#[uniform(4)]
	texture_rotation: f32,
}

impl UiMaterial for ButtonMaterial {
	fn fragment_shader() -> ShaderRef {
		BUTTON_SHADER_ASSET_PATH.into()
	}
}

fn on_add_button(
	add: On<Add, Button>,
	mut materials: ResMut<Assets<ButtonMaterial>>,
	mut query: Query<&mut MaterialNode<ButtonMaterial>>,
	time: Res<Time<Real>>,
) {
	let Ok(mut material_node) = query.get_mut(add.entity) else {
		return;
	};
	let material = ButtonMaterial {
		color: Vec4::ONE,
		color_texture: BUTTON_TEXTURE,
		border_color: Vec4::ZERO,
		time: time.elapsed_secs(),
		texture_translation: Vec2::new(rand::random(), rand::random()),
		texture_rotation: rand::random_range(0.0..std::f32::consts::TAU),
	};
	let material_handle = materials.add(material);
	material_node.0 = material_handle;
}

fn animate_button_material(
	mut materials: ResMut<Assets<ButtonMaterial>>,
	query: Query<&MaterialNode<ButtonMaterial>>,
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

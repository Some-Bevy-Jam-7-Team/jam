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
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
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
) {
	let Ok(mut material_node) = query.get_mut(add.entity) else {
		return;
	};
	let material = ButtonMaterial {
		color: Vec4::ONE,
		color_texture: BUTTON_TEXTURE,
		border_color: Vec4::ZERO,
	};
	let material_handle = materials.add(material);
	material_node.0 = material_handle;
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

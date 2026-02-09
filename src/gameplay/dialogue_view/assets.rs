use bevy::asset::RenderAssetUsages;
use bevy::asset::load_internal_binary_asset;
use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
use bevy::prelude::*;

pub(crate) fn ui_assets_plugin(app: &mut App) {
	load_internal_binary_asset!(
		app,
		image_handle::CONTINUE_INDICATOR,
		"../../../assets/sprites/dialogue_continue.png",
		load_image
	);
}

fn load_image(bytes: &[u8], _path: String) -> Image {
	const IS_SRGB: bool = true;
	Image::from_buffer(
		bytes,
		ImageType::Extension("png"),
		CompressedImageFormats::NONE,
		IS_SRGB,
		ImageSampler::Default,
		RenderAssetUsages::RENDER_WORLD,
	)
	.unwrap()
}

pub(crate) mod image_handle {
	use bevy::{asset::uuid_handle, prelude::*};

	pub(crate) const CONTINUE_INDICATOR: Handle<Image> =
		uuid_handle!("b45deb7a-170c-45af-be78-fd36af674355");
}

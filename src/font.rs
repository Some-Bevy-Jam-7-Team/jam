//! Font assets.

use bevy::{
	asset::{load_internal_binary_asset, uuid_handle},
	ecs::{lifecycle::HookContext, world::DeferredWorld},
	prelude::*,
};

pub(crate) const DEFAULT_FONT: Handle<Font> = uuid_handle!("28dfb1e9-7b35-454f-9fe3-3457797c40bc");
pub(crate) const VARIABLE_FONT: Handle<Font> = uuid_handle!("28dfb1e9-7b35-454f-9fe3-3457797c40bc");

pub(crate) fn plugin(app: &mut App) {
	load_internal_binary_asset!(
		app,
		DEFAULT_FONT,
		"../assets/fonts/Finger_Paint/FingerPaint-Regular.ttf",
		load_font
	);
	load_internal_binary_asset!(
		app,
		VARIABLE_FONT,
		"../assets/fonts/Shantell_Sans/ShantellSans-VariableFont_BNCE,INFM,SPAC,wght.ttf",
		load_font
	);

	app.world_mut()
		.register_component_hooks::<TextFont>()
		.on_add(on_add_text);
}

fn load_font(bytes: &[u8], _path: String) -> Font {
	Font::try_from_bytes(bytes.to_vec()).unwrap()
}

fn on_add_text(mut world: DeferredWorld, ctx: HookContext) {
	let mut font = world.get_mut::<TextFont>(ctx.entity).unwrap();

	// If the font is the default handle, replace it with our default font.
	if font.font == Handle::default() {
		font.font = DEFAULT_FONT;
	}
}

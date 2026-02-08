//! Font assets.

use bevy::{
	ecs::{lifecycle::HookContext, world::DeferredWorld},
	prelude::*,
};

pub(crate) fn plugin(app: &mut App) {
	app.register_type::<FontAssets>();
	app.world_mut()
		.register_component_hooks::<TextFont>()
		.on_add(on_add_text);

	let assets = app.world().resource::<AssetServer>();
	app.insert_resource(FontAssets {
		default: assets.load("fonts/Finger_Paint/FingerPaint-Regular.ttf"),
	});
}

#[derive(Resource, Clone, Debug, Reflect)]
#[reflect(Resource)]
pub(crate) struct FontAssets {
	pub default: Handle<Font>,
}

fn on_add_text(mut world: DeferredWorld, ctx: HookContext) {
	let default_font = world.resource::<FontAssets>().default.clone();
	let mut font = world.get_mut::<TextFont>(ctx.entity).unwrap();

	// If the font is the default handle, replace it with our default font.
	if font.font == Handle::default() {
		font.font = default_font;
	}
}

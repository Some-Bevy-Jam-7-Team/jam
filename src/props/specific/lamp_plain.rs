use avian3d::prelude::*;
use bevy::prelude::*;

use bevy_trenchbroom::prelude::*;

use crate::{
	asset_tracking::LoadResource as _,
	props::{effects::disable_shadow_casting_on_instance_ready, setup::static_bundle},
	third_party::bevy_trenchbroom::GetTrenchbroomModelPath as _,
};

pub(super) fn plugin(app: &mut App) {
	app.add_observer(setup_lamp_wall_electric);
	app.load_asset::<Gltf>(LampPlain::model_path());
}

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/lights/non-extinguishable/electric_plain1_unattached.gltf"),
	classname("light_lamp_plain")
)]
struct LampPlain {
	color: Color,
	intensity: f32,
}

impl Default for LampPlain {
	fn default() -> Self {
		Self {
			color: Color::srgb_u8(180, 180, 232),
			intensity: 13_000.0,
		}
	}
}

fn setup_lamp_wall_electric(
	add: On<Add, LampPlain>,
	lamp: Query<&LampPlain>,
	asset_server: Res<AssetServer>,
	mut commands: Commands,
) {
	let lamp = lamp.get(add.entity).unwrap();
	let bundle = static_bundle::<LampPlain>(&asset_server, ColliderConstructor::ConvexHullFromMesh);
	commands
		.entity(add.entity)
		.insert(bundle)
		.with_child((
			Transform::from_xyz(0.0, -0.08, -0.35),
			PointLight {
				color: lamp.color,
				intensity: lamp.intensity,
				radius: 0.05,
				range: 20.0,
				shadows_enabled: true,
				..default()
			},
		))
		.observe(disable_shadow_casting_on_instance_ready);
}

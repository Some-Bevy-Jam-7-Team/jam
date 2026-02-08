use std::f32::consts::TAU;

use avian3d::prelude::*;
use bevy::prelude::*;

use bevy_trenchbroom::prelude::*;

use crate::{
	asset_tracking::LoadResource as _,
	props::{effects::disable_shadow_casting_on_instance_ready, setup::static_bundle},
	third_party::bevy_trenchbroom::GetTrenchbroomModelPath as _,
};

pub(super) fn plugin(app: &mut App) {
	app.add_observer(setup_lamp_shaded);
	app.load_asset::<Gltf>(LampShaded::model_path());
}

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/lights/non-extinguishable/lamp_shaded03/lamp_shaded03.gltf"),
	classname("light_lamp_shaded03")
)]
pub(crate) struct LampShaded;

fn setup_lamp_shaded(
	add: On<Add, LampShaded>,
	asset_server: Res<AssetServer>,
	mut commands: Commands,
) {
	let bundle =
		static_bundle::<LampShaded>(&asset_server, ColliderConstructor::ConvexHullFromMesh);
	commands
		.entity(add.entity)
		.insert((
			bundle,
			children![(
				SpotLight {
					color: Color::srgb_u8(232, 199, 176),
					intensity: 800_000.0,
					radius: 0.1,
					shadows_enabled: true,
					..default()
				},
				Transform::from_xyz(0.0, 0.1, -0.25)
					.with_rotation(Quat::from_rotation_x(TAU / 4.5)),
			)],
		))
		.observe(disable_shadow_casting_on_instance_ready);
}

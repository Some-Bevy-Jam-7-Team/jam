use avian3d::prelude::*;
use bevy::prelude::*;

use bevy_trenchbroom::prelude::*;

use crate::{
    asset_tracking::LoadResource as _,
    gameplay::core::*,
    props::{effects::disable_shadow_casting_on_instance_ready, setup::static_bundle},
    third_party::bevy_trenchbroom::GetTrenchbroomModelPath as _,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(setup_lamp_wall_electric);
    app.load_asset::<Gltf>(LampWallElectric::model_path());
}

#[point_class(
    base(Transform, Visibility),
    model(
        "models/darkmod/lights/non-extinguishable/lamp_wall_electric_01/lamp_wall_electric_01.gltf"
    ),
    classname("light_lamp_wall_electric")
)]
pub(crate) struct LampWallElectric;

fn setup_lamp_wall_electric(
    add: On<Add, LampWallElectric>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let bundle =
        static_bundle::<LampWallElectric>(&asset_server, ColliderConstructor::ConvexHullFromMesh);
    commands
        .entity(add.entity)
        .insert((bundle, EnvironmentTemperature::default()))
        .with_child((
            Transform::from_xyz(0.0, -0.08, -0.35),
            PointLight {
                color: Color::srgb_u8(232, 199, 176),
                intensity: 40_000.0,
                radius: 0.05,
                range: 20.0,
                shadows_enabled: true,
                ..default()
            },
        ))
        .observe(disable_shadow_casting_on_instance_ready);
}

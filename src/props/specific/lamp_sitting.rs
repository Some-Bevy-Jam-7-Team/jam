use avian_pickup::prop::PreferredPickupRotation;
use avian3d::prelude::*;
use bevy::prelude::*;

use bevy_trenchbroom::prelude::*;

use crate::{
    asset_tracking::LoadResource as _,
    gameplay::core::*,
    props::{effects::disable_shadow_casting_on_instance_ready, setup::dynamic_bundle},
    third_party::bevy_trenchbroom::GetTrenchbroomModelPath as _,
};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(setup_lamp_sitting);
    app.load_asset::<Gltf>(LampSitting::model_path());
}

#[point_class(
    base(Transform, Visibility),
    model(
        "models/darkmod/lights/non-extinguishable/round_lantern_sitting/round_lantern_sitting.gltf"
    )
)]
pub(crate) struct LampSitting;

fn setup_lamp_sitting(
    add: On<Add, LampSitting>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let bundle = dynamic_bundle::<LampSitting>(
        &asset_server,
        ColliderConstructor::ConvexDecompositionFromMesh,
    );
    commands
        .entity(add.entity)
        // The prop should be held upright.
        .insert((
            bundle,
            PreferredPickupRotation(Quat::IDENTITY),
            EnvironmentTemperature::default(),
        ))
        // The lamp's origin is at the bottom of the lamp, so we need to offset the light a bit.
        .with_child((
            Transform::from_xyz(0.0, 0.2, 0.0),
            PointLight {
                color: Color::srgb(1.0, 0.7, 0.4),
                intensity: 40_000.0,
                radius: 0.05,
                shadows_enabled: true,
                ..default()
            },
        ))
        .observe(disable_shadow_casting_on_instance_ready);
}

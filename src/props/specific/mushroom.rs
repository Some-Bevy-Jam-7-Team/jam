use crate::ReflectComponent;
use crate::asset_tracking::LoadResource;
use crate::gameplay::core::EnvironmentTemperature;
use crate::props::setup::{setup_static_prop_with_convex_hull, static_bundle};
use crate::scatter::layers::MushroomLayer;
use crate::third_party::bevy_trenchbroom::GetTrenchbroomModelPath;

use avian3d::prelude::ColliderConstructor;
use bevy::prelude::*;
use bevy_feronia::prelude::ScatteredInstance;
use bevy_trenchbroom::prelude::ReflectQuakeClass;
use bevy_trenchbroom::prelude::point_class;

pub(in crate::props) fn plugin(app: &mut App) {
	app.add_plugins(MushroomPlugin);
}

struct MushroomPlugin;

impl Plugin for MushroomPlugin {
	fn build(&self, app: &mut App) {
		app.add_observer(setup_mushroom);
		app.add_observer(setup_static_prop_with_convex_hull::<MushroomModel>);
		app.load_asset::<Gltf>(MushroomModel::model_path());
		app.add_observer(scattered_shroom);
	}
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[require(EnvironmentTemperature)]
pub(crate) struct Mushroom;

#[point_class(
	base(Transform, Visibility),
	model("models/mushroom/mushroom_single.gltf")
)]
pub(crate) struct MushroomModel;

fn setup_mushroom(
	add: On<Add, MushroomModel>,
	asset_server: Res<AssetServer>,
	mut commands: Commands,
) {
	let bundle =
		static_bundle::<MushroomModel>(&asset_server, ColliderConstructor::ConvexHullFromMesh);
	commands.entity(add.entity).insert((bundle, Mushroom));
}

pub fn scattered_shroom(
	trigger: On<Add, ScatteredInstance>,
	q_scattered_instance: Query<&ScatteredInstance>,
	q_mushroom_layer: Query<(), With<MushroomLayer>>,
	mut cmd: Commands,
) {
	if q_scattered_instance
		.get(trigger.entity)
		.and_then(|instance| q_mushroom_layer.get(**instance))
		.is_ok()
	{
		cmd.entity(trigger.entity).insert(Mushroom);
	}
}

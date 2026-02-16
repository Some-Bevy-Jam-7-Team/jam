use crate::ReflectComponent;
use crate::asset_tracking::LoadResource;
use crate::gameplay::TargetName;
use crate::gameplay::core::EnvironmentTemperature;
use crate::gameplay::level::CurrentLevel;
use crate::props::interactables::InteractableEntity;
use crate::props::setup::{setup_static_prop_with_convex_hull, static_bundle};
use crate::scatter::layers::MushroomLayer;
use crate::third_party::bevy_trenchbroom::GetTrenchbroomModelPath;

use avian3d::prelude::ColliderConstructor;
use bevy::prelude::*;
use bevy_feronia::prelude::ScatteredInstance;
use bevy_trenchbroom::prelude::point_class;
use bevy_trenchbroom::prelude::{QuakeClass, ReflectQuakeClass};

pub(in crate::props) fn plugin(app: &mut App) {
	app.add_plugins(MushroomPlugin);
}

struct MushroomPlugin;

impl Plugin for MushroomPlugin {
	fn build(&self, app: &mut App) {
		app.add_observer(setup_mushroom::<MushroomModel1>);
		app.add_observer(setup_mushroom::<MushroomModel2>);
		app.add_observer(setup_mushroom::<MushroomModel3>);
		app.add_observer(setup_mushroom::<MushroomModel4>);
		app.add_observer(setup_mushroom::<MushroomModel5>);
		app.add_observer(setup_static_prop_with_convex_hull::<MushroomModel1>);
		app.add_observer(setup_static_prop_with_convex_hull::<MushroomModel2>);
		app.add_observer(setup_static_prop_with_convex_hull::<MushroomModel3>);
		app.add_observer(setup_static_prop_with_convex_hull::<MushroomModel4>);
		app.add_observer(setup_static_prop_with_convex_hull::<MushroomModel5>);
		app.load_asset::<Gltf>(MushroomModel1::model_path());
		app.load_asset::<Gltf>(MushroomModel2::model_path());
		app.load_asset::<Gltf>(MushroomModel3::model_path());
		app.load_asset::<Gltf>(MushroomModel4::model_path());
		app.load_asset::<Gltf>(MushroomModel5::model_path());
		app.add_observer(scattered_shroom);
	}
}

#[point_class(
	base(TargetName, Transform, Visibility),
	model("models/mushroom/mushroom1.gltf")
)]
pub(crate) struct MushroomModel1;

#[point_class(
	base(TargetName, Transform, Visibility),
	model("models/mushroom/mushroom2.gltf")
)]
pub(crate) struct MushroomModel2;

#[point_class(
	base(TargetName, Transform, Visibility),
	model("models/mushroom/mushroom3.gltf")
)]
pub(crate) struct MushroomModel3;

#[point_class(
	base(TargetName, Transform, Visibility),
	model("models/mushroom/mushroom4.gltf")
)]
pub(crate) struct MushroomModel4;

#[point_class(
	base(TargetName, Transform, Visibility),
	model("models/mushroom/mushroom5.gltf")
)]
pub(crate) struct MushroomModel5;

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
#[require(EnvironmentTemperature)]
pub(crate) struct Mushroom;

fn setup_mushroom<T: Component + QuakeClass>(
	add: On<Add, T>,
	asset_server: Res<AssetServer>,
	mut commands: Commands,
) {
	let bundle = static_bundle::<T>(&asset_server, ColliderConstructor::ConvexHullFromMesh);
	commands.entity(add.entity).insert((bundle, Mushroom));
}

pub fn scattered_shroom(
	trigger: On<Add, ScatteredInstance>,
	q_scattered_instance: Query<&ScatteredInstance>,
	q_mushroom_layer: Query<(), With<MushroomLayer>>,
	current_level: Res<CurrentLevel>,
	mut cmd: Commands,
) {
	if q_scattered_instance
		.get(trigger.entity)
		.and_then(|instance| q_mushroom_layer.get(**instance))
		.is_ok()
	{
		cmd.entity(trigger.entity).insert(Mushroom);
		if *current_level != CurrentLevel::Commune {
			return;
		}

		cmd.entity(trigger.entity).insert(InteractableEntity {
			is_edible: true,
			interaction_text_override: Some("Take a bite".to_string()),
			completes_subobjective: Some("leave".to_string()),
			interaction_relay: None,
		});
	}
}

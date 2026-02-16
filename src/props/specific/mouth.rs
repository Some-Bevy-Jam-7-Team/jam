use avian3d::prelude::*;
use bevy::{prelude::*, scene::SceneInstanceReady};
use bevy_trenchbroom::prelude::*;

use crate::{
	asset_tracking::LoadResource as _,
	gameplay::TargetName,
	props::interactables::InteractableEntity,
	third_party::{
		avian3d::CollisionLayer,
		bevy_trenchbroom::{GetTrenchbroomModelPath as _, LoadTrenchbroomModel as _},
	},
};

pub(super) fn plugin(app: &mut App) {
	app.load_asset::<Gltf>(Mouth::model_path());
	app.add_systems(Update, pulsate_mouth);
	app.add_observer(setup_mouth);
}

#[point_class(model("models/landscape/landscape_flat_large.gltf"))]
pub(crate) struct LandscapePreview;

#[point_class(
	base(Transform, Visibility, TargetName),
	model("models/mouth/mouth.gltf")
)]
pub(crate) struct Mouth;

fn setup_mouth(add: On<Add, Mouth>, mut commands: Commands, asset_server: Res<AssetServer>) {
	let model = asset_server.load_trenchbroom_model::<Mouth>();
	let mouth_entity = add.entity;
	commands
		.entity(add.entity)
		.insert((
			ColliderConstructorHierarchy::new(ColliderConstructor::TrimeshFromMesh)
				.with_default_layers(CollisionLayers::new(
					CollisionLayer::Default,
					LayerMask::ALL,
				)),
			RigidBody::Static,
			SceneRoot(model),
		))
		.observe(
			move |ready: On<SceneInstanceReady>,
			      mut commands: Commands,
			      names: Query<(Entity, &Name)>,
			      children: Query<&Children>| {
				for (entity, name) in names.iter_many(children.iter_descendants(ready.entity)) {
					match name.as_str() {
						"Bone" => {
							commands.entity(entity).insert(Bone1Of(mouth_entity));
						}
						"Bone.001" => {
							commands.entity(entity).insert(Bone2Of(mouth_entity));
						}
						_ => {}
					}
				}
			},
		);
}

fn pulsate_mouth(
	time: Res<Time>,
	mouth: Single<(&Bone1, &Bone2)>,
	mut transforms: Query<&mut Transform>,
) {
	let (bone1, _bone2) = mouth.into_inner();
	if let Ok(mut bone1) = transforms.get_mut(bone1.0) {
		let t = time.elapsed_secs();
		let t_s = (t * 0.615).sin();
		bone1.scale = Vec3::ONE - (t_s * t_s * t_s * t_s) * 0.268;
		let t_s = ((t + 812.214) * 0.34).sin();
		bone1.scale.y = 1.0 + (t_s * t_s) * 0.1874;
	};
}

#[derive(Component, Clone, PartialEq, Eq, Debug)]
#[relationship(relationship_target = Bone1)]
pub struct Bone1Of(#[entities] pub Entity);

#[derive(Component, Clone, PartialEq, Eq, Debug)]
#[relationship_target(relationship = Bone1Of, linked_spawn)]
pub struct Bone1(#[entities] Entity);

#[derive(Component, Clone, PartialEq, Eq, Debug)]
#[relationship(relationship_target = Bone2)]
pub struct Bone2Of(#[entities] pub Entity);

#[derive(Component, Clone, PartialEq, Eq, Debug)]
#[relationship_target(relationship = Bone2Of, linked_spawn)]
pub struct Bone2(#[entities] Entity);

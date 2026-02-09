//! NPC handling. In the demo, the NPC is a fox that moves towards the player. We can interact with the NPC to trigger dialogue.

use animation::{NpcAnimationState, setup_npc_animations};
use avian3d::prelude::*;
use bevy::prelude::*;

use bevy_ahoy::CharacterController;
use bevy_trenchbroom::prelude::*;

use crate::{
	animation::AnimationState,
	asset_tracking::LoadResource,
	third_party::{
		avian3d::CollisionLayer,
		bevy_trenchbroom::{GetTrenchbroomModelPath, LoadTrenchbroomModel as _},
		bevy_yarnspinner::YarnNode,
	},
};

use super::animation::AnimationPlayerAncestor;
pub(crate) mod ai;
mod animation;
mod assets;
mod sound;

pub(super) fn plugin(app: &mut App) {
	app.add_plugins((ai::plugin, animation::plugin, assets::plugin, sound::plugin));
	app.load_asset::<Gltf>(Npc::model_path());
	app.load_asset::<Gltf>(Jan::model_path());
	app.add_observer(on_add).add_observer(on_add_jan);
}

#[point_class(base(Transform, Visibility), model("models/jan_npc/jan.gltf"))]
pub(crate) struct Npc;

#[point_class(base(Transform, Visibility), model("models/jan_npc/jan.gltf"))]
pub(crate) struct Jan;

pub(crate) const NPC_RADIUS: f32 = 0.6;
pub(crate) const NPC_HEIGHT: f32 = 1.3;
const NPC_HALF_HEIGHT: f32 = NPC_HEIGHT / 2.0;
const NPC_SPEED: f32 = 7.0;

fn on_add(add: On<Add, Npc>, mut commands: Commands, assets: Res<AssetServer>) {
	commands
		.entity(add.entity)
		.insert((
			Npc,
			Collider::cylinder(NPC_RADIUS, NPC_HEIGHT),
			CharacterController {
				speed: NPC_SPEED,
				filter: SpatialQueryFilter::DEFAULT
					.with_mask(LayerMask::ALL & !CollisionLayer::Stomach.to_bits()),
				..default()
			},
			ColliderDensity(1_000.0),
			RigidBody::Kinematic,
			AnimationState::<NpcAnimationState>::default(),
			AnimationPlayerAncestor,
			CollisionLayers::new(CollisionLayer::Character, LayerMask::ALL),
			// The Yarn Node is what we use to trigger dialogue.
			YarnNode::new("Npc"),
		))
		.with_child((
			Name::new("Npc Model"),
			SceneRoot(assets.load_trenchbroom_model::<Npc>()),
			Transform::from_xyz(0.0, -NPC_HALF_HEIGHT, 0.0),
		))
		.observe(setup_npc_animations);
}

fn on_add_jan(add: On<Add, Jan>, mut commands: Commands, assets: Res<AssetServer>) {
	commands
		.entity(add.entity)
		.insert((
			Collider::cylinder(NPC_RADIUS, NPC_HEIGHT),
			CharacterController {
				speed: NPC_SPEED,
				filter: SpatialQueryFilter::DEFAULT
					.with_mask(LayerMask::ALL & !CollisionLayer::Stomach.to_bits()),
				..default()
			},
			ColliderDensity(1_000.0),
			RigidBody::Kinematic,
			AnimationState::<NpcAnimationState>::default(),
			AnimationPlayerAncestor,
			CollisionLayers::new(CollisionLayer::Character, LayerMask::ALL),
		))
		.with_child((
			Name::new("Npc Model"),
			SceneRoot(assets.load_trenchbroom_model::<Jan>()),
			Transform::from_xyz(0.0, -NPC_HALF_HEIGHT, 0.0),
		));
}

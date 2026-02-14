use crate::gameplay::level::{CurrentLevel, EnvironmentAssets};
use crate::third_party::avian3d::CollisionLayer;

use avian3d::prelude::*;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_feronia::prelude::{HeightMapState, ScatterState};

#[derive(Event)]
pub struct ScatterDone;

#[derive(Component)]
#[component(on_add = Self::on_add)]
#[require(RigidBody::Static, Name::new("Landscape"))]
pub struct Landscape;

impl Landscape {
	pub fn on_add(mut world: DeferredWorld, ctx: HookContext) {
		world
			.get_resource_mut::<NextState<ScatterState>>()
			.unwrap()
			.set(ScatterState::Setup);

		world
			.get_resource_mut::<NextState<HeightMapState>>()
			.unwrap()
			.set(HeightMapState::Setup);

		let level = world.get_resource::<CurrentLevel>().unwrap();

		match *level {
			CurrentLevel::DayOne | CurrentLevel::Karoline | CurrentLevel::Train => {}
			CurrentLevel::DayTwo => {
				let landscape = world
					.get_resource::<EnvironmentAssets>()
					.map(|a| a.landscape.clone())
					.expect("Assets should be loaded.");

				world.commands().entity(ctx.entity).insert((
					SceneRoot(landscape.clone()),
					ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh)
						.with_default_layers(CollisionLayers::new(
							CollisionLayer::Default,
							LayerMask::ALL,
						))
						.with_default_density(1_000.0),
				));
			}
		}
	}
}

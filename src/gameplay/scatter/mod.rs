use avian3d::prelude::*;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy_eidolon::prelude::*;
use bevy_feronia::prelude::*;

use crate::gameplay::core::EnvironmentTemperature;
use crate::gameplay::level::LevelAssets;
use crate::third_party::avian3d::CollisionLayer;
use crate::{RenderLayer, RenderLayers};

#[derive(Component)]
#[component(on_add = Self::on_add)]
pub struct Landscape;

impl Landscape {
	pub fn on_add(mut world: DeferredWorld, _ctx: HookContext) {
		world
			.get_resource_mut::<NextState<ScatterState>>()
			.unwrap()
			.set(ScatterState::Setup);

		world
			.get_resource_mut::<NextState<HeightMapState>>()
			.unwrap()
			.set(HeightMapState::Setup);
	}
}

pub fn update_density_map(
	mut ev_asset: MessageReader<AssetEvent<Image>>,
	mut assets: ResMut<Assets<Image>>,
	mut level_assets: ResMut<LevelAssets>,
) {
	for ev in ev_asset.read() {
		if let AssetEvent::Modified { id, .. } = ev {
			if *id == level_assets.grass_density_map.id() {
				level_assets.grass_density_map = assets.get_strong_handle(*id).unwrap();
			}
			if *id == level_assets.rock_density_map.id() {
				level_assets.rock_density_map = assets.get_strong_handle(*id).unwrap();
			}
			if *id == level_assets.mushroom_density_map.id() {
				level_assets.mushroom_density_map = assets.get_strong_handle(*id).unwrap();
			}
		}
	}
}

#[derive(Component)]
#[require(EnvironmentTemperature)]
pub struct Mushroom;

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

#[derive(Component)]
#[component(on_add = Self::on_add)]
#[require(
    Name::new("Rock Layer"),
    ScatterLayerType::<StandardMaterial>,
    InstanceRotationYaw,
    InstanceScale,
    InstanceScaleRange{
		min: 8.,
	    max: 16.
	},
    InstanceJitter,
    DistributionDensity(25.),
    Avoidance(1.6),
)]
pub(crate) struct RockLayer;

impl RockLayer {
	fn on_add(mut world: DeferredWorld, ctx: HookContext) {
		let LevelAssets {
			rocks,
			rocks_med,
			rocks_low,
			rock_density_map,
			..
		} = world
			.get_resource::<LevelAssets>()
			.cloned()
			.expect("Assets should be added!");

		let mut cmd = world.commands();

		let collider_hierarchy =
			ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh)
				.with_default_layers(CollisionLayers::new(
					CollisionLayer::Default,
					LayerMask::ALL,
				));

		cmd.entity(ctx.entity)
			.insert((DistributionPattern(rock_density_map),));

		cmd.spawn_batch([
			(
				LevelOfDetail(0),
				ChildOf(ctx.entity),
				SceneRoot(rocks),
				collider_hierarchy.clone(),
				RigidBody::Static,
			),
			(
				LevelOfDetail(1),
				ChildOf(ctx.entity),
				SceneRoot(rocks_med),
				collider_hierarchy.clone(),
				RigidBody::Static,
			),
			(
				LevelOfDetail(2),
				ChildOf(ctx.entity),
				SceneRoot(rocks_low),
				collider_hierarchy,
				RigidBody::Static,
			),
		]);
	}
}

#[derive(Component)]
#[component(on_add = Self::on_add)]
#[require(
    Name::new("Mushroom Layer"),
    ScatterLayerType::<ExtendedWindAffectedMaterial>,
    InstanceRotationYaw,
    InstanceScale,
	InstanceScaleRange {
       min: 4.,
	   max: 32.
	},
    InstanceJitter,
	Strength(0.2),
	MicroStrength(0.1),
	SCurveStrength(0.1),
	BopStrength(0.2),
    DistributionDensity(20.),
    Avoidance(0.02),
	WindAffected,
	SubsurfaceScattering,
)]
pub struct MushroomLayer;

impl MushroomLayer {
	fn on_add(mut world: DeferredWorld, ctx: HookContext) {
		let LevelAssets {
			mushroom,
			mushroom_density_map,
			..
		} = world
			.get_resource::<LevelAssets>()
			.cloned()
			.expect("Assets should be added!");

		let mut cmd = world.commands();

		let collider_hierarchy =
			ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh)
				.with_default_layers(CollisionLayers::new(CollisionLayer::Prop, LayerMask::ALL));

		cmd.entity(ctx.entity)
			.insert((DistributionPattern(mushroom_density_map),));

		cmd.spawn_batch([
			(
				ChildOf(ctx.entity),
				SceneRoot(mushroom.clone()),
				LevelOfDetail(0),
				RigidBody::Static,
				collider_hierarchy.clone(),
			),
			(
				ChildOf(ctx.entity),
				SceneRoot(mushroom.clone()),
				LevelOfDetail(1),
				RigidBody::Static,
				collider_hierarchy.clone(),
			),
			(
				ChildOf(ctx.entity),
				SceneRoot(mushroom),
				LevelOfDetail(2),
				RigidBody::Static,
				collider_hierarchy,
			),
		]);
	}
}

#[derive(Component)]
#[component(on_add = Self::on_add)]
#[require(
    Name::new("Grass Layer"),
    ScatterLayerType::<InstancedWindAffectedMaterial>,

    // Scatter options

	DistributionDensity(150.0),
    InstanceJitter,
    InstanceScale,
    ScatterChunked,
    ScaleDensity,

    // Material options
	WindAffected,
    CurveNormals,
    AnalyticalNormals,
    InstanceRotationYaw,
    StandardPbr,
    SubsurfaceScattering,
	InstanceColor::new(Srgba::hex("#1f3114").unwrap()),
	InstanceColorGradient {
		end: 0.7,
		start: 0.2,
		..InstanceColorGradient::new(
			Srgba::hex("#3e6328").unwrap(),
			Srgba::hex("#0f190a").unwrap()
		)
	},
    StaticBend,
    AmbientOcclusion,
	MicroStrength(1.2),
	GpuCullCompute,
	RenderLayers::from(RenderLayer::GRASS),
)]
pub(crate) struct GrassLayer;

impl GrassLayer {
	fn on_add(mut world: DeferredWorld, ctx: HookContext) {
		let LevelAssets {
			grass,
			grass_med,
			grass_low,
			grass_density_map,
			..
		} = world
			.get_resource::<LevelAssets>()
			.cloned()
			.expect("Assets should be added!");

		let mut cmd = world.commands();

		cmd.entity(ctx.entity)
			.insert((DistributionPattern(grass_density_map),));

		// Just for collecting the asset, since we use avian anyway and the backend requires it when using the `avian` feature.
		let collider_hierarchy =
			ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh);

		cmd.spawn_batch([
			(
				SceneRoot(grass),
				ChildOf(ctx.entity),
				LevelOfDetail(0),
				collider_hierarchy.clone(),
			),
			(
				SceneRoot(grass_med),
				ChildOf(ctx.entity),
				LevelOfDetail(1),
				collider_hierarchy.clone(),
			),
			(
				SceneRoot(grass_low),
				ChildOf(ctx.entity),
				LevelOfDetail(2),
				collider_hierarchy,
			),
		]);
	}
}

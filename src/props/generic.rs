use std::time::Duration;

use crate::{
	asset_tracking::LoadResource,
	audio::{SfxPool, SpatialPool},
	gameplay::{
		level::LevelAssets,
		objectives::{CurrentObjective, Objective, ObjectiveCompleted, SubObjectives},
	},
	props::{interactables::InteractableEntity, logic_entity::ObjectiveEntity},
	third_party::{
		avian3d::CollisionLayer,
		bevy_trenchbroom::{GetTrenchbroomModelPath as _, LoadTrenchbroomModel as _},
		bevy_yarnspinner::YarnNode,
	},
	timer::{GenericTimer, TimerFinished},
};

use super::setup::*;
use avian3d::prelude::*;
use bevy::{
	asset::io::embedded::GetAssetServer,
	ecs::{lifecycle::HookContext, world::DeferredWorld},
	prelude::*,
};
use bevy_seedling::{prelude::RepeatMode, sample::SamplePlayer};
use bevy_trenchbroom::prelude::*;

pub(super) fn plugin(app: &mut App) {
	app.add_observer(setup_static_prop_with_convex_hull::<Grate>)
		.add_observer(setup_static_prop_with_convex_decomposition::<Table>)
		.add_observer(setup_static_prop_with_convex_hull::<Bookshelf>)
		.add_observer(setup_static_prop_with_convex_hull::<Generator2>)
		.add_observer(setup_static_prop_with_convex_hull::<BarrelLargeClosed>)
		.add_observer(setup_static_prop_with_convex_hull::<Barrel01>)
		.add_observer(setup_static_prop_with_convex_hull::<CrateSquare>)
		.add_observer(setup_static_prop_with_convex_hull::<FenceBarsDecorativeSingle>)
		.add_observer(setup_static_prop_with_convex_hull::<DoorStainedGlass>);

	app.add_observer(setup_static_prop_with_convex_hull::<Keyboard>)
		.add_observer(setup_static_prop_with_convex_hull::<Mouse>)
		.add_observer(setup_dynamic_prop_with_convex_hull::<PackageMedium>)
		.add_observer(setup_dynamic_prop_with_convex_hull::<PackageSmall>);

	app.add_observer(setup_break_room);

	app.add_observer(setup_nonphysical_prop::<IvyPart8>)
		.add_observer(setup_nonphysical_prop::<SmallDoorSign1>);

	app.load_asset::<Gltf>(Crt::model_path())
		.load_asset::<Gltf>(Keyboard::model_path())
		.load_asset::<Gltf>(Mouse::model_path())
		.load_asset::<Gltf>(PackageMedium::model_path())
		.load_asset::<Gltf>(PackageSmall::model_path())
		.load_asset::<Gltf>(Grate::model_path())
		.load_asset::<Gltf>(Table::model_path())
		.load_asset::<Gltf>(Bookshelf::model_path())
		.load_asset::<Gltf>(Generator2::model_path())
		.load_asset::<Gltf>(BarrelLargeClosed::model_path())
		.load_asset::<Gltf>(Barrel01::model_path())
		.load_asset::<Gltf>(CrateSquare::model_path())
		.load_asset::<Gltf>(FenceBarsDecorativeSingle::model_path())
		.load_asset::<Gltf>(DoorStainedGlass::model_path())
		.load_asset::<Gltf>(IvyPart8::model_path())
		.load_asset::<Gltf>(SmallDoorSign1::model_path());
}

// generic dynamic props

// office

#[point_class(base(Transform, Visibility, YarnNode), model("models/office/crt.gltf"))]
#[component(on_add = Crt::on_add)]
#[derive(Default)]
pub(crate) struct Crt;

impl Crt {
	fn on_add(mut world: DeferredWorld, ctx: HookContext) {
		if world.is_scene_world() {
			return;
		}
		let model = world.get_asset_server().load_trenchbroom_model::<Self>();

		world.commands().entity(ctx.entity).insert((
			ColliderConstructorHierarchy::new(ColliderConstructor::ConvexHullFromMesh)
				.with_default_layers(CollisionLayers::new(
					[CollisionLayer::Default, CollisionLayer::Dialog],
					LayerMask::ALL,
				)),
			RigidBody::Static,
			SceneRoot(model),
		));
	}
}

#[point_class(
	base(InteractableEntity, Transform, Visibility),
	model("models/office/keyboard.gltf")
)]
pub(crate) struct Keyboard;

#[point_class(
	base(InteractableEntity, Transform, Visibility),
	model("models/office/mouse.gltf")
)]
pub(crate) struct Mouse;

// darkmod

#[point_class(
	base(InteractableEntity, Transform, Visibility),
	model("models/darkmod/containers/package_medium.gltf")
)]
pub(crate) struct PackageMedium;

#[point_class(
	base(InteractableEntity, Transform, Visibility),
	model("models/darkmod/containers/package_small.gltf")
)]
pub(crate) struct PackageSmall;

// generic static props
#[point_class(
	base(InteractableEntity, Transform, Visibility),
	model("models/darkmod/fireplace/grate.gltf")
)]
pub(crate) struct Grate;

#[point_class(
	base(InteractableEntity, Transform, Visibility),
	model("models/darkmod/furniture/tables/rtable1.gltf")
)]
pub(crate) struct Table;

#[point_class(
	base(InteractableEntity, Transform, Visibility),
	model("models/darkmod/furniture/shelves/bookshelf02.gltf")
)]
pub(crate) struct Bookshelf;

#[point_class(
	base(InteractableEntity, Transform, Visibility),
	model("models/darkmod/mechanical/generator2/generator2.gltf")
)]
pub(crate) struct Generator2;

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/containers/barrel_large_closed.gltf")
)]
pub(crate) struct BarrelLargeClosed;

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/containers/barrel01.gltf")
)]
pub(crate) struct Barrel01;

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/containers/crate_square.gltf")
)]
pub(crate) struct CrateSquare;

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/architecture/fencing/fence_bars_decorative01_single.gltf")
)]
pub(crate) struct FenceBarsDecorativeSingle;

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/architecture/doors/door_stained_glass_118x52.gltf")
)]
pub(crate) struct DoorStainedGlass;

// Generic non-physical props

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/nature/ivy_part08.gltf")
)]
pub(crate) struct IvyPart8;

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/decorative/signs/small_door_sign1.gltf")
)]
pub(crate) struct SmallDoorSign1;

struct BreakRoomTimer;

#[solid_class(base(Transform, Visibility))]
#[require(Sensor, CollisionEventsEnabled)]
pub(crate) struct BreakRoomSensor;

fn setup_break_room(add: On<Add, BreakRoomSensor>, mut commands: Commands) {
	commands
		.entity(add.entity)
		.insert(
			GenericTimer::<BreakRoomTimer>::new(Timer::new(
				Duration::from_secs(3),
				TimerMode::Once,
			))
			.with_active(false),
		)
		.observe(activate_timer)
		.observe(deactivate_timer)
		.observe(change_objective);
}

fn activate_timer(
	collision: On<CollisionStart>,
	mut timer: Query<&mut GenericTimer<BreakRoomTimer>>,
	objectives: Query<&ObjectiveEntity>,
	current_objective: Res<CurrentObjective>,
) -> Result<(), BevyError> {
	let Some(current_objective) = **current_objective else {
		return Ok(());
	};
	let Ok(ObjectiveEntity { targetname, .. }) = objectives.get(current_objective) else {
		return Ok(());
	};

	if targetname != "breakroom" {
		return Ok(());
	}

	let mut timer = timer.get_mut(collision.collider1)?;

	timer.set_active(true);

	Ok(())
}

fn deactivate_timer(
	collision: On<CollisionEnd>,
	mut timer: Query<&mut GenericTimer<BreakRoomTimer>>,
) -> Result<(), BevyError> {
	println!("Left breakroom");
	let mut timer = timer.get_mut(collision.collider1)?;

	timer.set_active(false);

	Ok(())
}

fn change_objective(
	_: On<TimerFinished<BreakRoomTimer>>,
	mut commands: Commands,
	objectives: Query<(Entity, &ObjectiveEntity)>,
	level_assets: Res<LevelAssets>,
) {
	if let Some((entity, _)) = objectives
		.iter()
		.find(|(_, ObjectiveEntity { targetname, .. })| targetname == "breakroom")
	{
		// TODO: Figure out why sound isn't playing
		commands.spawn((
			SamplePlayer {
				sample: level_assets.break_room_alarm.clone(),
				repeat_mode: RepeatMode::RepeatMultiple {
					num_times_to_repeat: 5,
				},
				..Default::default()
			},
			SpatialPool,
			Transform::from_xyz(7.0, 21., -2.),
		));
		commands.entity(entity).insert(ObjectiveCompleted);
		commands.spawn((
			Objective::new("Break's over"),
			ObjectiveEntity {
				targetname: "back_to_llm".into(),
				..Default::default()
			},
			related!(SubObjectives[Objective::new("Talk to LLManager")]),
		));
	}
}

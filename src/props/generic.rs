use crate::{
	asset_tracking::LoadResource, gameplay::interaction::InteractableObject, third_party::bevy_trenchbroom::GetTrenchbroomModelPath as _
};

use super::setup::*;
use bevy::prelude::*;
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

	app.add_observer(setup_static_prop_with_convex_hull::<Crt>)
		.add_observer(setup_static_prop_with_convex_hull::<Keyboard>)
		.add_observer(setup_static_prop_with_convex_hull::<Mouse>)
		.add_observer(setup_dynamic_prop_with_convex_hull::<PackageMedium>)
		.add_observer(setup_dynamic_prop_with_convex_hull::<PackageSmall>);

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

#[point_class(base(Transform, Visibility), model("models/office/crt.gltf"))]
pub(crate) struct Crt;

#[point_class(base(Transform, Visibility), model("models/office/keyboard.gltf"))]
pub(crate) struct Keyboard;

#[point_class(base(Transform, Visibility), model("models/office/mouse.gltf"))]
pub(crate) struct Mouse;

// darkmod

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/containers/package_medium.gltf")
)]
#[require(InteractableObject(Some("Open Medium! package".to_string())))]
pub(crate) struct PackageMedium;

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/containers/package_small.gltf")
)]
#[require(InteractableObject(Some("Open package".to_string())))]
pub(crate) struct PackageSmall;

// generic static props
#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/fireplace/grate.gltf")
)]
pub(crate) struct Grate;

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/furniture/tables/rtable1.gltf")
)]
#[require(InteractableObject(Some("Lick table".to_string())))]
pub(crate) struct Table;

#[point_class(
	base(Transform, Visibility),
	model("models/darkmod/furniture/shelves/bookshelf02.gltf")
)]
#[require(InteractableObject)]
pub(crate) struct Bookshelf;

#[point_class(
	base(Transform, Visibility),
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

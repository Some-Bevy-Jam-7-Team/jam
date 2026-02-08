use bevy::{
	math::bounding::{Aabb2d, BoundingVolume, RayCast2d},
	prelude::*,
};

pub(super) fn plugin(app: &mut App) {
	app.add_systems(Update, (extract_root_sizes, compute_layout, apply_root_positions).chain());
}

/// Component that marks that a node should be laid out on the HUD via the box stacking algorithm.
/// This should be placed on the root node of any "widget" you want in the HUD
/// 
/// # See also:
/// [`UiCanvas`]
#[derive(Component, Default)]
#[require(Node, RootWidgetSize, RootWidgetPosition)]
pub struct RootWidget;

/// Marker component for a singleton(!) entity into which the [`RootWidget`]s should be laid out.
/// The [`RootWidgets`] should be parented to the unique UiCanvas entity, and the UiCanvas should ideally span the whole screen
#[derive(Component, Default)]
#[require(Node)]
pub struct UiCanvas;

#[derive(Component, Default)]
struct RootWidgetPosition(Vec2);

#[derive(Component, Default)]
struct RootWidgetSize(Vec2);

fn extract_root_sizes(
	mut widget_query: Query<(&ComputedNode, &mut RootWidgetSize), Changed<ComputedNode>>,
) {
	for (node, mut widget_size) in widget_query.iter_mut() {
		// Only change when needed for future caching
		if Vec2::length_squared(widget_size.0 - node.content_size)
			> 0.0001 * Vec2::length_squared(node.content_size)
		{
			widget_size.0 = node.content_size;
		}
	}
}

fn compute_layout(
	window_size: Single<&ComputedNode, With<UiCanvas>>,
	mut widget_query: Query<(Entity, &mut RootWidgetPosition, &RootWidgetSize)>,
) {
	// TODO: Caching

	let window_half_size: Vec2 = window_size.size / 2.0;

	// println!("Running layout");

	let mut placed_rects: Vec<Aabb2d> = Vec::new();
	let mut rects_to_place: Vec<(Entity, Vec2)> = Vec::new();
	for (entity, _, &RootWidgetSize(size)) in widget_query.iter() {
		rects_to_place.push((entity, size));
	}
	// Sort into descending order by rectangle size
	rects_to_place
		.sort_by(|(_, a), (_, b)| f32::total_cmp(&f32::abs(b.x * b.y), &f32::abs(a.x * a.y)));

	// Pushes the rectangle as far as possible
	fn push_in_direction(
		half_size: Vec2,
		position: Vec2,
		direction: Dir2,
		other_rects: &Vec<Aabb2d>,
	) -> f32 {
		let mut t = f32::MAX;
		for rect in other_rects.iter() {
			if let Some(collision) =
				RayCast2d::new(position, direction, t).aabb_intersection_at(&rect.grow(half_size * 0.999))
			{
				t = f32::min(t, collision * 0.995);
			}
		}
		f32::max(t, 0.0)
	}

	fn get_max_bound(half_size: Vec2, direction: Dir2) -> f32 {
		let mut maximum = f32::MAX;
		if direction.x.abs() > 0.001 {
			maximum = maximum.min(half_size.x / direction.x.abs());
		}
		if direction.y.abs() > 0.001 {
			maximum = maximum.min(half_size.y / direction.y.abs());
		}
		maximum.max(0.0)
	}

	// println!("{window_half_size}");

	for (entity, size) in rects_to_place.into_iter() {
		let mut best_position = Vec2::ZERO;
		let half_size = size / 2.0;
		let available_space = window_half_size - half_size;

		for &initial_direction in &[
			Dir2::NORTH_EAST,
			Dir2::NORTH_WEST,
			Dir2::SOUTH_EAST,
			Dir2::SOUTH_WEST,
		] {
			let mut position = Vec2::ZERO;
			// Move diagonally
			let direction = initial_direction;
			let maximum_onscreen = get_max_bound(available_space, direction);
			let distance = f32::min(
				maximum_onscreen,
				push_in_direction(half_size, position, direction, &placed_rects),
			);
			position += direction * distance;
			// println!("Step:\tDir:{direction}\tDist:{distance}\tBound:{maximum_onscreen}\tPos:{position}");
			// Move horizontally
			let direction = Dir2::from_xy(1.0f32.copysign(initial_direction.x), 0.0).unwrap();
			let maximum_onscreen = get_max_bound(available_space - position.abs(), direction);
			let distance = f32::min(
				maximum_onscreen,
				push_in_direction(half_size, position, direction, &placed_rects),
			);
			position += direction * distance;
			// println!("Step:\tDir:{direction}\tDist:{distance}\tBound:{maximum_onscreen}\tPos:{position}");
			// Move vertically
			let direction = Dir2::from_xy(0.0, 1.0f32.copysign(initial_direction.y)).unwrap();
			let maximum_onscreen = get_max_bound(available_space - position.abs(), direction);
			let distance = f32::min(
				maximum_onscreen,
				push_in_direction(half_size, position, direction, &placed_rects),
			);
			position += direction * distance;
			// println!("Step:\tDir:{direction}\tDist:{distance}\tBound:{maximum_onscreen}\tPos:{position}");
			// if the new position is farther out than the previous one, we use it
			if position.length_squared() > best_position.length_squared() {
				best_position = position;
			}
		}

		placed_rects.push(Aabb2d::new(best_position, half_size));
		// println!("Result:\n{placed_rects:?}");
		if let Ok((_, mut position, _)) = widget_query.get_mut(entity) {
			position.0 = best_position;
		}
	}
}

fn apply_root_positions(
	window_size: Single<&ComputedNode, With<UiCanvas>>,
	mut widget_query: Query<(&mut Node, &RootWidgetPosition, &RootWidgetSize)>,
) {
	let window_size = window_size.size.max(Vec2::ONE);

	for (mut node, &RootWidgetPosition(position), &RootWidgetSize(size)) in widget_query.iter_mut() {
		node.position_type = PositionType::Absolute;
		let offset = 100.0 * (Vec2::splat(0.5) + (position - size / 2.0) / window_size);
		node.left = Val::Percent(offset.x);
		node.top = Val::Percent(offset.y);
		node.right = Val::Auto;
		node.bottom = Val::Auto;
	}
}

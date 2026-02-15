//! A high-level way to load collections of asset handles as resources.

use bevy::asset::{AssetEventSystems, UntypedAssetId};
use bevy::prelude::*;
use std::collections::HashMap;

pub(super) fn plugin(app: &mut App) {
	app.init_resource::<ResourceHandles>();
	app.add_systems(PreUpdate, load_resource_assets.after(AssetEventSystems));
}

pub(crate) trait LoadResource {
	/// This will load the [`Resource`] as an [`Asset`]. When all of its asset dependencies
	/// have been loaded, it will be inserted as a resource. This ensures that the resource only
	/// exists when the assets are ready.
	fn load_resource<T: Resource + Asset + Clone + FromWorld>(&mut self) -> &mut Self;
	fn load_asset<T: Asset>(&mut self, path: impl Into<String>) -> &mut Self;
}

impl LoadResource for App {
	fn load_resource<T: Resource + Asset + Clone + FromWorld>(&mut self) -> &mut Self {
		self.init_asset::<T>();
		let world = self.world_mut();
		let value = T::from_world(world);
		let assets = world.resource::<AssetServer>();
		let handle = assets.add(value);
		let mut handles = world.resource_mut::<ResourceHandles>();
		let id = handle.id();

		handles.waiting.insert(
			id.into(),
			(handle.untyped(), |world, handle| {
				let assets = world.resource::<Assets<T>>();
				if let Some(value) = assets.get(handle.id().typed::<T>()) {
					world.insert_resource(value.clone());
				}
			}),
		);
		self
	}

	fn load_asset<T: Asset>(&mut self, path: impl Into<String>) -> &mut Self {
		let handle: Handle<T> = self.world().load_asset(path.into());
		let mut handles = self.world_mut().resource_mut::<ResourceHandles>();
		let id = handle.id();
		handles
			.waiting
			.insert(id.into(), (handle.untyped(), |_world, _handle| {}));
		self
	}
}

/// A function that inserts a loaded resource.
type InsertLoadedResource = fn(&mut World, &UntypedHandle);

#[derive(Resource, Default)]
pub(crate) struct ResourceHandles {
	// Use a queue for waiting assets so they can be cycled through and moved to
	// `finished` one at a time.
	pub(crate) waiting: HashMap<UntypedAssetId, (UntypedHandle, InsertLoadedResource)>,
	pub(crate) finished: Vec<UntypedHandle>,
}

impl ResourceHandles {
	/// Returns true if all requested [`Asset`]s have finished loading and are available as [`Resource`]s.
	pub(crate) fn is_all_done(&self) -> bool {
		self.waiting.is_empty()
	}

	pub(crate) fn total_count(&self) -> usize {
		self.waiting.len() + self.finished.len()
	}

	pub(crate) fn finished_count(&self) -> usize {
		self.finished.len()
	}

	pub(crate) fn clear(&mut self) {
		self.waiting.clear();
		self.finished.clear();
	}
}

fn load_resource_assets(world: &mut World) {
	world.resource_scope(|world, mut resource_handles: Mut<ResourceHandles>| {
		world.resource_scope(|world, assets: Mut<AssetServer>| {
			let resource_ids: Vec<_> = resource_handles
				.waiting
				.iter()
				.filter_map(|(id, (handle, _))| {
					assets.is_loaded_with_dependencies(handle).then_some(*id)
				})
				.collect();

			debug!(
				"Processing {} loaded assets out of {} waiting",
				resource_ids.len(),
				resource_handles.waiting.len()
			);

			for id in resource_ids.into_iter() {
				let Some((handle, insert_fn)) = resource_handles.waiting.remove(&id) else {
					continue;
				};

				insert_fn(world, &handle);
				resource_handles.finished.push(handle);
			}
		});
	});
}

use std::marker::PhantomData;

use bevy::{
	ecs::{lifecycle::HookContext, world::DeferredWorld},
	prelude::*,
};

#[derive(EntityEvent)]
pub(crate) struct TimerFinished<T> {
	pub entity: Entity,
	_phantom: PhantomData<T>,
}

#[derive(Component, Reflect)]
#[component(on_add = GenericTimer::<T>::on_add)]
pub(crate) struct GenericTimer<T: Sync + Send + 'static> {
	pub active: bool,
	timer: Timer,
	_phantom: PhantomData<T>,
}
impl<T: Sync + Send + 'static> GenericTimer<T> {
	fn on_add(mut world: DeferredWorld, _: HookContext) {
		let total_timers = world
			.try_query_filtered::<(), With<GenericTimer<T>>>()
			.unwrap()
			.iter(&world)
			.count();

		if total_timers == 1 {
			world.commands().queue(|world: &mut World| {
				world.schedule_scope(Update, |_, scope| {
					scope.add_systems(tick_timer::<T>);
				});
			});
		}
	}

	pub(crate) fn new(timer: Timer) -> Self {
		Self {
			active: true,
			timer,
			_phantom: PhantomData,
		}
	}
	pub(crate) fn with_active(mut self, active: bool) -> Self {
		self.active = active;
		self
	}
	pub(crate) fn set_active(&mut self, active: bool) -> &mut Self {
		self.active = active;
		self
	}
}

fn tick_timer<T: Sync + Send + 'static>(
	mut timers: Query<(Entity, &mut GenericTimer<T>)>,
	time: Res<Time>,
	mut commands: Commands,
) {
	for (entity, mut timer) in timers.iter_mut().filter(|(_, timer)| timer.active) {
		timer.timer.tick(time.delta());
		dbg!(timer.timer.elapsed_secs());

		if timer.timer.just_finished() {
			commands.trigger(TimerFinished {
				entity,
				_phantom: PhantomData::<T>,
			});
		}
	}
}

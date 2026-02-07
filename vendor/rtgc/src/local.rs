use core::{any::Any, cell::UnsafeCell, ops::Deref, ptr::NonNull};

#[cfg(all(feature = "std", not(feature = "bevy_platform")))]
use std::sync::Arc;

#[cfg(feature = "bevy_platform")]
use bevy_platform::{prelude::Box, sync::Arc};

use crate::{
    ArcGc, Collector, CollectorState, OwnedGc, OwnedGcUnsized, OwnedGcWrapper, StrongCount,
};

// TODO: Add a single-threaded variant of LocalRtGc that doesn't rely
// on Mutex?

/// A simple garbage collector which collects resources dropped on a
/// realtime thread and safely deallocates them on another thread.
///
/// Unlike [`GlobalRtGc`](crate::GlobalRtGc), this version does not
/// use global statics.
///
/// The performance characteristics of the [`ArcGc`], [`OwnedGc`], and
/// [`OwnedGcUnsized`] smart pointers are equivalant to [`Arc`]  when
/// reading (but constructing them is a bit more expensive).
///
/// # Example
///
/// ```rust
/// # use rtgc::*;
/// # use std::time::Duration;
/// let mut collector = LocalRtGc::new();
/// let handle = collector.handle();
///
/// let value_1 = ArcGc::new_loc(String::from("foo"), &handle);
/// // Same as `ArcGc` but for `!Sync` data.
/// let value_2 = OwnedGc::new_loc(String::from("bar"), &handle);
///
/// // A simulated "realtime thread"
/// let rt_thread = std::thread::spawn(move || {
///     std::thread::sleep(Duration::from_millis(15));
///
///     // Dropping the values on the realtime thread is realtime-safe
///     // because the contents are automatically collected and
///     // deallocated on a separate non-realtime thread.
///     let _ = value_1;
///     let _ = value_2;
/// });
///
/// // A simulated update loop on the main thread
/// for _ in 0..4 {
///     // Call `collector.collect()` periodically to deallocate any
///     // resources that were dropped on the realtime thread.
///     collector.collect();
///
///     std::thread::sleep(Duration::from_millis(15));
/// }
/// ```
pub struct LocalRtGc {
    shared_state: Arc<CollectorState>,
}

impl LocalRtGc {
    pub fn new() -> Self {
        Self {
            shared_state: Arc::new(CollectorState::new()),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            shared_state: Arc::new(CollectorState::with_capacity(capacity)),
        }
    }

    pub fn handle(&self) -> LocalRtGcHandle {
        LocalRtGcHandle {
            shared_state: Arc::clone(&self.shared_state),
        }
    }

    /// Returns `true` if any allocation has been dropped since the
    /// collection cycle.
    pub fn any_dropped(&self) -> bool {
        self.shared_state.any_dropped()
    }

    /// The total number of allocations currently active in this
    /// garbage collector.
    pub fn num_allocations(&self) -> usize {
        self.shared_state.num_allocations()
    }

    /// Collect and drop all unused [`ArcGc`] resources.
    pub fn collect(&mut self) {
        self.shared_state.collect();
    }

    /// The total number of active references to this garbage collector.
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.shared_state)
    }
}

impl Collector for LocalRtGc {
    fn register<T>(&self, data: Arc<T>)
    where
        T: ?Sized + Send + Sync + 'static,
        Arc<T>: StrongCount,
    {
        self.shared_state.register(data);
    }

    fn remove<T>(&self, data: &Arc<T>)
    where
        T: ?Sized + Send + Sync + 'static,
        Arc<T>: StrongCount,
    {
        self.shared_state.remove(data);
    }
}

impl Default for LocalRtGc {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for LocalRtGc {
    fn clone(&self) -> Self {
        Self {
            shared_state: Arc::clone(&self.shared_state),
        }
    }
}

/// A handle to a [`LocalRtGc`] garbage collector which collects
/// resources dropped on a realtime thread and safely deallocates
/// them on another thread.
pub struct LocalRtGcHandle {
    shared_state: Arc<CollectorState>,
}

impl LocalRtGcHandle {
    /// The total number of allocations currently active in this
    /// garbage collector.
    pub fn num_allocations(&self) -> usize {
        self.shared_state.num_allocations()
    }

    /// Returns `true` if any allocation has been dropped since the
    /// last call to [`LocalRtGc::collect`].
    pub fn any_dropped(&self) -> bool {
        self.shared_state.any_dropped()
    }

    /// The total number of active references to this garbage collector.
    pub fn strong_count(&self) -> usize {
        Arc::strong_count(&self.shared_state)
    }
}

impl Clone for LocalRtGcHandle {
    fn clone(&self) -> Self {
        Self {
            shared_state: Arc::clone(&self.shared_state),
        }
    }
}

impl<T: Send + Sync + 'static> ArcGc<T, LocalRtGc> {
    /// Construct a new [`ArcGc`].
    ///
    /// ```
    /// # use rtgc::*;
    /// let collector = LocalRtGc::new();
    /// let handle = collector.handle();
    ///
    /// let value: ArcGc<String, LocalRtGc> = ArcGc::new_loc(String::from("foo"), &handle);
    /// ```
    pub fn new_loc(value: T, handle: &LocalRtGcHandle) -> Self {
        let data = Arc::new(value);

        handle.shared_state.register(Arc::clone(&data));

        Self {
            data,
            collector: LocalRtGc {
                shared_state: Arc::clone(&handle.shared_state),
            },
        }
    }
}

impl<T: ?Sized + Send + Sync + 'static> ArcGc<T, LocalRtGc> {
    /// Construct a new [`ArcGc`] with _unsized_ data, such as `[T]` or `dyn Trait`.
    ///
    /// ```
    /// # use rtgc::*;
    /// # use std::sync::Arc;
    /// let collector = LocalRtGc::new();
    /// let handle = collector.handle();
    ///
    /// let value_1: ArcGc<[f32], LocalRtGc> = ArcGc::new_unsized_loc(
    ///     || Arc::<[f32]>::from([1.0, 2.0, 3.0]),
    ///     &handle
    /// );
    ///
    /// trait Foo: Send + Sync {}
    /// struct Bar {}
    /// impl Foo for Bar {}
    ///
    /// let value_2: ArcGc<dyn Foo, LocalRtGc> = ArcGc::new_unsized_loc(
    ///     || Arc::new(Bar {}) as Arc<dyn Foo>,
    ///     &handle
    /// );
    /// ```
    pub fn new_unsized_loc(f: impl FnOnce() -> Arc<T>, handle: &LocalRtGcHandle) -> Self {
        let data = f();

        handle.shared_state.register(Arc::clone(&data));

        Self {
            data,
            collector: LocalRtGc {
                shared_state: Arc::clone(&handle.shared_state),
            },
        }
    }
}

impl ArcGc<dyn Any + Send + Sync + 'static, LocalRtGc> {
    /// Construct a type-erased [`ArcGc`].
    ///
    /// ```
    /// # use rtgc::*;
    /// # use std::sync::Arc;
    /// # use std::any::Any;
    /// let collector = LocalRtGc::new();
    /// let handle = collector.handle();
    ///
    /// let value: ArcGc<dyn Any + Send + Sync + 'static, LocalRtGc> =
    ///     ArcGc::new_any_loc(String::new(), &handle);
    /// ```
    pub fn new_any_loc<T: Send + Sync + 'static>(value: T, handle: &LocalRtGcHandle) -> Self {
        ArcGc::<dyn Any + Send + Sync + 'static, LocalRtGc>::new_unsized_loc(
            || {
                let a: Arc<dyn Any + Send + Sync + 'static> = Arc::new(value);
                a
            },
            handle,
        )
    }
}

impl<T: Send + 'static> OwnedGc<T, LocalRtGc> {
    /// Construct a new [`OwnedGc`].
    pub fn new_loc(value: T, handle: &LocalRtGcHandle) -> Self {
        Self {
            data: ArcGc::<_, LocalRtGc>::new_loc(OwnedGcWrapper(UnsafeCell::new(value)), handle),
        }
    }
}

impl<T: ?Sized + Send + 'static> OwnedGcUnsized<T, LocalRtGc> {
    /// Construct a new [`OwnedGcUnsized`] with _unsized_ data, such as `[T]` or `dyn Trait`.
    ///
    /// ```
    /// # use rtgc::*;
    /// # use std::sync::Arc;
    /// let collector = LocalRtGc::new();
    /// let handle = collector.handle();
    ///
    /// let value_1: OwnedGcUnsized<[f32], LocalRtGc> = OwnedGcUnsized::new_unsized_loc(
    ///     vec![0.0, 1.0, 2.0].into_boxed_slice(),
    ///     &handle
    /// );
    ///
    /// trait Foo: Send {}
    /// struct Bar {}
    /// impl Foo for Bar {}
    ///
    /// let value_2: OwnedGcUnsized<dyn Foo, LocalRtGc> = OwnedGcUnsized::new_unsized_loc(
    ///     Box::new(Bar {}),
    ///     &handle
    /// );
    /// ```
    pub fn new_unsized_loc(data: Box<T>, handle: &LocalRtGcHandle) -> Self {
        let pinned = Box::into_pin(data);
        let ptr = NonNull::from_ref(pinned.deref());

        Self {
            _owned: OwnedGc::<_, LocalRtGc>::new_loc(pinned, handle),
            ptr,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_local_drop() {
        let mut collector = LocalRtGc::new();
        let handle = collector.handle();

        assert_eq!(collector.num_allocations(), 0);

        let value = ArcGc::new_loc(1, &handle);

        assert_eq!(collector.num_allocations(), 1);
        assert_eq!(collector.any_dropped(), false);

        collector.collect();

        assert_eq!(collector.num_allocations(), 1);
        assert_eq!(collector.any_dropped(), false);

        drop(value);

        // Even though we've dropped the "last reference,"
        // the inner drop won't be called until we do garbage
        // collection.
        assert_eq!(collector.num_allocations(), 1);
        assert_eq!(collector.any_dropped(), true);

        collector.collect();

        assert_eq!(collector.num_allocations(), 0);
        assert_eq!(collector.any_dropped(), false);
    }

    #[test]
    fn test_local_unsized() {
        let mut collector = LocalRtGc::new();
        let handle = collector.handle();

        assert_eq!(collector.num_allocations(), 0);

        let value = ArcGc::new_unsized_loc(|| Arc::<[i32]>::from([1, 2, 3]), &handle);

        assert_eq!(collector.num_allocations(), 1);
        assert_eq!(collector.any_dropped(), false);

        collector.collect();

        assert_eq!(collector.num_allocations(), 1);
        assert_eq!(collector.any_dropped(), false);

        drop(value);

        assert_eq!(collector.num_allocations(), 1);
        assert_eq!(collector.any_dropped(), true);

        collector.collect();

        assert_eq!(collector.num_allocations(), 0);
        assert_eq!(collector.any_dropped(), false);
    }
}

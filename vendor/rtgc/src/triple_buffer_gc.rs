use core::{
    cell::{Ref, RefCell, RefMut},
    fmt::Debug,
};

use crate::{ArcGc, Collector, GlobalRtGc, OwnedGc};

/// Create a new triple buffer that can be used to send [`ArcGc`] data to a realtime
/// thread.
///
/// This is similar to using an `spsc` ring buffer to send [`ArcGc`] data, except
/// that the consumer only reads the latest value that was pushed by the producer.
/// The producer can overwrite previously pushed data without waiting for the
/// consumer to read the previously pushed data.
///
/// And unlike using the
/// [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/index.html)
/// crate directly to sync data, this data structure does not use 3x the memory of
/// the data. Rather, the internal triple buffer only stores [`ArcGc`] pointers.
///
/// The downside is that the producer must allocate a new data struct every time it
/// wants to send new data, and the old data must later be deallocated in the garbage
/// collector. As such, this is designed for use cases where writes are infrequent,
/// and is especially useful when the data itself contains heap-allocated data (i.e.
/// `Vec`s).
///
/// This uses the
/// [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/index.html)
/// crate internally.
///
/// # Example
///
/// ```rust
/// # use rtgc::*;
/// # use std::time::Duration;
///
/// #[derive(Debug)]
/// struct MyStateStruct {
///     text: String,
///     generation: usize,
/// }
///
/// let (mut input, mut output) = arc_gc_triple_buffer(ArcGc::new(MyStateStruct {
///     text: String::from("foo"),
///     generation: 0,
/// }));
///
/// // A simulated "realtime thread"
/// let rt_thread = std::thread::spawn(move || {
///     for _ in 0..4 {
///         // Read the latest value from the buffer.
///         let latest = output.read();
///         println!("{:?}", latest);
///
///         std::thread::sleep(Duration::from_millis(3));
///     }
/// });
///
/// // A simulated update loop on the main thread
/// for i in 1..5 {
///     // Send a new `ArcGc` to the realtime thread to use.
///     input.write(ArcGc::new(MyStateStruct {
///         text: String::from("bar"),
///         generation: i,
///     }));
///
///     // Call `GlobalRtGc::collect()` periodically to deallocate
///     // any resources that were dropped on the realtime thread.
///     GlobalRtGc::collect();
///
///     std::thread::sleep(Duration::from_millis(5));
/// }
/// ```
pub fn arc_gc_triple_buffer<T: ?Sized + Send + Sync + 'static, C: Collector + Clone>(
    initial: ArcGc<T, C>,
) -> (ArcGcInput<T, C>, ArcGcOutput<T, C>) {
    let (input, output) = triple_buffer::triple_buffer(&initial);
    (ArcGcInput { input }, ArcGcOutput { output })
}

/// The input (producer) end to a triple buffer that is used to send [`ArcGc`]
/// data to a realtime thread.
///
/// This is similar to using an `spsc` ring buffer to send [`ArcGc`] data, except
/// that the consumer only reads the latest value that was pushed by the producer.
/// The producer can overwrite previously pushed data without waiting for the
/// consumer to read the previously pushed data.
///
/// And unlike using the
/// [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/index.html)
/// crate directly to sync data, this data structure does not use 3x the memory of
/// the data. Rather, the internal triple buffer only stores [`ArcGc`] pointers.
///
/// The downside is that the producer must allocate a new data struct every time it
/// wants to send new data, and the old data must later be deallocated in the garbage
/// collector. As such, this is designed for use cases where writes are infrequent,
/// and is especially useful when the data itself contains heap-allocated data (i.e.
/// `Vec`s).
///
/// This uses the
/// [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/index.html)
/// crate internally.
///
/// # Example
///
/// ```rust
/// # use rtgc::*;
/// # use std::time::Duration;
///
/// #[derive(Debug)]
/// struct MyStateStruct {
///     text: String,
///     generation: usize,
/// }
///
/// let (mut input, mut output) = arc_gc_triple_buffer(ArcGc::new(MyStateStruct {
///     text: String::from("foo"),
///     generation: 0,
/// }));
///
/// // A simulated "realtime thread"
/// let rt_thread = std::thread::spawn(move || {
///     for _ in 0..4 {
///         // Read the latest value from the buffer.
///         let latest = output.read();
///         println!("{:?}", latest);
///
///         std::thread::sleep(Duration::from_millis(3));
///     }
/// });
///
/// // A simulated update loop on the main thread
/// for i in 1..5 {
///     // Send a new `ArcGc` to the realtime thread to use.
///     input.write(ArcGc::new(MyStateStruct {
///         text: String::from("bar"),
///         generation: i,
///     }));
///
///     // Call `GlobalRtGc::collect()` periodically to deallocate
///     // any resources that were dropped on the realtime thread.
///     GlobalRtGc::collect();
///
///     std::thread::sleep(Duration::from_millis(5));
/// }
/// ```
pub struct ArcGcInput<T: ?Sized + Send + Sync + 'static, C: Collector = GlobalRtGc> {
    input: triple_buffer::Input<ArcGc<T, C>>,
}

impl<T: ?Sized + Send + Sync + 'static, C: Collector> ArcGcInput<T, C> {
    /// Send a new [`ArcGc`] data struct to the [`ArcGcOutput`] counterpart.
    ///
    /// The [`ArcGcOutput`] counterpart only reads the latest data that has been
    /// written. Previously pushed data may be overwritten before the output has
    /// a chance to read it.
    ///
    /// This method is realtime safe (though note creating a new [`ArcGc`] is NOT
    /// realtime safe).
    pub fn write(&mut self, value: ArcGc<T, C>) {
        self.input.write(value);
    }
}

impl<T: ?Sized + Send + Sync + 'static, C: Collector> Debug for ArcGcInput<T, C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut f = f.debug_struct("ArcGcInput");
        f.field("consumed", &self.input.consumed());
        f.finish()
    }
}

/// The output (consumer) end to a triple buffer that is used to send [`ArcGc`]
/// data to a realtime thread.
///
/// This is similar to using an `spsc` ring buffer to send [`ArcGc`] data, except
/// that the consumer only reads the latest value that was pushed by the producer.
/// The producer can overwrite previously pushed data without waiting for the
/// consumer to read the previously pushed data.
///
/// And unlike using the
/// [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/index.html)
/// crate directly to sync data, this data structure does not use 3x the memory of
/// the data. Rather, the internal triple buffer only stores [`ArcGc`] pointers.
///
/// The downside is that the producer must allocate a new data struct every time it
/// wants to send new data, and the old data must later be deallocated in the garbage
/// collector. As such, this is designed for use cases where writes are infrequent,
/// and is especially useful when the data itself contains heap-allocated data (i.e.
/// `Vec`s).
///
/// This uses the
/// [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/index.html)
/// crate internally.
///
/// # Example
///
/// ```rust
/// # use rtgc::*;
/// # use std::time::Duration;
///
/// #[derive(Debug)]
/// struct MyStateStruct {
///     text: String,
///     generation: usize,
/// }
///
/// let (mut input, mut output) = arc_gc_triple_buffer(ArcGc::new(MyStateStruct {
///     text: String::from("foo"),
///     generation: 0,
/// }));
///
/// // A simulated "realtime thread"
/// let rt_thread = std::thread::spawn(move || {
///     for _ in 0..4 {
///         // Read the latest value from the buffer.
///         let latest = output.read();
///         println!("{:?}", latest);
///
///         std::thread::sleep(Duration::from_millis(3));
///     }
/// });
///
/// // A simulated update loop on the main thread
/// for i in 1..5 {
///     // Send a new `ArcGc` to the realtime thread to use.
///     input.write(ArcGc::new(MyStateStruct {
///         text: String::from("bar"),
///         generation: i,
///     }));
///
///     // Call `GlobalRtGc::collect()` periodically to deallocate
///     // any resources that were dropped on the realtime thread.
///     GlobalRtGc::collect();
///
///     std::thread::sleep(Duration::from_millis(5));
/// }
/// ```
pub struct ArcGcOutput<T: ?Sized + Send + Sync + 'static, C: Collector = GlobalRtGc> {
    output: triple_buffer::Output<ArcGc<T, C>>,
}

impl<T: ?Sized + Send + Sync + 'static, C: Collector> ArcGcOutput<T, C> {
    /// Get the latest [`ArcGc`] data.
    ///
    /// This method is realtime safe.
    pub fn read(&mut self) -> &ArcGc<T, C> {
        self.output.read()
    }

    /// Get the current data in the buffer without checking to see if a newer
    /// value exists.
    ///
    /// This method is realtime safe.
    pub fn peek(&self) -> &ArcGc<T, C> {
        self.output.peek_output_buffer()
    }
}

impl<T: Debug + ?Sized + Send + Sync + 'static, C: Collector> Debug for ArcGcOutput<T, C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(self.peek(), f)
    }
}

/// Create a new triple buffer that can be used to send [`OwnedGc`] data to a
/// realtime thread.
///
/// This is similar to using an `spsc` ring buffer to send [`OwnedGc`] data, except
/// that the consumer only reads the latest value that was pushed by the producer.
/// The producer can overwrite previously pushed data without waiting for the
/// consumer to read the previously pushed data.
///
/// And unlike using the
/// [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/index.html)
/// crate directly to sync data, this data structure does not use 3x the memory of
/// the data. Rather, the internal triple buffer only stores [`OwnedGc`] pointers.
///
/// The downside is that the producer must allocate a new data struct every time it
/// wants to send new data, and the old data must later be deallocated in the garbage
/// collector. As such, this is designed for use cases where writes are infrequent,
/// and is especially useful when the data itself contains heap-allocated data (i.e.
/// `Vec`s).
///
/// This uses the
/// [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/index.html)
/// crate internally.
///
/// # Example
///
/// ```rust
/// # use rtgc::*;
/// # use std::time::Duration;
///
/// #[derive(Debug)]
/// struct MyStateStruct {
///     text: String,
///     scratch_buffer: Vec<f32>,
///     generation: usize,
/// }
///
/// let (mut input, mut output) = owned_gc_triple_buffer(OwnedGc::new(MyStateStruct {
///     text: String::from("foo"),
///     scratch_buffer: Vec::with_capacity(10),
///     generation: 0,
/// }));
///
/// // A simulated "realtime thread"
/// let rt_thread = std::thread::spawn(move || {
///     for _ in 0..4 {
///         {
///             // Read the latest value from the buffer.
///             let latest = output.read();
///             println!("{:?}", latest);
///         }
///
///         {
///             // Get a mutable reference to the latest value in the buffer.
///             // (Note, any changes will be discarded if the input writes new
///             // data, so this is more for accessing pre-allocated data for
///             // scratch buffers, or for taking ownership of the data.)
///             let mut latest = output.read_mut();
///             latest.scratch_buffer.clear();
///             latest.scratch_buffer.push(1.0);
///         }
///
///         std::thread::sleep(Duration::from_millis(3));
///     }
/// });
///
/// // A simulated update loop on the main thread
/// for i in 1..5 {
///     // Send a new `OwnedGc` to the realtime thread to use.
///     input.write(OwnedGc::new(MyStateStruct {
///         text: String::from("bar"),
///         scratch_buffer: Vec::with_capacity(10),
///         generation: i,
///     }));
///
///     // Call `GlobalRtGc::collect()` periodically to deallocate
///     // any resources that were dropped on the realtime thread.
///     GlobalRtGc::collect();
///
///     std::thread::sleep(Duration::from_millis(5));
/// }
/// ```
pub fn owned_gc_triple_buffer<T: ?Sized + Send + 'static, C: Collector>(
    initial: OwnedGc<T, C>,
) -> (OwnedGcInput<T, C>, OwnedGcOutput<T, C>) {
    let (mut input, output) = triple_buffer::triple_buffer(&OwnedGcCloneWrapper {
        owned: RefCell::new(None),
    });
    input.write(OwnedGcCloneWrapper {
        owned: RefCell::new(Some(initial)),
    });

    (OwnedGcInput { input }, OwnedGcOutput { output })
}

/// The input (producer) end to a triple buffer that is used to send [`OwnedGc`]
/// data to a realtime thread.
///
/// This is similar to using an `spsc` ring buffer to send [`OwnedGc`] data, except
/// that the consumer only reads the latest value that was pushed by the producer.
/// The producer can overwrite previously pushed data without waiting for the
/// consumer to read the previously pushed data.
///
/// And unlike using the
/// [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/index.html)
/// crate directly to sync data, this data structure does not use 3x the memory of
/// the data. Rather, the internal triple buffer only stores [`OwnedGc`] pointers.
///
/// The downside is that the producer must allocate a new data struct every time it
/// wants to send new data, and the old data must later be deallocated in the garbage
/// collector. As such, this is designed for use cases where writes are infrequent,
/// and is especially useful when the data itself contains heap-allocated data (i.e.
/// `Vec`s).
///
/// This uses the
/// [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/index.html)
/// crate internally.
///
/// # Example
///
/// ```rust
/// # use rtgc::*;
/// # use std::time::Duration;
///
/// #[derive(Debug)]
/// struct MyStateStruct {
///     text: String,
///     scratch_buffer: Vec<f32>,
///     generation: usize,
/// }
///
/// let (mut input, mut output) = owned_gc_triple_buffer(OwnedGc::new(MyStateStruct {
///     text: String::from("foo"),
///     scratch_buffer: Vec::with_capacity(10),
///     generation: 0,
/// }));
///
/// // A simulated "realtime thread"
/// let rt_thread = std::thread::spawn(move || {
///     for _ in 0..4 {
///         {
///             // Read the latest value from the buffer.
///             let latest = output.read();
///             println!("{:?}", latest);
///         }
///
///         {
///             // Get a mutable reference to the latest value in the buffer.
///             // (Note, any changes will be discarded if the input writes new
///             // data, so this is more for accessing pre-allocated data for
///             // scratch buffers, or for taking ownership of the data.)
///             let mut latest = output.read_mut();
///             latest.scratch_buffer.clear();
///             latest.scratch_buffer.push(1.0);
///         }
///
///         std::thread::sleep(Duration::from_millis(3));
///     }
/// });
///
/// // A simulated update loop on the main thread
/// for i in 1..5 {
///     // Send a new `OwnedGc` to the realtime thread to use.
///     input.write(OwnedGc::new(MyStateStruct {
///         text: String::from("bar"),
///         scratch_buffer: Vec::with_capacity(10),
///         generation: i,
///     }));
///
///     // Call `GlobalRtGc::collect()` periodically to deallocate
///     // any resources that were dropped on the realtime thread.
///     GlobalRtGc::collect();
///
///     std::thread::sleep(Duration::from_millis(5));
/// }
/// ```
pub struct OwnedGcInput<T: ?Sized + Send + 'static, C: Collector = GlobalRtGc> {
    input: triple_buffer::Input<OwnedGcCloneWrapper<T, C>>,
}

impl<T: ?Sized + Send + 'static, C: Collector> OwnedGcInput<T, C> {
    /// Send a new [`OwnedGc`] data struct to the [`OwnedGcOutput`] counterpart.
    ///
    /// The [`OwnedGcOutput`] counterpart only reads the latest data that has been
    /// written. Previously pushed data may be overwritten before the output has
    /// a chance to read it.
    ///
    /// This method is realtime safe (though note creating a new [`OwnedGc`] is NOT
    /// realtime safe).
    pub fn write(&mut self, value: OwnedGc<T, C>) {
        self.input.write(OwnedGcCloneWrapper {
            owned: RefCell::new(Some(value)),
        });
    }
}

impl<T: ?Sized + Send + Sync + 'static, C: Collector> Debug for OwnedGcInput<T, C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut f = f.debug_struct("OwnedGcInput");
        f.field("consumed", &self.input.consumed());
        f.finish()
    }
}

/// The output (consumer) end to a triple buffer that is used to send [`OwnedGc`]
/// data to a realtime thread.
///
/// This is similar to using an `spsc` ring buffer to send [`OwnedGc`] data, except
/// that the consumer only reads the latest value that was pushed by the producer.
/// The producer can overwrite previously pushed data without waiting for the
/// consumer to read the previously pushed data.
///
/// And unlike using the
/// [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/index.html)
/// crate directly to sync data, this data structure does not use 3x the memory of
/// the data. Rather, the internal triple buffer only stores [`OwnedGc`] pointers.
///
/// The downside is that the producer must allocate a new data struct every time it
/// wants to send new data, and the old data must later be deallocated in the garbage
/// collector. As such, this is designed for use cases where writes are infrequent,
/// and is especially useful when the data itself contains heap-allocated data (i.e.
/// `Vec`s).
///
/// This uses the
/// [`triple_buffer`](https://docs.rs/triple_buffer/latest/triple_buffer/index.html)
/// crate internally.
///
/// # Example
///
/// ```rust
/// # use rtgc::*;
/// # use std::time::Duration;
///
/// #[derive(Debug)]
/// struct MyStateStruct {
///     text: String,
///     scratch_buffer: Vec<f32>,
///     generation: usize,
/// }
///
/// let (mut input, mut output) = owned_gc_triple_buffer(OwnedGc::new(MyStateStruct {
///     text: String::from("foo"),
///     scratch_buffer: Vec::with_capacity(10),
///     generation: 0,
/// }));
///
/// // A simulated "realtime thread"
/// let rt_thread = std::thread::spawn(move || {
///     for _ in 0..4 {
///         {
///             // Read the latest value from the buffer.
///             let latest = output.read();
///             println!("{:?}", latest);
///         }
///
///         {
///             // Get a mutable reference to the latest value in the buffer.
///             // (Note, any changes will be discarded if the input writes new
///             // data, so this is more for accessing pre-allocated data for
///             // scratch buffers, or for taking ownership of the data.)
///             let mut latest = output.read_mut();
///             latest.scratch_buffer.clear();
///             latest.scratch_buffer.push(1.0);
///         }
///
///         std::thread::sleep(Duration::from_millis(3));
///     }
/// });
///
/// // A simulated update loop on the main thread
/// for i in 1..5 {
///     // Send a new `OwnedGc` to the realtime thread to use.
///     input.write(OwnedGc::new(MyStateStruct {
///         text: String::from("bar"),
///         scratch_buffer: Vec::with_capacity(10),
///         generation: i,
///     }));
///
///     // Call `GlobalRtGc::collect()` periodically to deallocate
///     // any resources that were dropped on the realtime thread.
///     GlobalRtGc::collect();
///
///     std::thread::sleep(Duration::from_millis(5));
/// }
/// ```
pub struct OwnedGcOutput<T: ?Sized + Send + 'static, C: Collector = GlobalRtGc> {
    output: triple_buffer::Output<OwnedGcCloneWrapper<T, C>>,
}

impl<T: ?Sized + Send + 'static, C: Collector> OwnedGcOutput<T, C> {
    /// Get an immutable reference to the latest [`OwnedGc`] data.
    ///
    /// This method is realtime safe.
    pub fn read<'a>(&'a mut self) -> Ref<'a, T> {
        Ref::map(RefCell::borrow(&self.output.read().owned), |data| {
            data.as_ref().unwrap().get()
        })
    }

    /// Get a mutable reference to the latest [`OwnedGc`] data.
    ///
    /// (Note, any changes will be discarded if the input writes new data,
    /// so this is more for accessing pre-allocated data for scratch
    /// buffers and such, or for taking ownership of the data.)
    ///
    /// This method is realtime safe.
    pub fn read_mut<'a>(&'a mut self) -> RefMut<'a, T> {
        RefMut::map(RefCell::borrow_mut(&self.output.read().owned), |data| {
            data.as_mut().unwrap().get_mut()
        })
    }

    /// Get an immutable reference to the value in the buffer without checking
    /// to see if a newer value exists.
    ///
    /// This method is realtime safe.
    pub fn peek<'a>(&'a self) -> Ref<'a, T> {
        Ref::map(
            RefCell::borrow(&self.output.peek_output_buffer().owned),
            |data| data.as_ref().unwrap().get(),
        )
    }
}

struct OwnedGcCloneWrapper<T: ?Sized + Send + 'static, C: Collector> {
    owned: RefCell<Option<OwnedGc<T, C>>>,
}

impl<T: ?Sized + Send + 'static, C: Collector> Clone for OwnedGcCloneWrapper<T, C> {
    fn clone(&self) -> Self {
        Self {
            owned: RefCell::new(self.owned.borrow_mut().take()),
        }
    }
}

impl<T: Debug + ?Sized + Send + Sync + 'static, C: Collector> Debug for OwnedGcOutput<T, C> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&*self.peek(), f)
    }
}

#[cfg(test)]
mod test {
    use crate::LocalRtGc;

    use super::*;

    #[test]
    #[cfg(feature = "local_collector")]
    fn test_arc_gc_triple_buffer() {
        let mut collector = LocalRtGc::new();
        let handle = collector.handle();

        assert_eq!(collector.num_allocations(), 0);

        let (mut input, mut output) =
            arc_gc_triple_buffer(ArcGc::new_loc(String::from("aaaa"), &handle));

        assert_eq!(output.read().as_str(), "aaaa");
        assert_eq!(output.read().as_str(), "aaaa");

        input.write(ArcGc::new_loc(String::from("bbbb"), &handle));

        assert_eq!(output.read().as_str(), "bbbb");
        assert_eq!(output.read().as_str(), "bbbb");

        collector.collect();
    }

    #[test]
    #[cfg(feature = "local_collector")]
    fn test_owned_gc_triple_buffer() {
        let mut collector = LocalRtGc::new();
        let handle = collector.handle();

        assert_eq!(collector.num_allocations(), 0);

        let (mut input, mut output) =
            owned_gc_triple_buffer(OwnedGc::new_loc(String::from("aaaa"), &handle));

        assert_eq!(output.read().as_str(), "aaaa");
        assert_eq!(output.read().as_str(), "aaaa");

        {
            let mut str_mut = output.read_mut();
            assert_eq!(str_mut.as_str(), "aaaa");
            *str_mut = String::from("bbbb");
        }

        assert_eq!(output.read().as_str(), "bbbb");

        input.write(OwnedGc::new_loc(String::from("cccc"), &handle));

        {
            let str_mut = output.read_mut();
            assert_eq!(str_mut.as_str(), "cccc");
        }

        assert_eq!(output.read().as_str(), "cccc");

        collector.collect();
    }
}

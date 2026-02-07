#![doc = include_str!("../README.md")]

pub use smart_default;
pub use once_cell;
pub use parking_lot;

/// Extra std imports that i use a lot.
pub mod std_prelude {
	pub use std::path::{Path, PathBuf};
	pub use std::fs;
	pub use std::io;
	pub use std::thread;
	pub use std::collections::{HashMap, BTreeMap, HashSet, BTreeSet};
	pub use std::sync::Arc;
	pub use std::sync::atomic::*;
	pub use std::error::Error;
	pub use std::borrow::Cow;
	pub use std::time::{Duration, Instant};
	pub use std::mem;
	pub use std::fmt;
	pub use std::env;
	pub use std::process;
	pub use std::any::*;
	pub use std::f32::consts::{
		PI,
		FRAC_PI_2,
		FRAC_PI_3,
		FRAC_PI_4,
		FRAC_PI_6,
		FRAC_PI_8,
	};
	pub use std::f64::consts::{
		PI as PI_F64,
		FRAC_PI_2 as FRAC_PI_2_F64,
		FRAC_PI_3 as FRAC_PI_3_F64,
		FRAC_PI_4 as FRAC_PI_4_F64,
		FRAC_PI_6 as FRAC_PI_6_F64,
		FRAC_PI_8 as FRAC_PI_8_F64,
	};
}

pub mod prelude {
	pub use crate::flat;
	pub use crate::io_add_msg;
	pub use crate::ShortToString;

	pub use once_cell::sync::Lazy;
	pub use parking_lot::{Mutex, MutexGuard, MappedMutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard, MappedRwLockReadGuard};
	pub use smart_default::*;
}

/// Makes defining a flat module (e.g. foo::Baz instead of foo::bar::Baz) easier.
///
/// Instead of
/// ```ignore
/// pub mod foo;
/// pub use foo::*;
/// pub mod bar;
/// pub use bar::*;
/// ```
/// You could use
/// ```ignore
/// use nil::*;
///
/// flat! {
///     foo;
///     bar;
/// }
/// ```
#[macro_export]
macro_rules! flat {
	{$($(#[$attr:meta])* $name:ident ;)*} => {
		$( $(#[$attr])* mod $name; $(#[$attr])* pub use self::$name::*; )*
	};
}

/// Expands to a function that prepends a message to an io error, to be used with `Result::map_err`.
#[macro_export]
macro_rules! io_add_msg {
	($($msg:tt)+) => {
		|err| std::io::Error::new(err.kind(), format!("{} {err}", format!($($msg)+)))
	};
}

/// Extension trait that shortens `.to_owned()` or `.to_string_lossy().to_string()` into just `.s()` to get a [String].
/// 
/// # Examples
/// ```
/// use nil::*;
/// 
/// let string: String = "foo".s();
/// let owned_str: String = "foo".to_owned();
/// 
/// assert_eq!(string, owned_str);
/// ```
pub trait ShortToString {
	/// Shorthand for getting a string representation
	fn s(&self) -> String;
}

impl ShortToString for str {
	#[inline]
	fn s(&self) -> String {
		self.to_owned()
	}
}

impl ShortToString for std::ffi::OsStr {
	#[inline]
	fn s(&self) -> String {
		self.to_string_lossy().to_string()
	}
}

impl ShortToString for std::path::Path {
	#[inline]
	fn s(&self) -> String {
		self.to_string_lossy().to_string()
	}
}

impl ShortToString for std::ffi::CStr {
	#[inline]
	fn s(&self) -> String {
		self.to_string_lossy().to_string()
	}
}

/// Shorthand for `T::default()` or `Default::default()`, good for structure initialization. Inspired by a function of the same name and purpose in `bevy_utils`.
#[inline]
pub fn default<T: Default>() -> T {
	T::default()
}
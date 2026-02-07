use super::{Diff, EventQueue, PathBuilder};

/// A "memoized" parameters wrapper.
///
/// This type simplifies diffing management for
/// standalone parameters.
#[derive(Debug, Clone, Copy, Default)]
pub struct Memo<T> {
    value: T,
    baseline: T,
}

impl<T: Diff + Clone> Memo<T> {
    /// Construct a new [`Memo`].
    ///
    /// This clones the provided value to maintain
    /// a baseline for diffing.
    pub fn new(value: T) -> Self {
        Self {
            baseline: value.clone(),
            value,
        }
    }

    /// Generate events if the inner value has changed.
    ///
    /// This will also clone the inner value and assign it to the baseline.
    /// This may be inneficient if cloning is slow.
    pub fn update_memo<E: EventQueue>(&mut self, event_queue: &mut E) {
        self.value
            .diff(&self.baseline, PathBuilder::default(), event_queue);
        self.baseline = self.value.clone();
    }
}

impl<T> core::ops::Deref for Memo<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T> core::ops::DerefMut for Memo<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value
    }
}

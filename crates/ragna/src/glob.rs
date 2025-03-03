use once_cell::sync::Lazy;
use std::ops::Deref;

/// A global GPU variable.
#[derive(Debug)]
pub struct Glob<T> {
    pub(crate) inner: Lazy<T>,
    pub(crate) default_value: fn() -> T,
}

impl<T> Glob<T> {
    #[doc(hidden)]
    pub const fn new(glob: fn() -> T, default_value: fn() -> T) -> Self {
        Self {
            inner: Lazy::new(glob),
            default_value,
        }
    }
}

impl<T> Deref for Glob<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

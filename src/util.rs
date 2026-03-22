use std::cell::UnsafeCell;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug)]
pub struct Smuggle<T>(u64, UnsafeCell<Option<T>>);

impl<T> Hash for Smuggle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Smuggle<T> {
    pub fn new(hash: impl Hash, v: T) -> Self {
        let mut hasher = DefaultHasher::new();
        hash.hash(&mut hasher);
        Self(hasher.finish(), UnsafeCell::new(Some(v)))
    }

    pub fn take(&self) -> T {
        unsafe { &mut *self.1.get() }.take().unwrap()
    }
}

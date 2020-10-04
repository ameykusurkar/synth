use druid::{Data, Lens};
use std::sync::{Arc, Mutex};

pub struct InArcMutex<L> {
    inner: L,
}

impl<L> InArcMutex<L> {
    pub fn new<A, B>(inner: L) -> Self
    where
        B: Data,
        L: Lens<A, B>,
    {
        Self { inner }
    }
}

impl<A, B, L> Lens<Arc<Mutex<A>>, B> for InArcMutex<L>
where
    B: Data,
    L: Lens<A, B>,
{
    fn with<V, F: FnOnce(&B) -> V>(&self, data: &Arc<Mutex<A>>, f: F) -> V {
        // TODO: Find better solution than `unwrap`
        self.inner.with(&data.lock().unwrap(), f)
    }

    fn with_mut<V, F: FnOnce(&mut B) -> V>(&self, data: &mut Arc<Mutex<A>>, f: F) -> V {
        // TODO: Find better solution than `unwrap`
        self.inner.with_mut(&mut data.lock().unwrap(), f)
    }
}

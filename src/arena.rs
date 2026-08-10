use std::{
    collections::{HashMap, hash_map::Iter},
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{
        LazyLock, RwLock,
        atomic::{AtomicUsize, Ordering},
    },
};

/* --------------------------------- STRUCTS -------------------------------- */

pub struct Arena<T> {
    map: LazyLock<RwLock<HashMap<Handle<T>, T>>>,
    next_id: AtomicUsize,
}

pub struct Handle<T>(usize, PhantomData<T>);

// TODO: register the constants somehow

/* ---------------------------------- IMPLS --------------------------------- */

impl<T> Arena<T>
where
    Handle<T>: Eq + Hash,
    T: Clone,
{
    pub const fn new() -> Self {
        Self {
            map: LazyLock::new(|| RwLock::new(HashMap::new())),
            next_id: AtomicUsize::new(0),
        }
    }

    pub fn insert(&self, val: T) -> Handle<T> {
        let id =
            Handle(self.next_id.fetch_add(1, Ordering::Relaxed), PhantomData);
        self.map.write().unwrap().insert(id, val);
        id
    }

    pub(crate) fn insert_at(&self, id: usize, value: T) {
        let handle = Handle::new(id);

        self.map.write().unwrap().insert(handle, value);

        self.next_id.fetch_max(id + 1, Ordering::Relaxed);
    }

    pub fn get_cloned(&self, id: Handle<T>) -> Option<T> {
        self.map.read().unwrap().get(&id).cloned()
    }

    pub fn find(
        &self,
        mut f: impl FnMut(Handle<T>, &T) -> bool,
    ) -> Option<(Handle<T>, T)> {
        let map = self.map.read().unwrap();

        map.iter()
            .find(|(handle, value)| f(**handle, value))
            .map(|(&handle, value)| (handle, value.clone()))
    }
}

impl<T> Handle<T> {
    pub(crate) const fn new(id: usize) -> Self {
        Self(id, PhantomData)
    }
}

impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> PartialEq for Handle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T> Eq for Handle<T> {}

impl<T> Hash for Handle<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Debug for Handle<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Id").field(&self.0).finish()
    }
}

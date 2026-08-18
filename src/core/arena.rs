use std::{
    collections::HashMap,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
    sync::{
        self, LazyLock, RwLock, RwLockReadGuard,
        atomic::{AtomicUsize, Ordering},
    },
};

/* --------------------------------- STRUCTS -------------------------------- */

pub struct Arena<T: Hash + Eq> {
    maps: LazyLock<RwLock<(HashMap<Handle<T>, T>, HashMap<T, Handle<T>>)>>,

    next_id: AtomicUsize,
}

pub struct ArenaRef<'a, T> {
    guard: RwLockReadGuard<'a, (HashMap<Handle<T>, T>, HashMap<T, Handle<T>>)>,
    id: Handle<T>,
}

impl<'a, T: Hash + Eq> Deref for ArenaRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.guard.0.get(&self.id).expect("THE HANDLE IS GON")
    }
}

pub struct Handle<T>(pub(crate) usize, PhantomData<T>);

// TODO: register the constants somehow

/* ---------------------------------- IMPLS --------------------------------- */

impl<T: Hash + Eq> Arena<T>
where
    Handle<T>: Eq + Hash,
    T: Clone,
{
    pub const fn new() -> Self {
        Self {
            maps: LazyLock::new(|| {
                RwLock::new((HashMap::new(), HashMap::new()))
            }),
            next_id: AtomicUsize::new(0),
        }
    }

    pub fn insert(&self, val: T) -> Handle<T> {
        let id =
            Handle(self.next_id.fetch_add(1, Ordering::Relaxed), PhantomData);
        let mut maps = self.maps.write().unwrap();

        maps.0.insert(id, val.clone());
        maps.1.insert(val, id);

        id
    }

    pub(crate) fn insert_at(&self, id: usize, val: T) {
        let handle = Handle::new(id);

        let mut maps = self.maps.write().unwrap();

        maps.0.insert(handle, val.clone());
        maps.1.insert(val, handle);

        self.next_id.fetch_max(id + 1, Ordering::Relaxed);
    }

    pub fn get_cloned(&self, id: Handle<T>) -> Option<T> {
        let maps = self.maps.read().unwrap();

        maps.0.get(&id).cloned()
    }

    pub fn get(&self, id: Handle<T>) -> Option<ArenaRef<'_, T>> {
        let guard = self.maps.read().unwrap();

        if guard.0.contains_key(&id) {
            Some(ArenaRef { guard, id })
        } else {
            None
        }
    }

    pub fn handle_of(&self, val: &T) -> Option<Handle<T>> {
        let maps = self.maps.read().unwrap();

        maps.1.get(val).copied()
    }

    pub fn modify(&self, id: Handle<T>, f: impl FnOnce(&mut T) -> ()) {
        let mut maps = self.maps.write().unwrap();

        maps.0.get_mut(&id).map(f);
    }

    pub fn find(
        &self,
        mut f: impl FnMut(Handle<T>, &T) -> bool,
    ) -> Option<(Handle<T>, T)> {
        let maps = self.maps.read().unwrap();

        maps.0
            .iter()
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

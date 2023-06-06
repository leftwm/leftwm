use std::collections::hash_map::Iter;

use nohash_hasher::{BuildNoHashHasher, IntMap};

use crate::managed_window::ManagedWindow;

pub type WindowHandle = usize;

pub struct WindowRegisty {
    map: IntMap<WindowHandle, ManagedWindow>,
    next_key: WindowHandle,
}

impl WindowRegisty {
    pub fn new() -> Self {
        Self {
            map: IntMap::with_hasher(BuildNoHashHasher::default()),
            next_key: 0,
        }
    }

    pub fn get(&self, key: WindowHandle) -> Option<&ManagedWindow> {
        self.map.get(&key)
    }

    pub fn get_mut(&mut self, key: WindowHandle) -> Option<&mut ManagedWindow> {
        self.map.get_mut(&key)
    }

    pub fn insert(&mut self, mut window: ManagedWindow) -> WindowHandle {
        let key = self.next_key;
        self.next_key += 1;
        window.handle = Some(key);
        self.map.insert(key, window);
        return key;
    }

    pub fn remove(&mut self, key: WindowHandle) -> Option<ManagedWindow> {
        self.map.remove(&key)
    }

    pub fn windows(&self) -> Iter<'_, WindowHandle, ManagedWindow> {
        self.map.iter()
    }
}

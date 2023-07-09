use std::collections::hash_map::Iter;

use nohash_hasher::{BuildNoHashHasher, IntMap};
use smithay::utils::{IsAlive, Logical, Point, Rectangle};

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
        window.set_handle(key);
        self.map.insert(key, window);
        return key;
    }

    pub fn remove(&mut self, key: WindowHandle) -> Option<ManagedWindow> {
        self.map.remove(&key)
    }

    pub fn windows(&self) -> Iter<'_, WindowHandle, ManagedWindow> {
        self.map.iter()
    }

    pub fn window_under(
        &self,
        pos: Point<i32, Logical>,
    ) -> Option<(ManagedWindow, Point<i32, Logical>)> {
        self.windows()
            .find(|(_, w)| {
                w.data
                    .read()
                    .unwrap()
                    .geometry
                    .map(|g| g.contains(pos))
                    .unwrap_or(false)
            })
            .map(|(_, w)| (w.clone(), w.data.read().unwrap().geometry.unwrap().loc))
    }

    pub fn clean(&mut self) {
        self.map.retain(|_, w| w.alive());
    }

    pub fn windows_in_rect(&self, output_geometry: &Rectangle<i32, Logical>) -> Vec<ManagedWindow> {
        self.windows()
            .filter(|(_, w)| {
                w.data
                    .read()
                    .unwrap()
                    .geometry
                    .map(|g| output_geometry.overlaps(g))
                    .unwrap_or(false)
            })
            .map(|(_, w)| w.clone())
            .collect()
    }
}

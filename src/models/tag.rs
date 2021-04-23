// We allow this so that we can be verbose and indicate that
// this is `TagModel` and not `WindowModel` or anything else.
#![allow(clippy::module_name_repetitions)]
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct TagModel {
    pub id: String,
    //HACK: This Arc<Mutex> has nothing to do with threading
    //It appears to be a dirty hack mutate this TagModel without having
    //a mutable reference. This should be refactored
    #[serde(skip)]
    main_width_percentage: Arc<Mutex<u8>>,
    #[serde(skip)]
    flipped_horizontal: Arc<AtomicBool>,
    #[serde(skip)]
    flipped_vertical: Arc<AtomicBool>,
}

impl TagModel {
    #[must_use]
    pub fn new(id: &str) -> Tag {
        Arc::new(TagModel {
            id: id.to_owned(),
            main_width_percentage: Arc::new(Mutex::new(50)),
            flipped_horizontal: Arc::new(AtomicBool::new(false)),
            flipped_vertical: Arc::new(AtomicBool::new(false)),
        })
    }

    /// # Panics
    ///
    /// Panics if `main_width_percentage` cannot be unwrapped.
    // TODO: Verify that `unwrap` panic cannot be hit and add note above
    pub fn increase_main_width(&self, delta: u8) {
        let lock = self.main_width_percentage.clone();
        let mut mwp = lock.lock().expect("FATAL ERROR: Mutex is corrupt");
        *mwp += delta;
        if *mwp > 100 {
            *mwp = 100
        }
    }
    /// # Panics
    ///
    /// Panics if `main_width_percentage` cannot be unwrapped.
    // TODO: Verify that `unwrap` panic cannot be hit and add note above.
    pub fn decrease_main_width(&self, delta: u8) {
        let lock = self.main_width_percentage.clone();
        let mut mwp = lock.lock().expect("FATAL ERROR: Mutex is corrupt");
        if *mwp > delta {
            *mwp -= delta;
        } else {
            *mwp = 0;
        }
    }
    /// # Panics
    ///
    /// Panics if `main_width_percentage` cannot be unwrapped.
    // TODO: Verify that `unwrap` panic cannot be hit and add note above.
    pub fn set_main_width(&self, val: u8) {
        let lock = self.main_width_percentage.clone();
        let mut mwp = lock.lock().expect("FATAL ERROR: Mutex is corrupt");

        if val > 100 {
            *mwp = 100;
        } else {
            *mwp = val;
        }
    }

    /// # Panics
    ///
    /// Panics if `main_width_percentage` cannot be unwrapped.
    // TODO: Verify that `unwrap` panic cannot be hit and add note above.
    #[must_use]
    pub fn main_width_percentage(&self) -> f32 {
        let lock = self.main_width_percentage.clone();
        let mwp = lock.lock().expect("FATAL ERROR: Mutex is corrupt");
        f32::from(*mwp)
    }

    pub fn flip_horizontal(&self, val: bool) {
        let clone = self.flipped_horizontal.clone();
        Arc::try_unwrap(clone)
            .unwrap_err()
            .store(val, Ordering::SeqCst);
    }

    pub fn flip_vertical(&self, val: bool) {
        let clone = self.flipped_vertical.clone();
        Arc::try_unwrap(clone)
            .unwrap_err()
            .store(val, Ordering::SeqCst);
    }

    #[must_use]
    pub fn flipped_horizontal(&self) -> bool {
        let clone = self.flipped_horizontal.clone();
        Arc::try_unwrap(clone).unwrap_err().load(Ordering::SeqCst)
    }

    #[must_use]
    pub fn flipped_vertical(&self) -> bool {
        let clone = self.flipped_vertical.clone();
        Arc::try_unwrap(clone).unwrap_err().load(Ordering::SeqCst)
    }
}

pub type Tag = Arc<TagModel>;

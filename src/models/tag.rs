use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct TagModel {
    pub id: String,
    #[serde(skip)]
    main_width_percentage: Arc<Mutex<u8>>,
    // margin_multiplier: f32,
}

impl TagModel {
    pub fn new(id: &str) -> Tag {
        Arc::new(TagModel {
            id: id.to_owned(),
            main_width_percentage: Arc::new(Mutex::new(50)),
            // margin_multiplier: 1.0,
        })
    }

    pub fn increase_main_width(&self, delta: u8) {
        let lock = self.main_width_percentage.clone();
        let mut mwp = lock.lock().unwrap();
        *mwp += delta;
        if *mwp > 100 {
            *mwp = 100
        }
    }
    pub fn decrease_main_width(&self, delta: u8) {
        let lock = self.main_width_percentage.clone();
        let mut mwp = lock.lock().unwrap();
        if *mwp > delta {
            *mwp -= delta;
        } else {
            *mwp = 0;
        }
    }

    pub fn set_main_width(&self, val: u8) {
        let lock = self.main_width_percentage.clone();
        let mut mwp = lock.lock().unwrap();

        if val > 100 {
            *mwp = 100;
        } else {
            *mwp = val;
        }
    }

    pub fn main_width_percentage(&self) -> f32 {
        let lock = self.main_width_percentage.clone();
        let mwp = lock.lock().unwrap();
        *mwp as f32
    }

    // /// Set the tag model's margin multiplier.
    // pub fn set_margin_multiplier(&mut self, margin_multiplier: f32) {
    //     self.margin_multiplier = margin_multiplier;
    // }

    // /// Get a reference to the tag model's margin multiplier.
    // pub fn margin_multiplier(&self) -> &f32 {
    //     &self.margin_multiplier
    // }
}

pub type Tag = Arc<TagModel>;

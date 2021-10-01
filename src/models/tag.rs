use serde::{Deserialize, Serialize};

use crate::layouts::Layout;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct Tag {
    pub id: String,
    pub hidden: bool,
    pub layout: Layout,
    pub layouts: Vec<Layout>,
    pub main_width_percentage: u8,
    pub flipped_horizontal: bool,
    pub flipped_vertical: bool,
}

impl Tag {
    #[must_use]
    pub fn new(id: &str, layouts: Vec<Layout>) -> Tag {
        Tag {
            id: id.to_owned(),
            hidden: false,
            layout: Layout::new(&layouts),
            layouts,
            main_width_percentage: 50,
            flipped_horizontal: false,
            flipped_vertical: false,
        }
    }

    pub fn change_main_width(&mut self, delta: i8) {
        //Check we are not gonna go negative
        let mwp = &mut self.main_width_percentage;
        if (*mwp as i8) < -delta {
            *mwp = 0;
            return;
        }
        if delta.is_negative() {
            *mwp -= delta.unsigned_abs();
            return;
        }
        *mwp += delta as u8;
        if *mwp > 100 {
            *mwp = 100;
        }
    }

    pub fn set_main_width(&mut self, val: u8) {
        let mwp = &mut self.main_width_percentage;
        if val > 100 {
            *mwp = 100;
        } else {
            *mwp = val;
        }
    }

    #[must_use]
    pub fn main_width_percentage(&self) -> f32 {
        f32::from(self.main_width_percentage)
    }
}

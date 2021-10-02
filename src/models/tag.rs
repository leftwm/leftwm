use serde::{Deserialize, Serialize};

use crate::{layouts::Layout, Window, Workspace};

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: String,
    pub hidden: bool,
    pub layout: Layout,
    pub main_width_percentage: u8,
    pub flipped_horizontal: bool,
    pub flipped_vertical: bool,
    pub layout_rotation: usize,
}

impl Tag {
    #[must_use]
    pub fn new(id: &str, layout: Layout) -> Tag {
        Tag {
            id: id.to_owned(),
            hidden: false,
            layout,
            main_width_percentage: 50,
            flipped_horizontal: false,
            flipped_vertical: false,
            layout_rotation: 0,
        }
    }

    pub fn update_windows(&self, windows: &mut Vec<Window>, workspace: &Workspace) {
        if let Some(w) = windows
            .iter_mut()
            .find(|w| w.has_tag(&self.id) && w.is_fullscreen())
        {
            w.set_visible(true);
        } else {
            //Don't bother updating the other windows
            //mark all windows for this workspace as visible
            let mut all_mine: Vec<&mut Window> =
                windows.iter_mut().filter(|w| w.has_tag(&self.id)).collect();
            all_mine.iter_mut().for_each(|w| w.set_visible(true));
            //update the location of all non-floating windows
            let mut managed_nonfloat: Vec<&mut Window> = windows
                .iter_mut()
                .filter(|w| w.has_tag(&self.id) && !w.is_unmanaged() && !w.floating())
                .collect();
            self.layout
                .update_windows(workspace, &mut managed_nonfloat, self);
            for w in &mut managed_nonfloat {
                w.container_size = Some(workspace.xyhw);
            }
            //update the location of all floating windows
            windows
                .iter_mut()
                .filter(|w| w.has_tag(&self.id) && !w.is_unmanaged() && w.floating())
                .for_each(|w| w.normal = workspace.xyhw);
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

    pub fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
        self.set_main_width(layout.main_width());
        self.layout_rotation = 0;
    }

    pub fn rotate_layout(&mut self) -> Option<()> {
        let rotations = self.layout.rotations();
        self.layout_rotation += 1;
        if self.layout_rotation >= rotations.len() {
            self.layout_rotation = 0;
        }
        let (horz, vert) = rotations.get(self.layout_rotation)?;
        self.flipped_horizontal = *horz;
        self.flipped_vertical = *vert;
        Some(())
    }
}

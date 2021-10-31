use serde::{Deserialize, Serialize};

use crate::{layouts::Layout, Window, Workspace};

use super::TagId;

/// Wrapper struct holding all the tags.
/// This wrapper provides convenience methods to change the tag-list
/// during its lifetime, while ensuring that all tags are numbered correctly.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tags {
    // holds all the 'normal' tags
    vec: Vec<Tag>,

    // holds the 'hidden' tags
    hidden: Vec<Tag>,
}

impl Tags {
    pub fn new() -> Self {
        Tags {
            vec: vec![],
            hidden: vec![],
        }
    }

    pub fn add_new(&mut self, label: &str, layout: Layout) -> TagId {
        let next_id = self.vec.len() + 1; // tag id starts at 1
        let tag = Tag::new(next_id, label, layout);
        let id = tag.id;
        self.vec.push(tag);
        id
    }

    pub fn add_new_unlabeled(&mut self, layout: Layout) -> TagId {
        let next_id = self.vec.len() + 1; // tag id starts at 1
        self.add_new(next_id.to_string().as_str(), layout)
    }

    // todo: add_new_at(position, label, layout) -> shifting all one to the right (vec.insert)

    pub fn add_new_hidden(&mut self, label: &str) -> TagId {
        // hidden tags are numbered descending from the highest possible number
        let next_id = usize::MAX - self.hidden.len();
        let tag = Tag {
            id: next_id,
            label: label.to_string(),
            hidden: true,
            ..Tag::default()
        };
        let id = tag.id;
        self.hidden.push(tag);
        id
    }

    /// Get all the visible (non-hidden) tags
    pub fn visible(&self) -> &Vec<Tag> {
        &self.vec
    }

    /// Get all tags, including hidden ones.
    /// The hidden tags are appended at the end of the list.
    pub fn all(&self) -> Vec<Tag> {
        let mut result: Vec<Tag> = vec![];
        result.append(&mut self.vec.clone());
        result.append(&mut self.hidden.clone());
        result
    }

    /// Get a tag by its ID
    pub fn get(&self, id: TagId) -> Option<&Tag> {
        self.vec
            .get(id - 1) // tag id starts at 1, arrays at 0
            .or_else(|| self.hidden.iter().find(|&hidden_tag| hidden_tag.id == id))
    }

    pub fn get_mut(&mut self, id: TagId) -> Option<&mut Tag> {
        if let Some(normal) = self.vec.get_mut(id - 1) {
            return Some(normal);
        }
        return self
            .hidden
            .iter_mut()
            .find(|hidden_tag| hidden_tag.id == id);
    }

    /// Get a hidden tag by its label
    pub fn get_hidden(&self, label: &str) -> Option<&Tag> {
        self.hidden.iter().find(|tag| tag.label.eq(label))
    }

    /// Get the amount of 'normal' tags
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }
}

impl Default for Tags {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub id: TagId,
    pub label: String,
    pub hidden: bool,
    pub layout: Layout,
    pub main_width_percentage: u8,
    pub flipped_horizontal: bool,
    pub flipped_vertical: bool,
    pub layout_rotation: usize,
}

impl Tag {
    #[must_use]
    pub fn new(id: TagId, label: &str, layout: Layout) -> Tag {
        Tag {
            id,
            label: label.to_owned(),
            hidden: false,
            layout,
            main_width_percentage: layout.main_width(),
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

    pub fn set_layout(&mut self, layout: Layout, main_width_percentage: u8) {
        self.layout = layout;
        self.set_main_width(main_width_percentage);
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

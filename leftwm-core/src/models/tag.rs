use serde::{Deserialize, Serialize};

use crate::{layouts::Layout, Window, Workspace};

use super::TagId;

/// Wrapper struct holding all the tags.
/// This wrapper provides convenience methods to change the tag-list
/// during its lifetime, while ensuring that all tags are in correct order
/// and numbered accordingly.
///
/// Each Tag is stored in either the 'normal' or the 'hidden' list.
/// This is just an internal simplification for handling the different
/// kinds of Tags.
///
/// ## Normal Tags
/// Normal tags are the tags visible to the user, those are the tags
/// that can be labelled via the config file. Tags are identified by a
/// unique ID which is automatically assigned by `LeftWM`, those IDs start at 1
/// and increment by 1. You can always expect that the tags are ordered by their ID
/// and that there is no gap between the numbers. The largest tag ID is equal to the
/// amount of normal tags. This also means: tags with larger IDs are always to the right
/// and tags with smaller IDs are always to the left of the reference tag.
///
/// Usually the number of normal tags is equal to the number of tags configured by the user.
/// However, if there are more workspaces than there are tags, additional "unnamed" tags
/// will be created automatically and appended to the list.
///
/// ## Hidden Tags
/// A hidden tag is a tag that is invisible and unknown to the user.
/// Those tags are created in the source code and can be used for
/// various purposes. The Scratchpad (NSP) feature is an example which uses
/// a hidden Tag called "NSP" to hide away the scratchpad until its summoned again.
/// You can think of a hidden tag as an invisible window storage. It is not possible
/// for a user to display a hidden tag in a workspace.
///
/// Hidden tags also have a unique ID starting at `usize::MAX`, decrementing by 1.
/// This prevents conflicts of Tag IDs between normal and hidden tags and makes it easier
/// to dynamically change normal tags and their ID without affecting hidden tags and vice versa.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Tags {
    // holds all the 'normal' tags
    normal: Vec<Tag>,

    // holds the 'hidden' tags (like 'NSP')
    hidden: Vec<Tag>,
}

impl Tags {
    /// Create a new empty Taglist
    pub fn new() -> Self {
        Tags {
            normal: vec![],
            hidden: vec![],
        }
    }

    /// Create a new tag with the provided label and layout,
    /// and append it to the list of normal tags.
    /// The ID will be assigned automatically and returned.
    pub fn add_new(&mut self, label: &str, layout: Layout) -> TagId {
        let next_id = self.normal.len() + 1; // tag id starts at 1
        let tag = Tag::new(next_id, label, layout);
        let id = tag.id;
        self.normal.push(tag);
        id
    }

    /// Create a new tag with the provided layout, labelling it directly with its ID,
    /// and append it to the list of normal tags.
    /// The ID will be assigned automatically and returned.
    pub fn add_new_unlabeled(&mut self, layout: Layout) -> TagId {
        let next_id = self.normal.len() + 1; // tag id starts at 1
        self.add_new(next_id.to_string().as_str(), layout)
    }

    // todo: add_new_at(position, label, layout)
    // -> shifting all one to the right and re-number them (vec.insert)

    // todo: remove(id)
    // -> shifting all right of the removed tag one to the left and re-number them

    /// Create a new hidden tag with the provided label,
    /// and append it to the list of hidden tags.
    /// The ID will be assigned automatically and returned.
    ///
    /// ## Non unique label
    /// May not create a new tag if a hidden tag with the same label already exists,
    /// this is indicated by the return value of `None`.
    pub fn add_new_hidden(&mut self, label: &str) -> Option<TagId> {
        if self.get_hidden_by_label(label).is_none() {
            // hidden tags are numbered descending from the highest possible number
            // to prevent conflicts with IDs of normal tags
            let next_id = usize::MAX - self.hidden.len();
            let tag = Tag {
                id: next_id,
                label: label.to_string(),
                hidden: true,
                ..Tag::default()
            };
            let id = tag.id;
            self.hidden.push(tag);
            Some(id)
        } else {
            log::error!(
                "tried creating a hidden tag with label {}, but a hidden tag with the same label already exists",
                label
            );
            None
        }
    }

    /// Get all normal tags
    pub fn normal(&self) -> &Vec<Tag> {
        &self.normal
    }

    /// Get all tags, including hidden ones.
    /// The hidden tags are appended at the end of the list.
    pub fn all(&self) -> Vec<Tag> {
        let mut result: Vec<Tag> = vec![];
        result.append(&mut self.normal.clone());
        result.append(&mut self.hidden.clone());
        result
    }

    /// Get a tag by its ID.
    /// This method returns normal, as well as hidden tags.
    pub fn get(&self, id: TagId) -> Option<&Tag> {
        self.normal
            .get(id - 1) // tag id starts at 1, but arrays at 0 :)
            .or_else(|| self.hidden.iter().find(|&hidden_tag| hidden_tag.id == id))
    }

    /// Get a tag by its ID as mutable
    /// This method returns normal, as well as hidden tags.
    pub fn get_mut(&mut self, id: TagId) -> Option<&mut Tag> {
        if let Some(normal) = self.normal.get_mut(id - 1) {
            return Some(normal);
        }
        return self
            .hidden
            .iter_mut()
            .find(|hidden_tag| hidden_tag.id == id);
    }

    /// Get a hidden tag by its label
    pub fn get_hidden_by_label(&self, label: &str) -> Option<&Tag> {
        self.hidden.iter().find(|tag| tag.label.eq(label))
    }

    /// Get the amount of 'normal' tags
    pub fn len_normal(&self) -> usize {
        self.normal.len()
    }
}

impl Default for Tags {
    fn default() -> Self {
        Self::new()
    }
}

/// A Tag is similar to a "Desktop".
/// Each Screen/Workspace will always display a certain Tag.
/// A Tag can not be displayed on more than one Workspace at a time.
///
/// In `LeftWM` the same set of tags and windows are shared among
/// all Workspaces, this means there aren't multiple instances of
/// the same Tag on different Screens, like it may be the cause for other
/// WMs (eg. `dwm`).
#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    /// Unique identifier for the tag,
    /// this is automatically assigned by `LeftWM`
    pub id: TagId,

    /// Label of the tag, may not be unique,
    /// only used for display purposes
    pub label: String,

    /// Indicates whether the tag can be
    /// displayed in a Workspace or not.
    /// Hidden tags are internal only, and
    /// are not known to other programs like (eg. polybar)
    pub hidden: bool,

    /// The layout in which the windows
    /// on this Tag are arranged
    pub layout: Layout,

    /// The percentage of available space
    /// which is designated for the "main"
    /// column of the layout, compared
    /// to the secondary column(s).
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

    /// Changes the main width percentage by the provided delta.
    /// Result is sanitized, so the percentage can't go below 0 or above 100.
    ///
    /// ## Arguments
    /// * `delta` - increase/decrease main width percentage by this amount
    pub fn change_main_width(&mut self, delta: i8) {
        self.main_width_percentage = (self.main_width_percentage as i8 + delta)
            .max(0) // not smaller than 0
            .min(100) as u8; // not larger than 100
    }

    /// Sets the main width percentage
    ///
    /// ## Arguments
    /// * `val` - the new with percentage
    pub fn set_main_width(&mut self, val: u8) {
        self.main_width_percentage = val.min(100); // not larger than 100
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

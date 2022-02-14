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
    pub const fn new() -> Self {
        Self {
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
    pub const fn normal(&self) -> &Vec<Tag> {
        &self.normal
    }

    /// Get a list of all tags, including hidden ones.
    /// The hidden tags are at the end of the list.
    pub fn all(&self) -> Vec<&Tag> {
        //&self.normal.append(&self.hidden)
        let mut result: Vec<&Tag> = vec![];
        self.normal.iter().for_each(|tag| result.push(tag));
        self.hidden.iter().for_each(|tag| result.push(tag));
        result
    }

    /// Get a list of all tags as mutable, including hidden ones.
    /// The hidden tags are at the end of the list
    pub fn all_mut(&mut self) -> Vec<&mut Tag> {
        //&self.normal.append(&self.hidden)
        let mut result: Vec<&mut Tag> = vec![];
        self.normal.iter_mut().for_each(|tag| result.push(tag));
        self.hidden.iter_mut().for_each(|tag| result.push(tag));
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
/// Unlike in some other WMs (eg. `dwm`), in `LeftWM`
/// the same set of tags and windows are shared among
/// all Workspaces, this means there aren't multiple instances of
/// the same Tag on different Screens.
#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    /// Unique identifier for the tag,
    /// this is automatically assigned by `LeftWM`.
    /// IDs start at 1 for the first tag, and are
    /// incremented by 1 for each subsequent tag.
    pub id: TagId,

    /// Label of the tag, only used for display purposes.
    ///
    /// Labels of normal tags may not be unique,
    /// but labels of hidden tags must be unique.
    ///
    /// ## Hint
    /// Unlike in earlier versions of LeftWM,
    /// the label of a Tag is not something that
    /// actually identifies a Tag. Tags are always
    /// identified and referenced by their ID (ie. `[1, 2, 3, ...]`).
    ///
    /// This means, if the user configures the tags `["home", "chat", "surf", "code"]`
    /// and later removes the `chat` tag, leaving `["home", "surf", "code"]`, this does not mean
    /// that Tag 2 is removed, it just means that Tag 2 has the label "surf" now.
    /// What is actually removed in that case is Tag 4.
    pub label: String,

    /// Indicates whether the tag can be
    /// displayed in a Workspace or not.
    /// Hidden tags are internal only, and
    /// are unknown to other programs (eg. polybar)
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
    pub fn new(id: TagId, label: &str, layout: Layout) -> Self {
        Self {
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

    pub fn update_windows(&self, windows: &mut [Window], workspace: &Workspace) {
        if let Some(window) = windows
            .iter_mut()
            .find(|w| w.has_tag(&self.id) && w.is_fullscreen())
        {
            window.set_visible(true);
            window.normal = workspace.xyhw;
            let handle = window.handle;
            windows
                .iter_mut()
                .filter(|w| {
                    w.has_tag(&self.id)
                        && w.transient.unwrap_or_else(|| 0.into()) == handle
                        && !w.is_unmanaged()
                })
                .for_each(|w| {
                    w.set_visible(true);
                });
        } else {
            // Don't bother updating the other windows when a window is fullscreen.
            // Mark all windows for this workspace as visible.
            let mut all_mine: Vec<&mut Window> =
                windows.iter_mut().filter(|w| w.has_tag(&self.id)).collect();
            all_mine.iter_mut().for_each(|w| w.set_visible(true));
            // Update the location of all non-floating windows.
            let mut managed_nonfloat: Vec<&mut Window> = windows
                .iter_mut()
                .filter(|w| w.has_tag(&self.id) && !w.is_unmanaged() && !w.floating())
                .collect();
            self.layout
                .update_windows(workspace, &mut managed_nonfloat, self);
            for w in &mut managed_nonfloat {
                w.container_size = Some(workspace.xyhw);
            }
            // Update the location of all floating windows.
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

#[cfg(test)]
mod tests {
    use super::Tags;
    use crate::layouts::Layout;

    #[test]
    fn normal_tags_are_numbered_in_order() {
        let mut tags = Tags::new();
        let home_id = tags.add_new("home", Layout::default());
        let chat_id = tags.add_new("chat", Layout::default());
        let surf_id = tags.add_new("surf", Layout::default());
        let code_id = tags.add_new("code", Layout::default());
        assert_eq!(home_id, 1);
        assert_eq!(chat_id, 2);
        assert_eq!(surf_id, 3);
        assert_eq!(code_id, 4);
    }

    #[test]
    fn hidden_tags_are_numbered_in_order() {
        let mut tags = Tags::new();
        let nsp_id = tags.add_new_hidden("NSP");
        let whatver_id = tags.add_new_hidden("whatever");
        assert_eq!(nsp_id, Some(usize::MAX));
        assert_eq!(whatver_id, Some(usize::MAX - 1));
    }

    #[test]
    fn multiple_normal_tags_can_have_same_label() {
        let mut tags = Tags::new();
        let first_id = tags.add_new("home", Layout::default());
        let second_id = tags.add_new("home", Layout::default());
        assert_eq!(first_id, 1);
        assert_eq!(second_id, 2);
    }

    #[test]
    fn unlabelled_tags_are_automatically_labelled_with_their_id() {
        let mut tags = Tags::new();
        let first_tag = tags.add_new_unlabeled(Layout::default());
        let second_tag = tags.add_new_unlabeled(Layout::default());
        let first_label = tags.get(first_tag).map(|tag| tag.label.clone());
        let second_label = tags.get(second_tag).map(|tag| tag.label.clone());
        assert_eq!(first_label, Some(String::from("1")));
        assert_eq!(second_label, Some(String::from("2")));
    }

    #[test]
    fn hidden_tags_must_have_unique_label() {
        let mut tags = Tags::new();
        let first_tag = tags.add_new_hidden("NSP");
        let second_tag = tags.add_new_hidden("NSP");
        let third_tag = tags.add_new_hidden("something-unique");
        assert_eq!(first_tag, Some(usize::MAX));
        assert_eq!(second_tag, None);
        assert_eq!(third_tag, Some(usize::MAX - 1));
        assert_eq!(tags.all().len(), 2); // the second tag must not be created
    }

    #[test]
    fn must_be_able_to_only_get_normal_tags() {
        let mut tags = Tags::new();
        tags.add_new("home", Layout::default());
        tags.add_new("chat", Layout::default());
        tags.add_new("surf", Layout::default());
        tags.add_new("code", Layout::default());
        tags.add_new_hidden("NSP");

        assert_eq!(tags.len_normal(), 4);
        assert_eq!(tags.normal().len(), 4);
    }

    #[test]
    fn must_be_able_to_get_all_tags() {
        let mut tags = Tags::new();
        tags.add_new("home", Layout::default());
        tags.add_new("chat", Layout::default());
        tags.add_new("surf", Layout::default());
        tags.add_new("code", Layout::default());
        tags.add_new_hidden("NSP");

        assert_eq!(tags.all().len(), 5);
    }

    #[test]
    fn hidden_tags_must_be_retrievable_by_label() {
        let mut tags = Tags::new();
        tags.add_new("home", Layout::default());
        tags.add_new_hidden("NSP");
        tags.add_new_hidden("whatever");

        let nsp_tag = tags.get_hidden_by_label("NSP");
        let whatever_tag = tags.get_hidden_by_label("whatever");
        let inexistent_tag = tags.get_hidden_by_label("inexistent");

        assert!(nsp_tag.is_some());
        assert!(whatever_tag.is_some());
        assert!(inexistent_tag.is_none());
    }

    #[test]
    fn only_hidden_tags_can_be_retrieved_by_label() {
        let mut tags = Tags::new();
        tags.add_new("home", Layout::default());
        let tag = tags.get_hidden_by_label("home");
        assert!(tag.is_none());
    }

    #[test]
    fn tags_can_be_mutable() {
        let mut tags = Tags::new();
        tags.add_new("home", Layout::default());
        tags.add_new("chat", Layout::default());
        tags.add_new("surf", Layout::default());

        let first_retrieve = tags.get_mut(2).unwrap();
        assert_eq!(first_retrieve.label, String::from("chat"));
        first_retrieve.label = String::from("code");

        let second_retrieve = tags.get_mut(2).unwrap();
        assert_eq!(second_retrieve.label, String::from("code"));
    }
}

use super::{TagId, Xyhw, Handle};
use crate::{layouts::LayoutManager, Window, Workspace};
use serde::{Deserialize, Serialize};

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
    pub fn add_new(&mut self, label: &str) -> TagId {
        let next_id = self.normal.len() + 1; // tag id starts at 1
        let tag = Tag::new(next_id, label);
        let id = tag.id;
        self.normal.push(tag);
        id
    }

    /// Create a new tag with the provided layout, labelling it directly with its ID,
    /// and append it to the list of normal tags.
    /// The ID will be assigned automatically and returned.
    pub fn add_new_unlabeled(&mut self) -> TagId {
        let next_id = self.normal.len() + 1; // tag id starts at 1
        self.add_new(next_id.to_string().as_str())
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
            };
            let id = tag.id;
            self.hidden.push(tag);
            Some(id)
        } else {
            tracing::error!(
                "Tried creating a hidden tag with label {}, but a hidden tag with the same label already exists",
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
        // &self.normal.append(&self.hidden)
        let mut result: Vec<&Tag> = vec![];
        self.normal.iter().for_each(|tag| result.push(tag));
        self.hidden.iter().for_each(|tag| result.push(tag));
        result
    }

    /// Get a list of all tags as mutable, including hidden ones.
    /// The hidden tags are at the end of the list
    pub fn all_mut(&mut self) -> Vec<&mut Tag> {
        // &self.normal.append(&self.hidden)
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
}

impl Tag {
    #[must_use]
    pub fn new(id: TagId, label: &str) -> Self {
        Self {
            id,
            label: label.to_owned(),
            hidden: false,
        }
    }

    pub fn update_windows<H: Handle>(
        &self,
        windows: &mut [Window<H>],
        workspace: &Workspace,
        layout_manager: &mut LayoutManager,
    ) {
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
                        && (w.transient == Some(handle)
                            || w.states.contains(&super::WindowState::Above) && w.floating())
                        && w.is_managed()
                })
                .for_each(|w| {
                    w.set_visible(true);
                });
        } else if let Some(window) = windows
            .iter_mut()
            .find(|w| w.has_tag(&self.id) && w.is_maximized())
        {
            window.set_visible(true);
            window.normal = workspace.rect().into();

            windows
                .iter_mut()
                .filter(|w| w.has_tag(&self.id) && w.floating())
                .for_each(|w| {
                    w.set_visible(true);
                });
        } else {
            // Don't bother updating the other windows when a window is fullscreen.
            // Mark all windows for this workspace as visible.
            let mut all_mine: Vec<&mut Window<H>> =
                windows.iter_mut().filter(|w| w.has_tag(&self.id)).collect();
            all_mine.iter_mut().for_each(|w| w.set_visible(true));

            // Update the location / visibility of all non-floating windows.
            let mut managed_nonfloat: Vec<&mut Window<H>> = windows
                .iter_mut()
                .filter(|w| w.has_tag(&self.id) && w.is_managed() && !w.floating())
                .collect();
            let def = layout_manager.layout(workspace.id, workspace.tag.unwrap_or(1));
            let rects = leftwm_layouts::apply(def, managed_nonfloat.len(), &workspace.rect());
            for (i, window) in managed_nonfloat.iter_mut().enumerate() {
                match rects.get(i) {
                    Some(rect) => {
                        window.normal = Xyhw::from(*rect);
                        window.container_size = Some(workspace.xyhw);
                    }
                    None => {
                        window.set_visible(false);
                    }
                }
            }

            // Update the location of all floating windows.
            windows
                .iter_mut()
                .filter(|w| w.has_tag(&self.id) && w.is_managed() && w.floating())
                .for_each(|w| w.normal = workspace.xyhw);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Tags;

    #[test]
    fn normal_tags_are_numbered_in_order() {
        let mut tags = Tags::new();
        let home_id = tags.add_new("home");
        let chat_id = tags.add_new("chat");
        let surf_id = tags.add_new("surf");
        let code_id = tags.add_new("code");
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
        let first_id = tags.add_new("home");
        let second_id = tags.add_new("home");
        assert_eq!(first_id, 1);
        assert_eq!(second_id, 2);
    }

    #[test]
    fn unlabelled_tags_are_automatically_labelled_with_their_id() {
        let mut tags = Tags::new();
        let first_tag = tags.add_new_unlabeled();
        let second_tag = tags.add_new_unlabeled();
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
        tags.add_new("home");
        tags.add_new("chat");
        tags.add_new("surf");
        tags.add_new("code");
        tags.add_new_hidden("NSP");

        assert_eq!(tags.len_normal(), 4);
        assert_eq!(tags.normal().len(), 4);
    }

    #[test]
    fn must_be_able_to_get_all_tags() {
        let mut tags = Tags::new();
        tags.add_new("home");
        tags.add_new("chat");
        tags.add_new("surf");
        tags.add_new("code");
        tags.add_new_hidden("NSP");

        assert_eq!(tags.all().len(), 5);
    }

    #[test]
    fn hidden_tags_must_be_retrievable_by_label() {
        let mut tags = Tags::new();
        tags.add_new("home");
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
        tags.add_new("home");
        let tag = tags.get_hidden_by_label("home");
        assert!(tag.is_none());
    }

    #[test]
    fn tags_can_be_mutable() {
        let mut tags = Tags::new();
        tags.add_new("home");
        tags.add_new("chat");
        tags.add_new("surf");

        let first_retrieve = tags.get_mut(2).unwrap();
        assert_eq!(first_retrieve.label, String::from("chat"));
        first_retrieve.label = String::from("code");

        let second_retrieve = tags.get_mut(2).unwrap();
        assert_eq!(second_retrieve.label, String::from("code"));
    }
}

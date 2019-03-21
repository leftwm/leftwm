use super::layouts::*;
use super::Screen;
use super::Window;
use std::collections::VecDeque;
use std::fmt;

#[derive(Clone)]
pub struct Workspace {
    pub id: i32,
    layouts: VecDeque<Box<Layout>>,
    pub tags: Vec<String>,
    pub height: i32,
    pub width: i32,
    pub x: i32,
    pub y: i32,
}

impl fmt::Debug for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Workspace {{ id: {}, tags: {:?}, x: {}, y: {} }}",
            self.id, self.tags, self.x, self.y
        )
    }
}

impl PartialEq for Workspace {
    fn eq(&self, other: &Workspace) -> bool {
        self.id != -1 && self.id == other.id
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Workspace {
            id: -1,
            layouts: get_all_layouts(),
            tags: vec![],
            height: 600,
            width: 800,
            x: 0,
            y: 0,
        }
    }
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace::default()
    }

    pub fn from_screen(screen: &Screen) -> Workspace {
        Workspace {
            id: -1,
            layouts: get_all_layouts(),
            tags: vec![],
            height: screen.height,
            width: screen.width,
            x: screen.x,
            y: screen.y,
        }
    }

    pub fn show_tag(&mut self, tag: String) {
        self.tags = vec![tag.clone()];
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        let maxx = self.x + self.width;
        let maxy = self.y + self.height;
        (self.x <= x && x <= maxx) && (self.y <= y && y <= maxy)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for t in &self.tags {
            if tag == t {
                return true;
            }
        }
        false
    }

    pub fn next_layout(&mut self) {
        let layout = self.layouts.pop_front();
        if let Some(layout) = layout {
            self.layouts.push_back(layout);
        }
    }

    pub fn prev_layout(&mut self) {
        let layout = self.layouts.pop_back();
        if let Some(layout) = layout {
            self.layouts.push_front(layout);
        }
    }

    /*
     * returns true if the workspace is displays a given window
     */
    pub fn is_displaying(&self, window: &Window) -> bool {
        for wd_t in &window.tags {
            if self.has_tag(wd_t) {
                return true;
            }
        }
        false
    }

    /*
     * given a list of windows, returns a sublist of the windows that this workspace is displaying
     */
    pub fn displayed_windows<'a>(&self, windows: Vec<&'a mut Window>) -> Vec<&'a mut Window> {
        windows
            .into_iter()
            .filter(|w| self.is_displaying(w) && !w.floating())
            .collect::<Vec<&mut Window>>()
    }

    pub fn update_windows(&self, windows: Vec<&mut Window>) {
        let mut mine = self.displayed_windows(windows);
        mine.iter_mut().for_each(|w| w.set_visable(true));
        self.layouts[0].update_windows(self, &mut mine);
    }
}

#[test]
fn empty_ws_should_not_contain_window() {
    use super::WindowHandle;
    let subject = Workspace::new();
    let w = Window::new(WindowHandle::MockHandle(1), None);
    assert!(
        subject.is_displaying(&w) == false,
        "workspace incorrectly owns window"
    );
}

#[test]
fn tagging_a_workspace_to_with_the_same_tag_as_a_window_should_couse_it_to_display() {
    use super::WindowHandle;
    let mut subject = Workspace::new();
    subject.show_tag("test".to_owned());
    let mut w = Window::new(WindowHandle::MockHandle(1), None);
    w.tag("test".to_owned());
    assert!(
        subject.is_displaying(&w) == true,
        "workspace should include window"
    );
}

#[test]
fn displayed_windows_should_return_a_list_of_display_windows() {
    use super::WindowHandle;
    let mut subject = Workspace::new();
    subject.show_tag("test".to_owned());
    let mut w = Window::new(WindowHandle::MockHandle(1), None);
    w.tag("test".to_owned());
    let windows = vec![&mut w];
    assert!(
        subject.displayed_windows(windows).len() == 1,
        "workspace should include window"
    );
}

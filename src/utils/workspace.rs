use super::layouts::*;
use super::screen::Screen;
use super::window::Window;

#[derive(Clone)]
pub struct Workspace {
    pub name: String,
    layout: Box<Layout>,
    pub tags: Vec<String>,
    pub height: i32,
    pub width: i32,
    pub x: i32,
    pub y: i32,
}

impl PartialEq for Workspace {
    fn eq(&self, other: &Workspace) -> bool {
        self.name == other.name
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Workspace {
            name: "".to_owned(),
            layout: Box::new(DefaultLayout {}),
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
            name: "".to_owned(),
            layout: Box::new(DefaultLayout {}),
            tags: vec![],
            height: screen.height,
            width: screen.width,
            x: 0,
            y: 0,
        }
    }

    pub fn show_tag(&mut self, tag: String) {
        self.tags = vec![tag.clone()];
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for t in &self.tags {
            if tag == t {
                return true
            }
        }
        false
    }

    /*
     * returns true if the workspace is displays a given window
     */
    pub fn is_displaying(&self, window: &Window) -> bool {
        for wd_t in &window.tags {
            if self.has_tag( wd_t ) {
                return true
            }
        }
        false
    }

    /*
     * given a list of windows, returns a sublist of the windows that this workspace is displaying
     */
    pub fn displayed_windows<'a>(&self, windows: Vec<&'a mut Window>) -> Vec<&'a mut Window> {
        let found: Vec<&mut Window> = windows
            .into_iter()
            .filter(|w| self.is_displaying(w))
            .collect();
        found
    }

    pub fn update_windows(&self, windows: Vec<&mut Window>) {
        let mut mine = self.displayed_windows(windows);
        mark_visable(&mut mine);
        self.layout.update_windows(self, mine);
    }
}


fn mark_visable(windows: &mut Vec<&mut Window>) {
    for w in windows {
        w.visable = true;
    }
}

#[test]
fn empty_ws_should_not_contain_window() {
    let subject = Workspace::new();
    let w: Window = unsafe { std::mem::zeroed() };
    assert!(
        subject.is_displaying(&w) == false,
        "workspace incorrectly owns window"
    );
}

#[test]
fn tagging_a_workspace_to_with_the_same_tag_as_a_window_should_couse_it_to_display() {
    let mut subject = Workspace::new();
    subject.show_tag("test".to_owned());
    let mut w: Window = unsafe { std::mem::zeroed() };
    w.tag("test".to_owned());
    assert!(
        subject.is_displaying(&w) == true,
        "workspace should include window"
    );
}

#[test]
fn displayed_windows_should_return_a_list_of_display_windows() {
    let mut subject = Workspace::new();
    subject.show_tag("test".to_owned());
    let mut w: Window = unsafe { std::mem::zeroed() };
    w.tag("test".to_owned());
    let windows = vec![&mut w];
    assert!(
        subject.displayed_windows(windows).len() == 1,
        "workspace should include window"
    );
}

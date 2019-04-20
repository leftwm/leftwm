use super::layouts::*;
use crate::models::Screen;
use crate::models::Window;
use crate::models::XYHW;
use std::collections::VecDeque;
use std::fmt;

#[derive(Clone)]
pub struct Workspace {
    pub id: i32,
    layouts: VecDeque<Box<Layout>>,
    pub tags: Vec<String>,
    pub avoid: Vec<XYHW>,
    pub xyhw: XYHW,
    xyhw_avoided: XYHW,
}

impl fmt::Debug for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Workspace {{ id: {}, tags: {:?}, x: {}, y: {} }}",
            self.id, self.tags, self.xyhw.x, self.xyhw.y
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
            avoid: vec![],
            xyhw: XYHW {
                h: 600,
                w: 800,
                x: 0,
                y: 0,
            },
            xyhw_avoided: XYHW {
                h: 600,
                w: 800,
                x: 0,
                y: 0,
            },
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
            avoid: vec![],
            xyhw: XYHW {
                h: screen.height,
                w: screen.width,
                x: screen.x,
                y: screen.y,
            },
            xyhw_avoided: XYHW {
                h: screen.height,
                w: screen.width,
                x: screen.x,
                y: screen.y,
            },
        }
    }

    pub fn show_tag(&mut self, tag: String) {
        self.tags = vec![tag.clone()];
    }

    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        self.xyhw.contains_point(x, y)
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
    //pub fn displayed_windows<'a>(&self, windows: &mut Vec<&'a mut Window>) -> &mut Vec<&'a mut Window> {
    //    windows
    //        .into_iter()
    //        .filter(|w| self.is_displaying(w) && !w.floating())
    //        .collect::<&mut Vec<&mut Window>>()
    //}

    pub fn update_windows(&self, windows: &mut Vec<&mut Window>) {
        let mut mine: Vec<&mut &mut Window> = windows
            .iter_mut()
            .filter(|w| self.is_displaying(w) && !w.floating())
            .collect();
        mine.iter_mut().for_each(|w| w.set_visable(true));
        self.layouts[0].update_windows(self, &mut mine);
    }

    pub fn x(&self) -> i32 {
        self.xyhw_avoided.x
    }
    pub fn y(&self) -> i32 {
        self.xyhw_avoided.y
    }
    pub fn height(&self) -> i32 {
        self.xyhw_avoided.h
    }
    pub fn width(&self) -> i32 {
        self.xyhw_avoided.w
    }

    pub fn center_halfed(&self) -> XYHW {
        self.xyhw_avoided.center_halfed()
    }

    pub fn update_avoided_areas(&mut self) {
        let mut xyhw = self.xyhw;
        for a in &self.avoid {
            xyhw = xyhw.without(a);
        }
        self.xyhw_avoided = xyhw;
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

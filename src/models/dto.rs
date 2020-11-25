use crate::layouts::Layout;
use crate::models::Manager;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Viewport {
    pub tags: Vec<String>,
    pub h: u32,
    pub w: u32,
    pub x: i32,
    pub y: i32,
    pub layout: Layout,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManagerState {
    pub window_title: Option<String>,
    pub desktop_names: Vec<String>,
    pub viewports: Vec<Viewport>,
    pub active_desktop: Vec<String>,
    pub working_tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TagsForWorkspace {
    pub name: String,
    pub index: usize,
    pub mine: bool,
    pub visible: bool,
    pub focused: bool,
    pub busy: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DisplayWorkspace {
    pub h: u32,
    pub w: u32,
    pub x: i32,
    pub y: i32,
    pub layout: Layout,
    pub index: usize,
    pub tags: Vec<TagsForWorkspace>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DisplayState {
    pub window_title: String,
    pub workspaces: Vec<DisplayWorkspace>,
}

impl Into<DisplayState> for ManagerState {
    fn into(self) -> DisplayState {
        let visible: Vec<String> = self
            .viewports
            .iter()
            .flat_map(|vp| vp.tags.clone())
            .collect();
        let workspaces = self
            .viewports
            .iter()
            .enumerate()
            .map(|(i, vp)| {
                viewport_into_display_workspace(
                    &self.desktop_names,
                    &self.active_desktop,
                    &visible,
                    &self.working_tags,
                    &vp,
                    i,
                )
            })
            .collect();
        DisplayState {
            workspaces,
            window_title: self.window_title.unwrap_or_default(),
        }
    }
}

fn viewport_into_display_workspace(
    all_tags: &[String],
    focused: &[String],
    visible: &[String],
    working_tags: &[String],
    viewport: &Viewport,
    ws_index: usize,
) -> DisplayWorkspace {
    let tags: Vec<TagsForWorkspace> = all_tags
        .iter()
        .enumerate()
        .map(|(index, t)| TagsForWorkspace {
            name: t.clone(),
            index,
            mine: viewport.tags.contains(t),
            visible: visible.contains(t),
            focused: focused.contains(t),
            busy: working_tags.contains(t),
        })
        .collect();
    DisplayWorkspace {
        tags,
        h: viewport.h,
        w: viewport.w,
        x: viewport.x,
        y: viewport.y,
        layout: viewport.layout.clone(),
        index: ws_index,
    }
}

impl From<&Manager> for ManagerState {
    fn from(manager: &Manager) -> Self {
        let mut viewports: Vec<Viewport> = vec![];
        let working_tags = manager
            .tags
            .iter()
            .filter(|tag| manager.windows.iter().any(|w| w.has_tag(tag.to_string())))
            .map(|tag| tag.clone())
            .collect();
        for ws in &manager.workspaces {
            viewports.push(Viewport {
                tags: ws.tags.clone(),
                x: ws.xyhw.x(),
                y: ws.xyhw.y(),
                h: ws.xyhw.h() as u32,
                w: ws.xyhw.w() as u32,
                layout: ws.layout.clone(),
            });
        }
        let active_desktop = match manager.focused_workspace() {
            Some(ws) => ws.tags.clone(),
            None => vec!["".to_owned()],
        };
        let window_title = match manager.focused_window() {
            Some(win) => win.name.clone(),
            None => None,
        };
        ManagerState {
            window_title,
            desktop_names: manager.tags.clone(),
            viewports,
            active_desktop,
            working_tags,
        }
    }
}

use crate::config::Config;
use crate::display_servers::DisplayServer;
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

#[allow(clippy::struct_excessive_bools)]
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

impl From<ManagerState> for DisplayState {
    fn from(m: ManagerState) -> Self {
        let visible: Vec<String> = m.viewports.iter().flat_map(|vp| vp.tags.clone()).collect();
        let workspaces = m
            .viewports
            .iter()
            .enumerate()
            .map(|(i, vp)| {
                viewport_into_display_workspace(
                    &m.desktop_names,
                    &m.active_desktop,
                    &visible,
                    &m.working_tags,
                    vp,
                    i,
                )
            })
            .collect();
        Self {
            workspaces,
            window_title: m.window_title.unwrap_or_default(),
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
        layout: viewport.layout,
        index: ws_index,
    }
}

impl<C: Config, SERVER: DisplayServer> From<&Manager<C, SERVER>> for ManagerState {
    fn from(manager: &Manager<C, SERVER>) -> Self {
        let mut viewports: Vec<Viewport> = vec![];
        let mut tags_len = manager.state.tags.len();
        tags_len = if tags_len == 0 { 0 } else { tags_len - 1 };
        let working_tags = manager.state.tags[0..tags_len]
            .iter()
            .filter(|tag| manager.state.windows.iter().any(|w| w.has_tag(&tag.id)))
            .map(|t| t.id.clone())
            .collect();
        for ws in &manager.state.workspaces {
            viewports.push(Viewport {
                tags: ws.tags.clone(),
                x: ws.xyhw.x(),
                y: ws.xyhw.y(),
                h: ws.xyhw.h() as u32,
                w: ws.xyhw.w() as u32,
                layout: ws.layout,
            });
        }
        let active_desktop = match manager
            .state
            .focus_manager
            .workspace(&manager.state.workspaces)
        {
            Some(ws) => ws.tags.clone(),
            None => vec!["".to_owned()],
        };
        let window_title = match manager.state.focus_manager.window(&manager.state.windows) {
            Some(win) => win.name.clone(),
            None => None,
        };
        Self {
            window_title,
            desktop_names: manager.state.tags[0..tags_len]
                .iter()
                .map(|t| t.id.clone())
                .collect(),
            viewports,
            active_desktop,
            working_tags,
        }
    }
}

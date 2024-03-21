use crate::state::State;
use serde::{Deserialize, Serialize};

use super::Handle;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Viewport {
    pub id: usize,
    pub output: String,
    pub tag: String,
    pub h: u32,
    pub w: u32,
    pub x: i32,
    pub y: i32,
    pub layout: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ManagerState {
    pub window_title: Option<String>,
    pub desktop_names: Vec<String>,
    pub viewports: Vec<Viewport>,
    pub active_desktop: Vec<String>,
    pub working_tags: Vec<String>,
    pub urgent_tags: Vec<String>,
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TagsForWorkspace {
    pub name: String,
    pub index: usize,
    pub mine: bool,
    pub visible: bool,
    pub focused: bool,
    pub urgent: bool,
    pub busy: bool,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DisplayWorkspace {
    pub id: usize,
    pub output: String,
    pub h: u32,
    pub w: u32,
    pub x: i32,
    pub y: i32,
    pub layout: String,
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
        let visible: Vec<String> = m.viewports.iter().map(|vp| vp.tag.clone()).collect();
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
                    &m.urgent_tags,
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
    urgent_tags: &[String],
    viewport: &Viewport,
    ws_index: usize,
) -> DisplayWorkspace {
    let tags: Vec<TagsForWorkspace> = all_tags
        .iter()
        .enumerate()
        .map(|(index, t)| TagsForWorkspace {
            name: t.clone(),
            index,
            mine: viewport.tag == *t,
            visible: visible.contains(t),
            focused: focused.contains(t),
            urgent: urgent_tags.contains(t),
            busy: working_tags.contains(t),
        })
        .collect();
    DisplayWorkspace {
        id: viewport.id,
        output: viewport.output.clone(),
        tags,
        h: viewport.h,
        w: viewport.w,
        x: viewport.x,
        y: viewport.y,
        index: ws_index,
        layout: viewport.layout.clone(),
    }
}

impl<H: Handle> From<&State<H>> for ManagerState {
    fn from(state: &State<H>) -> Self {
        let mut viewports: Vec<Viewport> = vec![];
        // tags_len = if tags_len == 0 { 0 } else { tags_len - 1 };
        let working_tags = state
            .tags
            .all()
            .iter()
            .filter(|tag| state.windows.iter().any(|w| w.has_tag(&tag.id)))
            .map(|t| t.label.clone())
            .collect();
        let urgent_tags = state
            .tags
            .all()
            .iter()
            .filter(|tag| state.windows.iter().any(|w| w.has_tag(&tag.id) && w.urgent))
            .map(|t| t.label.clone())
            .collect();
        for ws in &state.workspaces {
            let tag_label = ws
                .tag
                .map(|tag_id| state.tags.get(tag_id).map(|tag| tag.label.clone()))
                .unwrap()
                .unwrap();

            let layout_name: String = ws
                .tag
                .and_then(|tagid| state.layout_manager.layout_maybe(ws.id, tagid))
                .map_or_else(|| String::from("N/A"), |layout| layout.name.clone());

            let output = state
                .screens
                .iter()
                .find(|s| s.id == Some(ws.id))
                .map_or_else(
                    || String::from("Not found (unreachable)"),
                    |s| s.output.clone(),
                );

            viewports.push(Viewport {
                id: ws.id,
                output,
                tag: tag_label,
                x: ws.xyhw.x(),
                y: ws.xyhw.y(),
                h: ws.xyhw.h() as u32,
                w: ws.xyhw.w() as u32,
                layout: layout_name,
            });
        }
        let active_desktop = match state.focus_manager.workspace(&state.workspaces) {
            Some(ws) => ws
                .tag
                .iter()
                .map(|&tag_id| state.tags.get(tag_id).unwrap().label.clone())
                .collect(),
            None => vec![], // todo ??
        };
        let window_title = match state.focus_manager.window(&state.windows) {
            Some(win) => win.name.clone(),
            None => None,
        };
        Self {
            window_title,
            desktop_names: state
                .tags
                .normal()
                .iter()
                .map(|t| t.label.clone())
                .collect(),
            viewports,
            active_desktop,
            urgent_tags,
            working_tags,
        }
    }
}

use super::{window::Handle, DockArea, WindowHandle, WorkspaceId};
use crate::config::Workspace;
use serde::{Deserialize, Serialize};
use std::convert::From;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Screen<H: Handle> {
    #[serde(bound = "")]
    pub root: WindowHandle<H>,
    pub output: String,
    pub id: Option<WorkspaceId>,
    pub bbox: BBox,
}

/// Screen Bounding Box
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl<H: Handle> Screen<H> {
    #[must_use]
    pub fn new(bbox: BBox, output: String) -> Self {
        Self {
            root: WindowHandle::<H>(H::default()),
            output,
            bbox,
            id: None,
        }
    }

    #[must_use]
    pub const fn contains_point(&self, x: i32, y: i32) -> bool {
        let bbox = &self.bbox;
        let max_x = bbox.x + bbox.width;
        let max_y = bbox.y + bbox.height;
        (bbox.x <= x && x <= max_x) && (bbox.y <= y && y <= max_y)
    }

    #[must_use]
    pub const fn contains_dock_area(&self, dock_area: DockArea, screens_area: (i32, i32)) -> bool {
        if dock_area.top > 0 {
            return self.contains_point(dock_area.top_start_x, dock_area.top);
        }
        if dock_area.bottom > 0 {
            return self
                .contains_point(dock_area.bottom_start_x, screens_area.0 - dock_area.bottom);
        }
        if dock_area.left > 0 {
            return self.contains_point(dock_area.left, dock_area.left_start_y);
        }
        if dock_area.right > 0 {
            return self.contains_point(screens_area.1 - dock_area.right, dock_area.right_start_y);
        }
        false
    }
}

impl BBox {
    pub fn add(&mut self, bbox: BBox) {
        self.x += bbox.x;
        self.y += bbox.y;
        self.width += bbox.width;
        self.height += bbox.height;
    }
}

impl<H: Handle> From<&Workspace> for Screen<H> {
    fn from(wsc: &Workspace) -> Self {
        Self {
            bbox: BBox {
                height: wsc.height,
                width: wsc.width,
                x: wsc.x,
                y: wsc.y,
            },
            output: wsc.output.clone(),
            ..Default::default()
        }
    }
}

impl<H: Handle> Default for Screen<H> {
    fn default() -> Self {
        Self {
            root: WindowHandle::<H>(H::default()),
            output: String::default(),
            id: None,
            bbox: BBox {
                height: 600,
                width: 800,
                x: 0,
                y: 0,
            },
        }
    }
}

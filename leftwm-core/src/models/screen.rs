use super::{DockArea, Size, WindowHandle, WorkspaceId};
use crate::config::Workspace;
use serde::{Deserialize, Serialize};
use std::convert::From;
use x11_dl::xlib;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Screen {
    pub root: WindowHandle,
    pub output: String,
    pub id: Option<WorkspaceId>,
    pub bbox: BBox,
    pub max_window_width: Option<Size>,
}

/// Screen Bounding Box
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct BBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Screen {
    #[must_use]
    pub const fn new(bbox: BBox, output: String) -> Self {
        Self {
            root: WindowHandle::MockHandle(0),
            output,
            bbox,
            max_window_width: None,
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

impl From<&Workspace> for Screen {
    fn from(wsc: &Workspace) -> Self {
        Self {
            bbox: BBox {
                height: wsc.height,
                width: wsc.width,
                x: wsc.x,
                y: wsc.y,
            },
            output: wsc.output.clone(),
            max_window_width: wsc.max_window_width,
            ..Default::default()
        }
    }
}

impl From<x11_dl::xrandr::XRRCrtcInfo> for Screen {
    fn from(root: x11_dl::xrandr::XRRCrtcInfo) -> Self {
        Self {
            bbox: BBox {
                x: root.x,
                y: root.y,
                width: root.width as i32,
                height: root.height as i32,
            },
            ..Default::default()
        }
    }
}

impl From<&xlib::XWindowAttributes> for Screen {
    fn from(root: &xlib::XWindowAttributes) -> Self {
        Self {
            root: root.root.into(),
            bbox: BBox {
                height: root.height,
                width: root.width,
                x: root.x,
                y: root.y,
            },
            ..Default::default()
        }
    }
}

impl From<&x11_dl::xinerama::XineramaScreenInfo> for Screen {
    fn from(root: &x11_dl::xinerama::XineramaScreenInfo) -> Self {
        Self {
            bbox: BBox {
                height: root.height.into(),
                width: root.width.into(),
                x: root.x_org.into(),
                y: root.y_org.into(),
            },
            ..Default::default()
        }
    }
}

impl Default for Screen {
    fn default() -> Self {
        Self {
            root: WindowHandle::MockHandle(0),
            output: String::default(),
            id: None,
            bbox: BBox {
                height: 600,
                width: 800,
                x: 0,
                y: 0,
            },
            max_window_width: None,
        }
    }
}

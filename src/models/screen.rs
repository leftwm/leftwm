use super::WindowHandle;
use crate::config::WorkspaceConfig;
use serde::{Deserialize, Serialize};
use std::convert::From;
use x11_dl::xlib;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Screen {
    pub root: WindowHandle,
    #[serde(flatten)]
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

impl Screen {
    #[must_use]
    pub fn new(bbox: BBox) -> Screen {
        Screen {
            root: WindowHandle::MockHandle(0),
            bbox,
        }
    }

    #[must_use]
    pub fn contains_point(&self, x: i32, y: i32) -> bool {
        let bbox = &self.bbox;
        let max_x = bbox.x + bbox.width;
        let max_y = bbox.y + bbox.height;
        (bbox.x <= x && x <= max_x) && (bbox.y <= y && y <= max_y)
    }
}

impl From<&WorkspaceConfig> for Screen {
    fn from(wsc: &WorkspaceConfig) -> Self {
        Screen {
            root: WindowHandle::MockHandle(0),
            bbox: BBox {
                height: wsc.height,
                width: wsc.width,
                x: wsc.x,
                y: wsc.y,
            },
        }
    }
}

impl From<&xlib::XWindowAttributes> for Screen {
    fn from(root: &xlib::XWindowAttributes) -> Self {
        Screen {
            root: WindowHandle::XlibHandle(root.root),
            bbox: BBox {
                height: root.height,
                width: root.width,
                x: root.x,
                y: root.y,
            },
        }
    }
}

impl From<&x11_dl::xinerama::XineramaScreenInfo> for Screen {
    fn from(root: &x11_dl::xinerama::XineramaScreenInfo) -> Self {
        Screen {
            root: WindowHandle::MockHandle(0),
            bbox: BBox {
                height: root.height.into(),
                width: root.width.into(),
                x: root.x_org.into(),
                y: root.y_org.into(),
            },
        }
    }
}

impl Default for Screen {
    fn default() -> Self {
        Screen {
            root: WindowHandle::MockHandle(0),
            bbox: BBox {
                height: 600,
                width: 800,
                x: 0,
                y: 0,
            },
        }
    }
}

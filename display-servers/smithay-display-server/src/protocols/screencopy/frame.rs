use std::time::UNIX_EPOCH;

use smithay::{
    output::Output,
    reexports::{
        wayland_protocols_wlr::screencopy::v1::server::{
            zwlr_screencopy_frame_v1::Request,
            zwlr_screencopy_frame_v1::{Flags, ZwlrScreencopyFrameV1},
        },
        wayland_server::{
            protocol::wl_buffer::WlBuffer, Client, DataInit, Dispatch, DisplayHandle,
        },
    },
    utils::{Physical, Rectangle},
};

use super::{ScreencopyHandler, ScreencopyManagerState};

pub struct ScreencopyFrameState {
    pub rect: Rectangle<i32, Physical>,
    pub overlay_cursor: bool,
    pub output: Output,
}

impl<D> Dispatch<ZwlrScreencopyFrameV1, ScreencopyFrameState, D> for ScreencopyManagerState
where
    D: Dispatch<ZwlrScreencopyFrameV1, ScreencopyFrameState>,
    D: ScreencopyHandler,
    D: 'static,
{
    fn request(
        state: &mut D,
        _client: &Client,
        frame: &ZwlrScreencopyFrameV1,
        request: Request,
        data: &ScreencopyFrameState,
        _display: &DisplayHandle,
        _data_init: &mut DataInit<'_, D>,
    ) {
        let (buffer, send_damage) = match request {
            Request::Copy { buffer } => (buffer, false),
            Request::CopyWithDamage { buffer } => (buffer, true),
            Request::Destroy => return,
            _ => unreachable!(),
        };

        state.frame(Screencopy {
            send_damage,
            buffer,
            frame: frame.clone(),
            region: data.rect,
            submitted: false,
            output: data.output.clone(),
            overlay_cursor: data.overlay_cursor,
        });
    }
}

/// Screencopy frame.
pub struct Screencopy {
    region: Rectangle<i32, Physical>,
    frame: ZwlrScreencopyFrameV1,
    send_damage: bool,
    buffer: WlBuffer,
    submitted: bool,
    pub output: Output,
    pub overlay_cursor: bool,
}

impl Drop for Screencopy {
    fn drop(&mut self) {
        if !self.submitted {
            self.frame.failed();
        }
    }
}

impl Screencopy {
    /// Get the target buffer to copy to.
    pub fn buffer(&self) -> &WlBuffer {
        &self.buffer
    }

    /// Get the region which should be copied.
    pub fn region(&self) -> Rectangle<i32, Physical> {
        self.region
    }

    /// Mark damaged regions of the screencopy buffer.
    pub fn damage(&mut self, damage: &[Rectangle<i32, Physical>]) {
        if !self.send_damage {
            return;
        }

        for Rectangle { loc, size } in damage {
            self.frame
                .damage(loc.x as u32, loc.y as u32, size.w as u32, size.h as u32);
        }
    }

    /// Submit the copied content.
    pub fn submit(mut self) {
        // Notify client that buffer is ordinary.
        self.frame.flags(Flags::empty());

        // Notify client about successful copy.
        let now = UNIX_EPOCH.elapsed().unwrap();
        let secs = now.as_secs();
        self.frame
            .ready((secs >> 32) as u32, secs as u32, now.subsec_nanos());

        // Mark frame as submitted to ensure destructor isn't run.
        self.submitted = true;
    }

    pub fn failed(self) {
        self.frame.failed();
    }
}

use smithay::{output::Output, reexports::wayland_server::protocol::wl_output::WlOutput};

use crate::{
    delegate_screencopy_manager,
    protocols::screencopy::{frame::Screencopy, ScreencopyHandler},
    state::SmithayState,
};

impl ScreencopyHandler for SmithayState {
    fn output(&mut self, output: &WlOutput) -> &Output {
        self.outputs
            .iter()
            .find(|(o, _)| o.owns(output))
            .map(|(o, _)| o)
            .unwrap()
    }

    fn frame(&mut self, frame: Screencopy) {
        for (node, device) in &self.udev_data.devices {
            for (crtc, surface) in &device.surfaces {
                if surface.output == frame.output {
                    self.render(*node, *crtc, Some(frame)).unwrap();
                    return;
                }
            }
        }
    }
}
delegate_screencopy_manager!(SmithayState);

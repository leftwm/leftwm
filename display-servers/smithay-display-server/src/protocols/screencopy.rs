pub mod frame;

use smithay::{
    output::Output,
    reexports::{
        wayland_protocols_wlr::screencopy::v1::server::{
            zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1, zwlr_screencopy_manager_v1::Request,
            zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
        },
        wayland_server::{
            protocol::{wl_output::WlOutput, wl_shm},
            Client, DataInit, Dispatch, DisplayHandle, GlobalDispatch, New, Resource,
        },
    },
    utils::Rectangle,
};

use self::frame::{Screencopy, ScreencopyFrameState};

const MANAGER_VERSION: u32 = 3;

pub struct ScreencopyManagerState;

impl ScreencopyManagerState {
    pub fn new<D>(display: &DisplayHandle) -> Self
    where
        D: GlobalDispatch<ZwlrScreencopyManagerV1, ()>,
        D: Dispatch<ZwlrScreencopyManagerV1, ()>,
        D: Dispatch<ZwlrScreencopyFrameV1, ScreencopyFrameState>,
        D: ScreencopyHandler,
        D: 'static,
    {
        display.create_global::<D, ZwlrScreencopyManagerV1, _>(MANAGER_VERSION, ());

        Self
    }
}

impl<D> GlobalDispatch<ZwlrScreencopyManagerV1, (), D> for ScreencopyManagerState
where
    D: GlobalDispatch<ZwlrScreencopyManagerV1, ()>,
    D: Dispatch<ZwlrScreencopyManagerV1, ()>,
    D: Dispatch<ZwlrScreencopyFrameV1, ScreencopyFrameState>,
    D: ScreencopyHandler,
    D: 'static,
{
    fn bind(
        _state: &mut D,
        _display: &DisplayHandle,
        _client: &Client,
        manager: New<ZwlrScreencopyManagerV1>,
        _manager_state: &(),
        data_init: &mut DataInit<'_, D>,
    ) {
        data_init.init(manager, ());
    }
}

impl<D> Dispatch<ZwlrScreencopyManagerV1, (), D> for ScreencopyManagerState
where
    D: GlobalDispatch<ZwlrScreencopyManagerV1, ()>,
    D: Dispatch<ZwlrScreencopyManagerV1, ()>,
    D: Dispatch<ZwlrScreencopyFrameV1, ScreencopyFrameState>,
    D: ScreencopyHandler,
    D: 'static,
{
    fn request(
        state: &mut D,
        _client: &Client,
        manager: &ZwlrScreencopyManagerV1,
        request: Request,
        _data: &(),
        _display: &DisplayHandle,
        data_init: &mut DataInit<'_, D>,
    ) {
        let (frame, overlay_cursor, rect, output) = match request {
            Request::CaptureOutput {
                frame,
                overlay_cursor,
                output,
            } => {
                let output = state.output(&output);
                let rect =
                    Rectangle::from_loc_and_size((0, 0), output.current_mode().unwrap().size);
                (frame, overlay_cursor, rect, output.clone())
            }
            Request::CaptureOutputRegion {
                frame,
                overlay_cursor,
                x,
                y,
                width,
                height,
                output,
            } => {
                let rect = Rectangle::from_loc_and_size((x, y), (width, height));

                let output = state.output(&output);
                let output_transform = output.current_transform();
                let rotated_rect =
                    output_transform.transform_rect_in(rect, &output.current_mode().unwrap().size);

                let clamped_rect = rotated_rect
                    .intersection(Rectangle::from_loc_and_size(
                        (0, 0),
                        output.current_mode().unwrap().size,
                    ))
                    .unwrap_or_default();

                (frame, overlay_cursor, clamped_rect, output.clone())
            }
            Request::Destroy => return,
            _ => unreachable!(),
        };

        let overlay_cursor = overlay_cursor != 0;
        let frame = data_init.init(
            frame,
            ScreencopyFrameState {
                overlay_cursor,
                rect,
                output,
            },
        );

        frame.buffer(
            wl_shm::Format::Argb8888,
            rect.size.w as u32,
            rect.size.h as u32,
            rect.size.w as u32 * 4,
        );

        if manager.version() >= 3 {
            frame.buffer_done();
        }
    }
}

pub trait ScreencopyHandler {
    fn output(&mut self, output: &WlOutput) -> &Output;

    fn frame(&mut self, frame: Screencopy);
}

#[allow(missing_docs)]
#[macro_export]
macro_rules! delegate_screencopy_manager {
    ($(@<$( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+>)? $ty: ty) => {
        smithay::reexports::wayland_server::delegate_global_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            smithay::reexports::wayland_protocols_wlr::screencopy::v1::server::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1: ()
        ] => $crate::protocols::screencopy::ScreencopyManagerState);

        smithay::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            smithay::reexports::wayland_protocols_wlr::screencopy::v1::server::zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1: ()
        ] => $crate::protocols::screencopy::ScreencopyManagerState);

        smithay::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            smithay::reexports::wayland_protocols_wlr::screencopy::v1::server::zwlr_screencopy_frame_v1::ZwlrScreencopyFrameV1: $crate::protocols::screencopy::frame::ScreencopyFrameState
        ] => $crate::protocols::screencopy::ScreencopyManagerState);
    };
}

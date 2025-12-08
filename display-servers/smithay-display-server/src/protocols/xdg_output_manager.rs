use smithay::{
    output::Output,
    reexports::{
        wayland_protocols::xdg::xdg_output::zv1::server::{
            zxdg_output_manager_v1::Request, zxdg_output_manager_v1::ZxdgOutputManagerV1,
            zxdg_output_v1::ZxdgOutputV1,
        },
        wayland_server::{
            protocol::wl_output::WlOutput, Client, DataInit, Dispatch, DisplayHandle,
            GlobalDispatch, New,
        },
    },
    utils::{Logical, Rectangle},
};

const MANAGER_VERSION: u32 = 3;

pub struct XdgOutputManagerState;

impl XdgOutputManagerState {
    pub fn new<D>(display: &DisplayHandle) -> Self
    where
        D: GlobalDispatch<ZxdgOutputManagerV1, ()>,
        D: Dispatch<ZxdgOutputManagerV1, ()>,
        D: Dispatch<ZxdgOutputV1, ()>,
        D: XdgOutputHandler,
        D: 'static,
    {
        display.create_global::<D, ZxdgOutputManagerV1, _>(MANAGER_VERSION, ());

        Self
    }
}

impl<D> GlobalDispatch<ZxdgOutputManagerV1, (), D> for XdgOutputManagerState
where
    D: GlobalDispatch<ZxdgOutputManagerV1, ()>,
    D: Dispatch<ZxdgOutputManagerV1, ()>,
    D: Dispatch<ZxdgOutputV1, ()>,
    D: XdgOutputHandler,
    D: 'static,
{
    fn bind(
        _state: &mut D,
        _handle: &DisplayHandle,
        _client: &Client,
        manager: New<ZxdgOutputManagerV1>,
        _global_data: &(),
        data_init: &mut DataInit<'_, D>,
    ) {
        data_init.init(manager, ());
    }
}

impl<D> Dispatch<ZxdgOutputManagerV1, (), D> for XdgOutputManagerState
where
    D: GlobalDispatch<ZxdgOutputManagerV1, ()>,
    D: Dispatch<ZxdgOutputManagerV1, ()>,
    D: Dispatch<ZxdgOutputV1, ()>,
    D: XdgOutputHandler,
    D: 'static,
{
    fn request(
        state: &mut D,
        _client: &Client,
        _manager: &ZxdgOutputManagerV1,
        request: Request,
        _data: &(),
        _dhandle: &DisplayHandle,
        data_init: &mut DataInit<'_, D>,
    ) {
        match request {
            Request::Destroy => return,
            Request::GetXdgOutput { id, output } => {
                let id = data_init.init(id, ());
                let (output, geometry) = state.output(&output);
                id.logical_size(geometry.size.w, geometry.size.h);
                id.logical_position(geometry.loc.x, geometry.loc.y);
                id.name(output.name());
                id.description(output.description());
                id.done();
            }
            _ => unreachable!(),
        }
    }
}

impl<D> Dispatch<ZxdgOutputV1, (), D> for XdgOutputManagerState
where
    D: GlobalDispatch<ZxdgOutputManagerV1, ()>,
    D: Dispatch<ZxdgOutputManagerV1, ()>,
    D: Dispatch<ZxdgOutputV1, ()>,
    D: XdgOutputHandler,
    D: 'static,
{
    fn request(
        _state: &mut D,
        _client: &Client,
        _resource: &ZxdgOutputV1,
        request: <ZxdgOutputV1 as smithay::reexports::wayland_server::Resource>::Request,
        _data: &(),
        _dhandle: &DisplayHandle,
        _data_init: &mut DataInit<'_, D>,
    ) {
        match request {
            smithay::reexports::wayland_protocols::xdg::xdg_output::zv1::server::zxdg_output_v1::Request::Destroy => return,
            _ => unreachable!(),
        }
    }
}

pub trait XdgOutputHandler {
    fn output(&mut self, output: &WlOutput) -> &(Output, Rectangle<i32, Logical>);
}

#[allow(missing_docs)]
#[macro_export]
macro_rules! delegate_xdg_output_handler {
    ($(@<$( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+>)? $ty: ty) => {
        // smithay::reexports::wayland_server::delegate_global_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
        //     smithay::reexports::wayland_protocols::xdg::xdg_output::zv1::server::zxdg_output_manager_v1::ZxdgOutputManagerV1: ()
        // ] => $crate::protocols::xdg_output_manager::XdgOutputManagerState);

        // smithay::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
        //     smithay::reexports::wayland_protocols::xdg::xdg_output::zv1::server::zxdg_output_manager_v1::ZxdgOutputManagerV1: ()
        // ] => $crate::protocols::xdg_output_manager::XdgOutputManagerState);

        smithay::reexports::wayland_server::delegate_dispatch!($(@< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)? $ty: [
            smithay::reexports::wayland_protocols::xdg::xdg_output::zv1::server::zxdg_output_v1::ZxdgOutputV1: ()
        ] => $crate::protocols::xdg_output_manager::XdgOutputManagerState);
    };
}

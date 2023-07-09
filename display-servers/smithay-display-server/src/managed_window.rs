use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use smithay::{
    backend::renderer::{
        element::{surface::WaylandSurfaceRenderElement, AsRenderElements},
        gles::element::PixelShaderElement,
        ImportAll, Renderer,
    },
    desktop::{space::SpaceElement, utils::OutputPresentationFeedback, Window},
    input::{keyboard::KeyboardTarget, pointer::PointerTarget},
    output::Output,
    reexports::{
        wayland_protocols::wp::presentation_time::server::wp_presentation_feedback::Kind,
        wayland_server::protocol::wl_surface::WlSurface,
    },
    utils::{IsAlive, Logical, Point, Rectangle},
    wayland::{
        compositor::{self, SurfaceData},
        dmabuf::DmabufFeedback,
        seat::WaylandFocus,
        shell::xdg::ToplevelSurface,
    },
};

use crate::{
    drawing::border::{BorderRenderer, WindowState},
    leftwm_config::BorderConfig,
    state::SmithayState,
    udev::rendering::AsGlowRenderer,
    window_registry::WindowHandle,
};

#[derive(PartialEq, Clone, Debug, Default)]
pub struct ManagedWindowData {
    pub managed: bool,
    pub floating: bool,
    pub visible: bool,
    pub geometry: Option<Rectangle<i32, Logical>>,
}

#[derive(Clone, Debug)]
pub struct ManagedWindow {
    pub window: Window,
    handle: Option<WindowHandle>,
    pub data: Arc<RwLock<ManagedWindowData>>,
}

impl PartialEq for ManagedWindow {
    fn eq(&self, other: &Self) -> bool {
        // We assume that if both windows have a handle and they are the same the windows should be
        // the same
        self.handle
            .is_some_and(|h1| other.handle.is_some_and(|h2| h2 == h1))
    }
}

impl IsAlive for ManagedWindow {
    fn alive(&self) -> bool {
        self.window.alive()
    }
}

impl WaylandFocus for ManagedWindow {
    fn wl_surface(&self) -> Option<WlSurface> {
        self.window.wl_surface()
    }
}

impl KeyboardTarget<SmithayState> for ManagedWindow {
    fn enter(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        keys: Vec<smithay::input::keyboard::KeysymHandle<'_>>,
        serial: smithay::utils::Serial,
    ) {
        KeyboardTarget::enter(&self.window, seat, data, keys, serial);
    }

    fn leave(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        serial: smithay::utils::Serial,
    ) {
        KeyboardTarget::leave(&self.window, seat, data, serial);
    }

    fn key(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        key: smithay::input::keyboard::KeysymHandle<'_>,
        state: smithay::backend::input::KeyState,
        serial: smithay::utils::Serial,
        time: u32,
    ) {
        self.window.key(seat, data, key, state, serial, time);
    }

    fn modifiers(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        modifiers: smithay::input::keyboard::ModifiersState,
        serial: smithay::utils::Serial,
    ) {
        self.window.modifiers(seat, data, modifiers, serial);
    }
}

impl PointerTarget<SmithayState> for ManagedWindow {
    fn enter(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        PointerTarget::enter(&self.window, seat, data, event);
    }

    fn motion(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        self.window.motion(seat, data, event);
    }

    fn relative_motion(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        event: &smithay::input::pointer::RelativeMotionEvent,
    ) {
        self.window.relative_motion(seat, data, event);
    }

    fn button(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        event: &smithay::input::pointer::ButtonEvent,
    ) {
        self.window.button(seat, data, event);
    }

    fn axis(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        frame: smithay::input::pointer::AxisFrame,
    ) {
        self.window.axis(seat, data, frame);
    }

    fn leave(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        serial: smithay::utils::Serial,
        time: u32,
    ) {
        PointerTarget::leave(&self.window, seat, data, serial, time);
    }
}

impl ManagedWindow {
    pub fn new(window: Window) -> Self {
        Self {
            window,
            data: Arc::new(RwLock::new(ManagedWindowData::default())),
            handle: None,
        }
    }

    pub fn render_elements<C, R>(
        &self,
        renderer: &mut R,
        focused_window: &Option<WindowHandle>,
        borders: &BorderConfig,
        location: Point<i32, smithay::utils::Physical>,
        scale: smithay::utils::Scale<f64>,
        alpha: f32,
    ) -> Vec<C>
    where
        C: From<WaylandSurfaceRenderElement<R>> + From<PixelShaderElement>,
        R: Renderer + ImportAll + AsGlowRenderer,
        <R as Renderer>::TextureId: 'static,
    {
        let mut elements = Vec::new();

        // borders
        if self.handle == *focused_window {
            elements.push(C::from(BorderRenderer::render_element(
                renderer.glow_renderer_mut(),
                self,
                borders,
                WindowState::Focused,
                self.data.read().unwrap().geometry.unwrap().loc,
            )));
        } else if self.data.read().unwrap().floating {
            elements.push(C::from(BorderRenderer::render_element(
                renderer.glow_renderer_mut(),
                self,
                borders,
                WindowState::Floating,
                self.data.read().unwrap().geometry.unwrap().loc,
            )));
        } else {
            elements.push(C::from(BorderRenderer::render_element(
                renderer.glow_renderer_mut(),
                self,
                borders,
                WindowState::Default,
                self.data.read().unwrap().geometry.unwrap().loc,
            )));
        }

        elements.append(
            &mut self
                .window
                .render_elements(renderer, location, scale, alpha),
        );

        elements.reverse();
        elements
    }

    /// Sets the window handle only if the current handle is `None`
    pub fn set_handle(&mut self, handle: WindowHandle) {
        if self.handle.is_none() {
            self.handle = Some(handle);
        }
    }

    pub fn get_handle(&self) -> Option<WindowHandle> {
        self.handle
    }

    pub fn toplevel(&self) -> &ToplevelSurface {
        self.window.toplevel()
    }

    pub fn on_commit(&self) {
        self.window.on_commit()
    }

    pub fn set_geometry(&mut self, geometry: Rectangle<i32, Logical>) {
        self.data.write().unwrap().geometry = Some(geometry);
        self.window
            .toplevel()
            .with_pending_state(|state| state.size = Some(geometry.size));
        self.window.toplevel().send_configure();
    }

    pub fn send_frame<T, F>(
        &self,
        output: &Output,
        time: T,
        throttle: Option<Duration>,
        primary_scan_out_output: F,
    ) where
        T: Into<Duration>,
        F: FnMut(&WlSurface, &compositor::SurfaceData) -> Option<Output> + Copy,
    {
        self.window
            .send_frame(output, time, throttle, primary_scan_out_output)
    }

    pub fn _send_dmabuf_feedback<'a, P, F>(
        &self,
        output: &Output,
        primary_scan_out_output: P,
        select_dmabuf_feedback: F,
    ) where
        P: FnMut(&WlSurface, &compositor::SurfaceData) -> Option<Output> + Copy,
        F: Fn(&WlSurface, &compositor::SurfaceData) -> &'a DmabufFeedback + Copy,
    {
        self.window
            .send_dmabuf_feedback(output, primary_scan_out_output, select_dmabuf_feedback)
    }

    pub fn take_presentation_feedback<F1, F2>(
        &self,
        output_feedback: &mut OutputPresentationFeedback,
        primary_scan_out_output: F1,
        presentation_feedback_flags: F2,
    ) where
        F1: FnMut(&WlSurface, &SurfaceData) -> Option<Output> + Copy,
        F2: FnMut(&WlSurface, &SurfaceData) -> Kind + Copy,
    {
        self.window.take_presentation_feedback(
            output_feedback,
            primary_scan_out_output,
            presentation_feedback_flags,
        )
    }
}

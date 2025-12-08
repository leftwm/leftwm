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
    desktop::{utils::OutputPresentationFeedback, LayerSurface, Window},
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

#[derive(Clone, Debug)]
enum InnerManagedWindow {
    Window(Window),
    Surface(LayerSurface),
}

#[derive(PartialEq, Clone, Debug, Default)]
pub struct ManagedWindowData {
    pub managed: bool,
    pub floating: bool,
    pub visible: bool,
    pub geometry: Option<Rectangle<i32, Logical>>,
}

#[derive(Clone, Debug)]
pub struct ManagedWindow {
    window: InnerManagedWindow,
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
        match &self.window {
            InnerManagedWindow::Window(w) => w.alive(),
            InnerManagedWindow::Surface(s) => s.alive(),
        }
    }
}

impl WaylandFocus for ManagedWindow {
    fn wl_surface(&self) -> Option<WlSurface> {
        match &self.window {
            InnerManagedWindow::Window(w) => w.wl_surface(),
            InnerManagedWindow::Surface(s) => Some(s.wl_surface().clone()),
        }
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
        match &self.window {
            InnerManagedWindow::Window(w) => KeyboardTarget::enter(w, seat, data, keys, serial),
            InnerManagedWindow::Surface(s) => {
                KeyboardTarget::enter(s.wl_surface(), seat, data, keys, serial)
            }
        }
    }

    fn leave(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        serial: smithay::utils::Serial,
    ) {
        match &self.window {
            InnerManagedWindow::Window(w) => KeyboardTarget::leave(w, seat, data, serial),
            InnerManagedWindow::Surface(s) => {
                KeyboardTarget::leave(s.wl_surface(), seat, data, serial)
            }
        }
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
        match &self.window {
            InnerManagedWindow::Window(w) => w.key(seat, data, key, state, serial, time),
            InnerManagedWindow::Surface(s) => {
                s.wl_surface().key(seat, data, key, state, serial, time)
            }
        }
    }

    fn modifiers(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        modifiers: smithay::input::keyboard::ModifiersState,
        serial: smithay::utils::Serial,
    ) {
        match &self.window {
            InnerManagedWindow::Window(w) => w.modifiers(seat, data, modifiers, serial),
            InnerManagedWindow::Surface(s) => {
                s.wl_surface().modifiers(seat, data, modifiers, serial)
            }
        }
    }
}

impl PointerTarget<SmithayState> for ManagedWindow {
    fn enter(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        match &self.window {
            InnerManagedWindow::Window(w) => PointerTarget::enter(w, seat, data, event),
            InnerManagedWindow::Surface(s) => {
                PointerTarget::enter(s.wl_surface(), seat, data, event)
            }
        }
    }

    fn motion(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        match &self.window {
            InnerManagedWindow::Window(w) => w.motion(seat, data, event),
            InnerManagedWindow::Surface(s) => s.wl_surface().motion(seat, data, event),
        }
    }

    fn relative_motion(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        event: &smithay::input::pointer::RelativeMotionEvent,
    ) {
        match &self.window {
            InnerManagedWindow::Window(w) => w.relative_motion(seat, data, event),
            InnerManagedWindow::Surface(s) => s.wl_surface().relative_motion(seat, data, event),
        }
    }

    fn button(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        event: &smithay::input::pointer::ButtonEvent,
    ) {
        match &self.window {
            InnerManagedWindow::Window(w) => w.button(seat, data, event),
            InnerManagedWindow::Surface(s) => s.wl_surface().button(seat, data, event),
        }
    }

    fn axis(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        frame: smithay::input::pointer::AxisFrame,
    ) {
        match &self.window {
            InnerManagedWindow::Window(w) => w.axis(seat, data, frame),
            InnerManagedWindow::Surface(s) => s.wl_surface().axis(seat, data, frame),
        }
    }

    fn leave(
        &self,
        seat: &smithay::input::Seat<SmithayState>,
        data: &mut SmithayState,
        serial: smithay::utils::Serial,
        time: u32,
    ) {
        match &self.window {
            InnerManagedWindow::Window(w) => PointerTarget::leave(w, seat, data, serial, time),
            InnerManagedWindow::Surface(s) => {
                PointerTarget::leave(s.wl_surface(), seat, data, serial, time)
            }
        }
    }
}

impl ManagedWindow {
    pub fn from_window(window: Window) -> Self {
        Self {
            window: InnerManagedWindow::Window(window),
            data: Arc::new(RwLock::new(ManagedWindowData::default())),
            handle: None,
        }
    }

    pub fn from_surface(surface: LayerSurface) -> Self {
        Self {
            window: InnerManagedWindow::Surface(surface),
            data: Arc::new(RwLock::new(ManagedWindowData::default())),
            handle: None,
        }
    }

    pub fn is_wlr_surface(&self) -> bool {
        match self.window {
            InnerManagedWindow::Window(_) => false,
            InnerManagedWindow::Surface(_) => true,
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

        match &self.window {
            InnerManagedWindow::Window(w) => {
                elements.append(&mut w.render_elements(renderer, location, scale, alpha))
            }
            InnerManagedWindow::Surface(_) => (),
        }

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

    pub fn send_close(&self) {
        match &self.window {
            InnerManagedWindow::Window(w) => w.toplevel().send_close(),
            InnerManagedWindow::Surface(s) => s.layer_surface().send_close(),
        }
    }

    pub fn toplevel(&self) -> Option<&ToplevelSurface> {
        match &self.window {
            InnerManagedWindow::Window(w) => Some(w.toplevel()),
            InnerManagedWindow::Surface(_) => None,
        }
    }

    pub fn on_commit(&self) {
        match &self.window {
            InnerManagedWindow::Window(w) => w.on_commit(),
            InnerManagedWindow::Surface(_) => (),
        }
    }

    pub fn get_window(&self) -> Option<&Window> {
        match &self.window {
            InnerManagedWindow::Window(w) => Some(w),
            InnerManagedWindow::Surface(_) => None,
        }
    }

    pub fn set_geometry(&mut self, geometry: Rectangle<i32, Logical>) {
        self.data.write().unwrap().geometry = Some(geometry);
        match &self.window {
            InnerManagedWindow::Window(w) => {
                w.toplevel()
                    .with_pending_state(|state| state.size = Some(geometry.size));
                w.toplevel().send_configure();
            }
            InnerManagedWindow::Surface(_) => (),
        }
    }

    pub fn get_geometry(&self) -> Option<Rectangle<i32, Logical>> {
        match &self.window {
            InnerManagedWindow::Window(w) => Some(w.geometry()),
            InnerManagedWindow::Surface(_) => None,
        }
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
        match &self.window {
            InnerManagedWindow::Window(w) => {
                w.send_frame(output, time, throttle, primary_scan_out_output)
            }
            InnerManagedWindow::Surface(_) => (),
        }
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
        match &self.window {
            InnerManagedWindow::Window(w) => {
                w.send_dmabuf_feedback(output, primary_scan_out_output, select_dmabuf_feedback)
            }
            InnerManagedWindow::Surface(_) => (),
        }
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
        match &self.window {
            InnerManagedWindow::Window(w) => w.take_presentation_feedback(
                output_feedback,
                primary_scan_out_output,
                presentation_feedback_flags,
            ),
            InnerManagedWindow::Surface(_) => (),
        }
    }
}

use std::{borrow::BorrowMut, cell::RefCell, collections::HashMap};

use smithay::{
    backend::renderer::{
        gles::{
            element::PixelShaderElement, GlesPixelProgram, GlesRenderer, Uniform, UniformName,
            UniformType,
        },
        glow::GlowRenderer,
    },
    desktop::Window,
    utils::{IsAlive, Logical, Point, Rectangle},
};

use crate::{
    leftwm_config::BorderConfig, managed_window::ManagedWindow, window_registry::WindowHandle,
};

pub enum WindowState {
    Focused,
    Floating,
    Default,
}

#[derive(Clone, Copy, Debug)]
pub struct NormalisedColor(f32, f32, f32);

impl From<[u8; 3]> for NormalisedColor {
    fn from(value: [u8; 3]) -> Self {
        NormalisedColor(
            (value[0] as f32) / 255f32,
            (value[1] as f32) / 255f32,
            (value[2] as f32) / 255f32,
        )
    }
}

impl Into<(f32, f32, f32)> for NormalisedColor {
    fn into(self) -> (f32, f32, f32) {
        (self.0, self.1, self.2)
    }
}

const BORDER_SHADER: &str = include_str!("../../resources/border.frag");

pub struct BorderRenderer {
    shader: GlesPixelProgram,
}

struct BorderRendererElements(RefCell<HashMap<WindowHandle, (PixelShaderElement, Window)>>);

impl BorderRenderer {
    pub fn new(renderer: &mut GlowRenderer) {
        let renderer: &mut GlesRenderer = renderer.borrow_mut();
        let border_program = renderer
            .compile_custom_pixel_shader(
                BORDER_SHADER,
                &[
                    UniformName::new("color", UniformType::_3f),
                    UniformName::new("thickness", UniformType::_1f),
                    UniformName::new("halfThickness", UniformType::_1f),
                ],
            )
            .unwrap();
        renderer
            .egl_context()
            .user_data()
            .insert_if_missing(|| BorderRenderer {
                shader: border_program,
            });
        renderer
            .egl_context()
            .user_data()
            .insert_if_missing(|| BorderRendererElements(RefCell::new(HashMap::new())));
    }

    fn get(renderer: &GlowRenderer) -> &BorderRenderer {
        renderer
            .egl_context()
            .user_data()
            .get::<BorderRenderer>()
            .expect("This renderer does not yet have a border renderer")
    }

    /// Caller must pass the right color (default, floating, focused)
    pub fn render_element(
        renderer: &mut GlowRenderer,
        window: &ManagedWindow,
        borders: &BorderConfig,
        window_state: WindowState,
        loc: Point<i32, Logical>,
    ) -> PixelShaderElement {
        let border_renderer = Self::get(renderer);
        let border_width = borders.border_width;
        let geometry = Rectangle::from_loc_and_size(
            loc - Point::from((border_width, border_width)), // offset by a border width
            // Unwrap should be safe as anything that is not a window does not get a border.
            window.get_geometry().unwrap().size + (border_width * 2, border_width * 2).into(), // size the box to include the border
        );

        let elements = &mut renderer
            .egl_context()
            .user_data()
            .get::<BorderRendererElements>()
            .expect("This renderer does not yet have a border renderer")
            .0
            .borrow_mut();

        let Some(window_handle) = window.get_handle() else {
            panic!("This window does not have a handle, it should not be rendered");
            //TODO: get a better solution
        };

        let color = match window_state {
            WindowState::Focused => borders.focused_border_color,
            WindowState::Floating => borders.floating_border_color,
            WindowState::Default => borders.default_border_color,
        };

        let element = PixelShaderElement::new(
            border_renderer.shader.clone(),
            geometry,
            None,
            1.0,
            vec![
                Uniform::new("color", Into::<(f32, f32, f32)>::into(color)),
                Uniform::new("thickness", border_width as f32),
                Uniform::new("halfThickness", (border_width as f32) / 2f32),
            ],
        );
        elements.insert(
            window_handle.clone(),
            // Unwrap should be safe as anything that is not a window does not get a border.
            (element.clone(), window.get_window().unwrap().clone()),
        );
        element
    }

    pub fn cleanup(renderer: &mut GlowRenderer) {
        let elements = &mut renderer
            .egl_context()
            .user_data()
            .get::<BorderRendererElements>()
            .expect("This renderer does not yet have a border renderer")
            .0
            .borrow_mut();
        elements.retain(|_, w| w.1.alive());
    }
}

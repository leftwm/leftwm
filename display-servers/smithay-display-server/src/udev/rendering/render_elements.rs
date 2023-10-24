use smithay::{
    backend::renderer::{
        element::{surface::WaylandSurfaceRenderElement, Element, Id, RenderElement},
        gles::element::PixelShaderElement,
        glow::GlowRenderer,
        multigpu::Error as MultiError,
        utils::CommitCounter,
        ImportAll, ImportMem, Renderer,
    },
    utils::{Buffer, Physical, Rectangle, Scale},
};

use crate::drawing::PointerRenderElement;

use super::{UdevFrame, UdevRenderer};

pub enum CustomRenderElements<R>
where
    R: Renderer,
{
    Pointer(PointerRenderElement<R>),
    Surface(WaylandSurfaceRenderElement<R>),
    Shader(PixelShaderElement),
}

impl<R> Element for CustomRenderElements<R>
where
    R: Renderer,
    <R as Renderer>::TextureId: 'static,
    R: ImportAll + ImportMem,
{
    fn id(&self) -> &Id {
        match self {
            CustomRenderElements::Pointer(elem) => elem.id(),
            CustomRenderElements::Surface(elem) => elem.id(),
            CustomRenderElements::Shader(elem) => elem.id(),
        }
    }

    fn current_commit(&self) -> CommitCounter {
        match self {
            CustomRenderElements::Pointer(elem) => elem.current_commit(),
            CustomRenderElements::Surface(elem) => elem.current_commit(),
            CustomRenderElements::Shader(elem) => elem.current_commit(),
        }
    }

    fn src(&self) -> Rectangle<f64, Buffer> {
        match self {
            CustomRenderElements::Pointer(elem) => elem.src(),
            CustomRenderElements::Surface(elem) => elem.src(),
            CustomRenderElements::Shader(elem) => elem.src(),
        }
    }

    fn geometry(&self, scale: Scale<f64>) -> Rectangle<i32, Physical> {
        match self {
            CustomRenderElements::Pointer(elem) => elem.geometry(scale),
            CustomRenderElements::Surface(elem) => elem.geometry(scale),
            CustomRenderElements::Shader(elem) => elem.geometry(scale),
        }
    }

    fn location(&self, scale: Scale<f64>) -> smithay::utils::Point<i32, Physical> {
        match self {
            CustomRenderElements::Pointer(elem) => elem.location(scale),
            CustomRenderElements::Surface(elem) => elem.location(scale),
            CustomRenderElements::Shader(elem) => elem.location(scale),
        }
    }

    fn transform(&self) -> smithay::utils::Transform {
        match self {
            CustomRenderElements::Pointer(elem) => elem.transform(),
            CustomRenderElements::Surface(elem) => elem.transform(),
            CustomRenderElements::Shader(elem) => elem.transform(),
        }
    }

    fn damage_since(
        &self,
        scale: Scale<f64>,
        commit: Option<CommitCounter>,
    ) -> Vec<Rectangle<i32, Physical>> {
        match self {
            CustomRenderElements::Pointer(elem) => elem.damage_since(scale, commit),
            CustomRenderElements::Surface(elem) => elem.damage_since(scale, commit),
            CustomRenderElements::Shader(elem) => elem.damage_since(scale, commit),
        }
    }

    fn opaque_regions(&self, scale: Scale<f64>) -> Vec<Rectangle<i32, Physical>> {
        match self {
            CustomRenderElements::Pointer(elem) => elem.opaque_regions(scale),
            CustomRenderElements::Surface(elem) => elem.opaque_regions(scale),
            CustomRenderElements::Shader(elem) => elem.opaque_regions(scale),
        }
    }
}

impl<'a, 'b> RenderElement<UdevRenderer<'a, 'b>> for CustomRenderElements<UdevRenderer<'a, 'b>> {
    fn draw<'frame>(
        &self,
        frame: &mut UdevFrame<'a, 'b, 'frame>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
    ) -> Result<(), <UdevRenderer<'a, 'b> as Renderer>::Error> {
        match self {
            CustomRenderElements::Pointer(elem) => {
                RenderElement::<UdevRenderer>::draw(elem, frame, src, dst, damage)
            }
            CustomRenderElements::Surface(elem) => elem.draw(frame, src, dst, damage),
            CustomRenderElements::Shader(elem) => {
                RenderElement::<GlowRenderer>::draw(elem, frame.as_mut(), src, dst, damage)
                    .map_err(MultiError::Render)
            }
        }
    }

    fn underlying_storage(
        &self,
        renderer: &mut UdevRenderer<'a, 'b>,
    ) -> Option<smithay::backend::renderer::element::UnderlyingStorage> {
        match self {
            CustomRenderElements::Pointer(elem) => elem.underlying_storage(renderer),
            CustomRenderElements::Surface(elem) => elem.underlying_storage(renderer),
            CustomRenderElements::Shader(elem) => elem.underlying_storage(renderer.as_mut()),
        }
    }
}

impl RenderElement<GlowRenderer> for CustomRenderElements<GlowRenderer> {
    fn draw(
        &self,
        frame: &mut <GlowRenderer as Renderer>::Frame<'_>,
        src: Rectangle<f64, Buffer>,
        dst: Rectangle<i32, Physical>,
        damage: &[Rectangle<i32, Physical>],
    ) -> Result<(), <GlowRenderer as Renderer>::Error> {
        match self {
            CustomRenderElements::Pointer(elem) => {
                RenderElement::<GlowRenderer>::draw(elem, frame, src, dst, damage)
            }
            CustomRenderElements::Surface(elem) => elem.draw(frame, src, dst, damage),
            CustomRenderElements::Shader(elem) => {
                RenderElement::<GlowRenderer>::draw(elem, frame, src, dst, damage)
            }
        }
    }
}
impl<R> From<PointerRenderElement<R>> for CustomRenderElements<R>
where
    R: Renderer,
{
    fn from(value: PointerRenderElement<R>) -> Self {
        CustomRenderElements::Pointer(value)
    }
}

impl<R> From<WaylandSurfaceRenderElement<R>> for CustomRenderElements<R>
where
    R: Renderer,
{
    fn from(value: WaylandSurfaceRenderElement<R>) -> Self {
        CustomRenderElements::Surface(value)
    }
}

impl<R> From<PixelShaderElement> for CustomRenderElements<R>
where
    R: Renderer,
{
    fn from(value: PixelShaderElement) -> Self {
        CustomRenderElements::Shader(value)
    }
}

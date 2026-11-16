//! Rendering PDF pages with Poppler.

use gtk::prelude::Cast;
use gtk::{cairo, gdk, glib};

/// Width in pixels the first page is rendered at; the picture widget scales
/// it down to the pane width (zoom and further pages come with spec 0016).
const RENDER_WIDTH: f64 = 1600.0;

/// Renders the first page of a PDF, or `None` if it cannot be rendered.
pub fn render_first_page(pdf: &[u8]) -> Option<gdk::Texture> {
    let document = poppler::Document::from_bytes(&glib::Bytes::from(pdf), None).ok()?;
    let page = document.page(0)?;
    let (page_width, page_height) = page.size();
    if page_width <= 0.0 || page_height <= 0.0 {
        return None;
    }
    let scale = RENDER_WIDTH / page_width;
    let width = (page_width * scale).round() as i32;
    let height = (page_height * scale).round() as i32;

    let mut surface = cairo::ImageSurface::create(cairo::Format::ARgb32, width, height).ok()?;
    {
        let context = cairo::Context::new(&surface).ok()?;
        context.set_source_rgb(1.0, 1.0, 1.0);
        context.paint().ok()?;
        context.scale(scale, scale);
        page.render(&context);
    }
    surface.flush();

    let stride = usize::try_from(surface.stride()).ok()?;
    let data = surface.data().ok()?.to_vec();
    // Cairo's ARGB32 is native-endian; on little-endian machines the bytes
    // are in B, G, R, A order with premultiplied alpha.
    let format = if cfg!(target_endian = "little") {
        gdk::MemoryFormat::B8g8r8a8Premultiplied
    } else {
        gdk::MemoryFormat::A8r8g8b8Premultiplied
    };
    Some(
        gdk::MemoryTexture::new(
            width,
            height,
            format,
            &glib::Bytes::from_owned(data),
            stride,
        )
        .upcast(),
    )
}

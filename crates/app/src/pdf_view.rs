//! Rendering PDF pages with Poppler.

use gtk::prelude::*;
use gtk::{cairo, gdk, glib, graphene};

/// A parsed PDF whose pages can be rendered.
pub struct PdfDocument {
    document: poppler::Document,
}

impl PdfDocument {
    /// Parses a PDF; `None` if it cannot be read or has no page.
    pub fn new(pdf: &[u8]) -> Option<Self> {
        let document = poppler::Document::from_bytes(&glib::Bytes::from(pdf), None).ok()?;
        (document.n_pages() > 0).then_some(Self { document })
    }

    /// Page sizes in points; pages without a usable size get A4.
    pub fn page_sizes(&self) -> Vec<(f64, f64)> {
        (0..self.document.n_pages())
            .map(|index| {
                self.document
                    .page(index)
                    .map(|page| page.size())
                    .filter(|(width, height)| *width > 0.0 && *height > 0.0)
                    .unwrap_or((595.0, 842.0))
            })
            .collect()
    }

    /// Renders a horizontal strip of a page: `top` and `height` in points,
    /// at `scale` device pixels per point. Returns a paintable of
    /// `shown_width` × `shown_height` logical pixels, so that the texture's
    /// extra device pixels make it sharp on scaled displays.
    pub fn render_strip(
        &self,
        index: usize,
        (top, height): (f64, f64),
        scale: f64,
        (shown_width, shown_height): (f32, f32),
    ) -> Option<gdk::Paintable> {
        let page = self.document.page(i32::try_from(index).ok()?)?;
        let (page_width, _) = page.size();
        // Truncation to whole pixels is intended.
        #[allow(clippy::cast_possible_truncation)]
        let (width, strip_height) = (
            (page_width * scale).round().max(1.0) as i32,
            (height * scale).round().max(1.0) as i32,
        );
        let mut surface =
            cairo::ImageSurface::create(cairo::Format::ARgb32, width, strip_height).ok()?;
        {
            let context = cairo::Context::new(&surface).ok()?;
            context.set_source_rgb(1.0, 1.0, 1.0);
            context.paint().ok()?;
            context.scale(scale, scale);
            context.translate(0.0, -top);
            page.render(&context);
        }
        surface.flush();
        let texture = texture(&mut surface, width, strip_height)?;

        let snapshot = gtk::Snapshot::new();
        snapshot.append_texture(
            &texture,
            &graphene::Rect::new(0.0, 0.0, shown_width, shown_height),
        );
        snapshot.to_paintable(Some(&graphene::Size::new(shown_width, shown_height)))
    }
}

fn texture(surface: &mut cairo::ImageSurface, width: i32, height: i32) -> Option<gdk::Texture> {
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

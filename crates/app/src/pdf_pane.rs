//! The PDF pane: all pages one below the other, a page indicator and zoom
//! (spec 0016).
//!
//! Each page is split into strips ("tiles") of a fixed height in points,
//! created once per document. Zooming only resizes them: a tile keeps its
//! old picture, stretched, until it is rendered again at the display's
//! device resolution. Only tiles near the visible area are rendered; tiles
//! far away are released, so text stays sharp at 400 % without holding
//! whole high-resolution pages in memory.

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::time::Duration;

use adw::prelude::*;
use gtk::glib;

use crate::pdf_layout::{PAGE_GAP, PageStack, ZOOM_STEPS, Zoom, render_width, zoom_in, zoom_out};
use crate::pdf_view::PdfDocument;
use crate::texts;

/// Logical pixels per point at 100 % (96 dpi).
const PIXELS_PER_POINT: f64 = 96.0 / 72.0;
/// Height of one tile in points: 512 logical pixels at 400 %, which bounds
/// the size of a rendered tile.
const TILE_POINTS: f64 = 96.0;
/// Rendering waits this long after scrolling or zooming has stopped.
const RENDER_DELAY: Duration = Duration::from_millis(60);
/// CSS class of a page.
pub const PAGE_CLASS: &str = "pdf-page";

#[derive(Clone)]
pub struct PdfPane {
    inner: Rc<Inner>,
}

struct Inner {
    root: gtk::Box,
    scroller: gtk::ScrolledWindow,
    pages_box: gtk::Box,
    page_label: gtk::Label,
    zoom_label: gtk::Label,
    zoom_out_button: gtk::Button,
    zoom_in_button: gtk::Button,
    /// The shown PDF, `None` while the pane is empty.
    loaded: RefCell<Option<Loaded>>,
    /// Counts render requests; only the latest one runs.
    render_request: Cell<u64>,
}

struct Loaded {
    document: PdfDocument,
    pages: Vec<Page>,
    zoom: Zoom,
    layout: Layout,
}

/// Where the pages are at the current zoom and pane width.
struct Layout {
    /// Logical pixels per point.
    scale: f64,
    stack: PageStack,
}

impl Layout {
    /// The effective zoom in percent, also when fitting the width.
    fn percent(&self) -> f64 {
        self.scale / PIXELS_PER_POINT * 100.0
    }
}

struct Page {
    /// Width and height in points.
    size: (f64, f64),
    tiles: Vec<Tile>,
}

struct Tile {
    picture: gtk::Picture,
    /// Top and height in points.
    strip: (f64, f64),
    /// Device pixels per point the shown picture was rendered at, if any.
    rendered: Option<f64>,
}

impl PdfPane {
    pub fn new() -> Self {
        let pages_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            // Gaps as in `PageStack`.
            .spacing(PAGE_GAP as i32)
            .margin_top(PAGE_GAP as i32)
            .margin_bottom(PAGE_GAP as i32)
            .margin_start(PAGE_GAP as i32)
            .margin_end(PAGE_GAP as i32)
            .halign(gtk::Align::Center)
            .focusable(true)
            .build();
        let scroller = gtk::ScrolledWindow::builder()
            .child(&pages_box)
            .vexpand(true)
            .build();
        let page_label = gtk::Label::builder()
            .xalign(0.0)
            .hexpand(true)
            .css_classes(["dim-label", "numeric"])
            .build();
        let zoom_label = gtk::Label::builder()
            .css_classes(["dim-label", "numeric"])
            .build();
        let bar = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(12)
            .margin_start(12)
            .margin_end(12)
            .margin_top(4)
            .margin_bottom(4)
            .build();
        let zoom_button = |icon: &str, tooltip: &str| {
            let button = gtk::Button::builder()
                .icon_name(icon)
                .tooltip_text(tooltip)
                .css_classes(["flat", "circular"])
                .sensitive(false)
                .build();
            button.update_property(&[gtk::accessible::Property::Label(tooltip)]);
            button
        };
        let zoom_out_button = zoom_button("zoom-out-symbolic", texts::ZOOM_OUT);
        let zoom_in_button = zoom_button("zoom-in-symbolic", texts::ZOOM_IN);
        let zoom_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        zoom_box.append(&zoom_out_button);
        zoom_box.append(&zoom_label);
        zoom_box.append(&zoom_in_button);
        bar.append(&page_label);
        bar.append(&zoom_box);
        let root = gtk::Box::new(gtk::Orientation::Vertical, 0);
        root.append(&scroller);
        root.append(&gtk::Separator::new(gtk::Orientation::Horizontal));
        root.append(&bar);

        let this = Self {
            inner: Rc::new(Inner {
                root,
                scroller,
                pages_box,
                page_label,
                zoom_label,
                zoom_out_button,
                zoom_in_button,
                loaded: RefCell::default(),
                render_request: Cell::new(0),
            }),
        };
        this.connect_signals();
        this
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.inner.root
    }

    /// Shows a PDF fitted to the pane width; `false` if it cannot be shown.
    pub fn load(&self, pdf: &[u8]) -> bool {
        self.clear();
        let Some(document) = PdfDocument::new(pdf) else {
            return false;
        };
        let pages: Vec<Page> = document
            .page_sizes()
            .into_iter()
            .map(|size| self.add_page(size))
            .collect();
        let layout = layout(Zoom::FitWidth, self.pane_width(), &pages);
        *self.inner.loaded.borrow_mut() = Some(Loaded {
            document,
            pages,
            zoom: Zoom::FitWidth,
            layout,
        });
        self.apply_layout(None);
        true
    }

    pub fn clear(&self) {
        while let Some(child) = self.inner.pages_box.first_child() {
            self.inner.pages_box.remove(&child);
        }
        *self.inner.loaded.borrow_mut() = None;
        self.inner.page_label.set_text("");
        self.inner.zoom_label.set_text("");
        self.inner.zoom_out_button.set_sensitive(false);
        self.inner.zoom_in_button.set_sensitive(false);
    }

    /// Adds the widgets of a page of `size` points: its tiles, sized later.
    fn add_page(&self, (width, height): (f64, f64)) -> Page {
        let page_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .css_classes([PAGE_CLASS])
            .build();
        let mut tiles = Vec::new();
        let mut top = 0.0;
        while top < height {
            let strip = (top, TILE_POINTS.min(height - top));
            let picture = gtk::Picture::builder()
                .can_shrink(true)
                .content_fit(gtk::ContentFit::Fill)
                .build();
            page_box.append(&picture);
            tiles.push(Tile {
                picture,
                strip,
                rendered: None,
            });
            top += strip.1;
        }
        self.inner.pages_box.append(&page_box);
        Page {
            size: (width, height),
            tiles,
        }
    }

    fn connect_signals(&self) {
        let vadjustment = self.inner.scroller.vadjustment();
        let this = self.clone();
        vadjustment.connect_value_changed(move |_| {
            this.update_indicator();
            this.schedule_render();
        });
        // The pane height changed: other pages may have become visible.
        let this = self.clone();
        vadjustment.connect_changed(move |_| {
            this.update_indicator();
            this.schedule_render();
        });
        // The pane width changed: re-fit while fit-width is active.
        let this = self.clone();
        self.inner
            .scroller
            .hadjustment()
            .connect_changed(move |_| this.relayout());

        // Clicking the pages gives them the focus for the zoom keys.
        let click = gtk::GestureClick::new();
        let pages = self.inner.pages_box.clone();
        click.connect_pressed(move |_, _, _, _| {
            pages.grab_focus();
        });
        self.inner.scroller.add_controller(click);

        let keys = gtk::ShortcutController::new();
        for (trigger, zoom) in [
            ("<Control>plus|<Control>equal|<Control>KP_Add", 1),
            ("<Control>minus|<Control>KP_Subtract", -1),
            ("<Control>0|<Control>KP_0", 0),
        ] {
            let this = self.clone();
            keys.add_shortcut(gtk::Shortcut::new(
                gtk::ShortcutTrigger::parse_string(trigger),
                Some(gtk::CallbackAction::new(move |_, _| {
                    this.zoom(zoom);
                    glib::Propagation::Stop
                })),
            ));
        }
        self.inner.root.add_controller(keys);

        for (button, zoom) in [
            (&self.inner.zoom_in_button, 1),
            (&self.inner.zoom_out_button, -1),
        ] {
            let this = self.clone();
            button.connect_clicked(move |_| this.zoom(zoom));
        }
    }

    /// The width available to the pages.
    fn pane_width(&self) -> f64 {
        let width = self.inner.scroller.hadjustment().page_size();
        if width > 0.0 {
            width
        } else {
            f64::from(self.inner.scroller.width().max(600))
        }
    }

    /// Zooms in (1), out (-1) or back to fit width (0).
    fn zoom(&self, direction: i32) {
        {
            let mut loaded = self.inner.loaded.borrow_mut();
            let Some(loaded) = loaded.as_mut() else {
                return;
            };
            let percent = loaded.layout.percent();
            loaded.zoom = match direction {
                0 => Zoom::FitWidth,
                d if d > 0 => Zoom::Percent(zoom_in(percent)),
                _ => Zoom::Percent(zoom_out(percent)),
            };
        }
        self.relayout();
    }

    /// Lays the pages out again for the current zoom and pane width, keeping
    /// the visible place in view; nothing happens if the scale stays.
    fn relayout(&self) {
        let width = self.pane_width();
        let offset = self.inner.scroller.vadjustment().value();
        let anchor = {
            let mut loaded = self.inner.loaded.borrow_mut();
            let Some(loaded) = loaded.as_mut() else {
                return;
            };
            let new = layout(loaded.zoom, width, &loaded.pages);
            if (new.scale - loaded.layout.scale).abs() < 1e-9 {
                return;
            }
            let anchor = loaded.layout.stack.anchor(offset);
            loaded.layout = new;
            anchor
        };
        self.apply_layout(Some(anchor));
    }

    /// Sizes the tiles for the current layout and scrolls to `anchor`.
    /// Pictures stay until they are rendered again.
    fn apply_layout(&self, anchor: Option<(usize, f64)>) {
        let offset = {
            let loaded = self.inner.loaded.borrow();
            let Some(Loaded { pages, layout, .. }) = loaded.as_ref() else {
                return;
            };
            let scale = layout.scale;
            for page in pages {
                // Truncation to whole logical pixels is intended.
                #[allow(clippy::cast_possible_truncation)]
                let width = (page.size.0 * scale).round() as i32;
                for tile in &page.tiles {
                    let (top, height) = tile.strip;
                    #[allow(clippy::cast_possible_truncation)]
                    let shown =
                        ((top + height) * scale).round() as i32 - (top * scale).round() as i32;
                    tile.picture.set_size_request(width, shown);
                }
            }
            anchor.map_or(0.0, |anchor| layout.stack.offset_of(anchor))
        };
        // The scrolled window learns the new height only when it is
        // allocated; tell the adjustment now, so that the place is kept.
        let adjustment = self.inner.scroller.vadjustment();
        if let Some(loaded) = self.inner.loaded.borrow().as_ref() {
            adjustment.set_upper(loaded.layout.stack.height());
        }
        adjustment.set_value(offset);
        self.update_indicator();
        self.schedule_render();
    }

    fn update_indicator(&self) {
        let loaded = self.inner.loaded.borrow();
        let Some(Loaded { pages, layout, .. }) = loaded.as_ref() else {
            return;
        };
        let adjustment = self.inner.scroller.vadjustment();
        let page = layout
            .stack
            .visible_page(adjustment.value(), adjustment.page_size());
        self.inner
            .page_label
            .set_text(&texts::page_indicator_text(page, pages.len()));
        let percent = layout.percent();
        self.inner.zoom_label.set_text(&texts::zoom_text(percent));
        // At the smallest or largest step the button has no effect.
        self.inner
            .zoom_out_button
            .set_sensitive(percent > f64::from(ZOOM_STEPS[0]) + 0.5);
        self.inner
            .zoom_in_button
            .set_sensitive(percent < f64::from(ZOOM_STEPS[ZOOM_STEPS.len() - 1]) - 0.5);
    }

    /// Renders the tiles near the visible area once scrolling has stopped.
    fn schedule_render(&self) {
        let request = self.inner.render_request.get() + 1;
        self.inner.render_request.set(request);
        let this = self.clone();
        glib::timeout_add_local_once(RENDER_DELAY, move || {
            if this.inner.render_request.get() == request {
                this.render_visible();
            }
        });
    }

    /// Renders tiles within one screen above and below the visible area that
    /// are not sharp at the current scale, and releases tiles beyond three
    /// screens.
    fn render_visible(&self) {
        let adjustment = self.inner.scroller.vadjustment();
        let (top, height) = (adjustment.value(), adjustment.page_size().max(1.0));
        let scale_factor = self.inner.root.scale_factor();
        let mut loaded = self.inner.loaded.borrow_mut();
        let Some(Loaded {
            document,
            pages,
            layout,
            ..
        }) = loaded.as_mut()
        else {
            return;
        };
        let (want_top, want_bottom) = (top - height, top + 2.0 * height);
        let (keep_top, keep_bottom) = (top - 3.0 * height, top + 4.0 * height);
        for (index, page) in pages.iter_mut().enumerate() {
            let page_top = layout.stack.offset_of((index, 0.0));
            // Device pixels per point, so that text stays sharp at any zoom.
            let width_pt = page.size.0;
            let render_scale =
                f64::from(render_width(width_pt * layout.scale, scale_factor)) / width_pt;
            for tile in &mut page.tiles {
                let tile_top = page_top + tile.strip.0 * layout.scale;
                let tile_bottom = tile_top + tile.strip.1 * layout.scale;
                if tile_bottom > want_top && tile_top < want_bottom {
                    if tile.rendered != Some(render_scale) {
                        // Truncation to f32 is fine for logical sizes.
                        #[allow(clippy::cast_possible_truncation)]
                        let shown = (
                            tile.picture.width_request() as f32,
                            tile.picture.height_request() as f32,
                        );
                        let paintable =
                            document.render_strip(index, tile.strip, render_scale, shown);
                        tile.picture.set_paintable(paintable.as_ref());
                        tile.rendered = Some(render_scale);
                    }
                } else if tile.rendered.is_some()
                    && (tile_bottom < keep_top || tile_top > keep_bottom)
                {
                    tile.picture.set_paintable(None::<&gtk::gdk::Paintable>);
                    tile.rendered = None;
                }
            }
        }
    }
}

/// The layout of `pages` for `zoom` in a pane of `width` logical pixels.
fn layout(zoom: Zoom, width: f64, pages: &[Page]) -> Layout {
    let widest = pages.iter().map(|page| page.size.0).fold(1.0, f64::max);
    let scale = match zoom {
        Zoom::FitWidth => ((width - 2.0 * PAGE_GAP) / widest).max(0.05),
        Zoom::Percent(percent) => f64::from(percent) / 100.0 * PIXELS_PER_POINT,
    };
    let heights: Vec<f64> = pages
        .iter()
        .map(|page| (page.size.1 * scale).round())
        .collect();
    Layout {
        scale,
        stack: PageStack::new(&heights),
    }
}

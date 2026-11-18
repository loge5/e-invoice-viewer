//! The PDF pane: all pages one below the other, a page indicator and zoom
//! (spec 0016).
//!
//! Pages are split into strips ("tiles") that are rendered only while they
//! are near the visible area, at the display's device resolution. This keeps
//! text sharp at 400 % without holding whole high-resolution pages in memory.

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
/// Height of one tile in logical pixels.
const TILE_HEIGHT: f64 = 512.0;
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
    state: RefCell<State>,
    /// Counts render requests; only the latest one runs.
    render_request: Cell<u64>,
}

#[derive(Default)]
struct State {
    document: Option<PdfDocument>,
    /// Page sizes in points.
    sizes: Vec<(f64, f64)>,
    pages: Vec<Page>,
    zoom: Option<Zoom>,
    /// Logical pixels per point of the current layout.
    scale: f64,
    /// Pane width the fit-width layout was made for.
    laid_out_width: f64,
    stack: Option<PageStack>,
    /// Where to scroll once the new layout is allocated.
    pending_anchor: Option<(usize, f64)>,
}

struct Page {
    tiles: Vec<Tile>,
}

struct Tile {
    picture: gtk::Picture,
    /// Top and height in points.
    strip: (f64, f64),
    /// Device scale the tile is rendered at, if it is.
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
                state: RefCell::default(),
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
        {
            let mut state = self.inner.state.borrow_mut();
            state.sizes = document.page_sizes();
            state.document = Some(document);
            state.zoom = Some(Zoom::FitWidth);
        }
        self.inner.scroller.vadjustment().set_value(0.0);
        self.relayout(false);
        true
    }

    pub fn clear(&self) {
        while let Some(child) = self.inner.pages_box.first_child() {
            self.inner.pages_box.remove(&child);
        }
        *self.inner.state.borrow_mut() = State::default();
        self.inner.page_label.set_text("");
        self.inner.zoom_label.set_text("");
        self.inner.zoom_out_button.set_sensitive(false);
        self.inner.zoom_in_button.set_sensitive(false);
    }

    fn connect_signals(&self) {
        let vadjustment = self.inner.scroller.vadjustment();
        let this = self.clone();
        vadjustment.connect_value_changed(move |_| {
            this.update_indicator();
            this.schedule_render();
        });
        let this = self.clone();
        vadjustment.connect_changed(move |adjustment| {
            // The new layout is allocated: scroll to the remembered place.
            let anchor = this.inner.state.borrow_mut().pending_anchor.take();
            if let Some(anchor) = anchor {
                let offset = this
                    .inner
                    .state
                    .borrow()
                    .stack
                    .as_ref()
                    .map_or(0.0, |stack| stack.offset_of(anchor));
                adjustment.set_value(offset);
            }
            this.update_indicator();
            this.schedule_render();
        });
        // The pane width changed: re-fit while fit-width is active.
        let this = self.clone();
        self.inner
            .scroller
            .hadjustment()
            .connect_changed(move |adjustment| {
                let refit = {
                    let state = this.inner.state.borrow();
                    state.zoom == Some(Zoom::FitWidth)
                        && (adjustment.page_size() - state.laid_out_width).abs() > 1.0
                };
                if refit {
                    this.relayout(true);
                }
            });

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

    /// Zooms in (1), out (-1) or back to fit width (0).
    fn zoom(&self, direction: i32) {
        let new_zoom = {
            let state = self.inner.state.borrow();
            if state.document.is_none() {
                return;
            }
            let percent = state.scale / PIXELS_PER_POINT * 100.0;
            match direction {
                0 => Zoom::FitWidth,
                d if d > 0 => Zoom::Percent(zoom_in(percent)),
                _ => Zoom::Percent(zoom_out(percent)),
            }
        };
        self.inner.state.borrow_mut().zoom = Some(new_zoom);
        self.relayout(true);
    }

    /// Lays the pages out for the current zoom and pane width; with
    /// `keep_place`, the visible place stays in view.
    fn relayout(&self, keep_place: bool) {
        let width = self.inner.scroller.hadjustment().page_size();
        let width = if width > 0.0 {
            width
        } else {
            f64::from(self.inner.scroller.width().max(600))
        };
        let offset = self.inner.scroller.vadjustment().value();
        let mut state = self.inner.state.borrow_mut();
        let Some(zoom) = state.zoom else {
            return;
        };
        let anchor = keep_place
            .then(|| state.stack.as_ref().map(|stack| stack.anchor(offset)))
            .flatten();
        let widest = state.sizes.iter().map(|(w, _)| *w).fold(1.0, f64::max);
        state.scale = match zoom {
            Zoom::FitWidth => ((width - 2.0 * PAGE_GAP) / widest).max(0.05),
            Zoom::Percent(percent) => f64::from(percent) / 100.0 * PIXELS_PER_POINT,
        };
        state.laid_out_width = width;
        let scale = state.scale;

        while let Some(child) = self.inner.pages_box.first_child() {
            self.inner.pages_box.remove(&child);
        }
        let mut pages = Vec::new();
        let mut heights = Vec::new();
        for (width_pt, height_pt) in state.sizes.clone() {
            let (page_width, page_height) = (width_pt * scale, height_pt * scale);
            let page_box = gtk::Box::builder()
                .orientation(gtk::Orientation::Vertical)
                .css_classes([PAGE_CLASS])
                .build();
            let tile_pt = TILE_HEIGHT / scale;
            let mut tiles = Vec::new();
            let mut top = 0.0;
            while top < height_pt {
                let strip = (top, tile_pt.min(height_pt - top));
                let picture = gtk::Picture::builder()
                    .can_shrink(true)
                    .content_fit(gtk::ContentFit::Fill)
                    .build();
                // Truncation to whole logical pixels is intended.
                #[allow(clippy::cast_possible_truncation)]
                picture.set_size_request(
                    page_width.round() as i32,
                    ((top + strip.1) * scale).round() as i32 - (top * scale).round() as i32,
                );
                page_box.append(&picture);
                tiles.push(Tile {
                    picture,
                    strip,
                    rendered: None,
                });
                top += strip.1;
            }
            self.inner.pages_box.append(&page_box);
            pages.push(Page { tiles });
            heights.push(page_height.round());
        }
        state.pages = pages;
        state.stack = Some(PageStack::new(&heights));
        state.pending_anchor = anchor;
        drop(state);
        self.update_indicator();
        self.schedule_render();
    }

    fn update_indicator(&self) {
        let state = self.inner.state.borrow();
        let Some(stack) = state.stack.as_ref() else {
            return;
        };
        let adjustment = self.inner.scroller.vadjustment();
        let page = stack.visible_page(adjustment.value(), adjustment.page_size());
        self.inner
            .page_label
            .set_text(&texts::page_indicator_text(page, state.sizes.len()));
        let percent = state.scale / PIXELS_PER_POINT * 100.0;
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

    fn render_visible(&self) {
        let adjustment = self.inner.scroller.vadjustment();
        let (top, height) = (adjustment.value(), adjustment.page_size().max(1.0));
        let scale_factor = self.inner.root.scale_factor();
        let mut state = self.inner.state.borrow_mut();
        let State {
            document: Some(document),
            sizes,
            pages,
            stack: Some(stack),
            scale,
            ..
        } = &mut *state
        else {
            return;
        };
        // Rendered ahead: one screen above and below; released beyond three.
        let (keep_top, keep_bottom) = (top - 3.0 * height, top + 4.0 * height);
        let (want_top, want_bottom) = (top - height, top + 2.0 * height);
        for index in stack.pages_between(keep_top, keep_bottom) {
            let page_top = stack.offset_of((index, 0.0));
            // Device pixels per point, so that text stays sharp at any zoom.
            let width_pt = sizes[index].0;
            let render_scale = f64::from(render_width(width_pt * *scale, scale_factor)) / width_pt;
            for tile in &mut pages[index].tiles {
                let tile_top = page_top + tile.strip.0 * *scale;
                let tile_bottom = tile_top + tile.strip.1 * *scale;
                let wanted = tile_bottom > want_top && tile_top < want_bottom;
                if wanted && tile.rendered != Some(render_scale) {
                    // Truncation to f32 is fine for logical sizes.
                    #[allow(clippy::cast_possible_truncation)]
                    let shown = (
                        tile.picture.width_request() as f32,
                        tile.picture.height_request() as f32,
                    );
                    let paintable = document.render_strip(index, tile.strip, render_scale, shown);
                    tile.picture.set_paintable(paintable.as_ref());
                    tile.rendered = Some(render_scale);
                }
            }
        }
        // Release tiles far away to bound memory.
        for (index, page) in pages.iter_mut().enumerate() {
            let page_top = stack.offset_of((index, 0.0));
            for tile in &mut page.tiles {
                let tile_top = page_top + tile.strip.0 * *scale;
                let tile_bottom = tile_top + tile.strip.1 * *scale;
                if tile.rendered.is_some() && (tile_bottom < keep_top || tile_top > keep_bottom) {
                    tile.picture.set_paintable(None::<&gtk::gdk::Paintable>);
                    tile.rendered = None;
                }
            }
        }
    }
}

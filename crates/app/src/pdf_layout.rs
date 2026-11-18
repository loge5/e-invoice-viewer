//! Page layout and zoom of the PDF pane (spec 0016), free of GTK so that it
//! can be unit-tested.

/// Zoom steps in percent, from the smallest to the largest.
pub const ZOOM_STEPS: [u32; 9] = [25, 50, 75, 100, 125, 150, 200, 300, 400];
/// Gap between pages and around them, in logical pixels.
pub const PAGE_GAP: f64 = 12.0;

/// The zoom of the PDF pane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Zoom {
    /// Pages scaled to the pane width.
    FitWidth,
    /// A fixed zoom step.
    Percent(u32),
}

/// The next larger zoom step after `current_percent` (the effective zoom,
/// also when fitting the width); 400 % stays 400 %.
pub fn zoom_in(current_percent: f64) -> u32 {
    ZOOM_STEPS
        .iter()
        .copied()
        .find(|step| f64::from(*step) > current_percent + 0.5)
        .unwrap_or(ZOOM_STEPS[ZOOM_STEPS.len() - 1])
}

/// The next smaller zoom step before `current_percent`; 25 % stays 25 %.
pub fn zoom_out(current_percent: f64) -> u32 {
    ZOOM_STEPS
        .iter()
        .rev()
        .copied()
        .find(|step| f64::from(*step) < current_percent - 0.5)
        .unwrap_or(ZOOM_STEPS[0])
}

/// Vertical positions of pages of the given heights (logical pixels),
/// stacked with `PAGE_GAP` above, between and below them.
#[derive(Debug, Clone, PartialEq)]
pub struct PageStack {
    tops: Vec<f64>,
    heights: Vec<f64>,
}

impl PageStack {
    pub fn new(heights: &[f64]) -> Self {
        let mut top = PAGE_GAP;
        let tops = heights
            .iter()
            .map(|height| {
                let this = top;
                top += height + PAGE_GAP;
                this
            })
            .collect();
        Self {
            tops,
            heights: heights.to_vec(),
        }
    }

    /// Total height including the gaps.
    pub fn height(&self) -> f64 {
        self.tops
            .last()
            .zip(self.heights.last())
            .map_or(PAGE_GAP, |(top, height)| top + height + PAGE_GAP)
    }

    /// The page (0-based) taking up most of the visible area from
    /// `viewport_top` with `viewport_height`.
    pub fn visible_page(&self, viewport_top: f64, viewport_height: f64) -> usize {
        let bottom = viewport_top + viewport_height;
        let mut best = (0, f64::MIN);
        for page in self.pages_between(viewport_top, bottom) {
            let top = self.tops[page];
            let visible = (top + self.heights[page]).min(bottom) - top.max(viewport_top);
            if visible > best.1 {
                best = (page, visible);
            }
        }
        best.0
    }

    /// The page at `offset` and how far into it `offset` is (0 = top edge,
    /// 1 = bottom edge), so that the same place can be found after zooming.
    pub fn anchor(&self, offset: f64) -> (usize, f64) {
        let page = self
            .tops
            .partition_point(|top| *top <= offset)
            .saturating_sub(1);
        match (self.tops.get(page), self.heights.get(page)) {
            (Some(top), Some(height)) if *height > 0.0 => {
                (page, ((offset - top) / height).clamp(0.0, 1.0))
            }
            _ => (page, 0.0),
        }
    }

    /// The offset of an anchor from `anchor`.
    pub fn offset_of(&self, (page, fraction): (usize, f64)) -> f64 {
        match (self.tops.get(page), self.heights.get(page)) {
            (Some(top), Some(height)) => top + fraction * height,
            _ => 0.0,
        }
    }

    /// Pages that overlap the range from `top` to `bottom`.
    pub fn pages_between(&self, top: f64, bottom: f64) -> std::ops::Range<usize> {
        let first = self
            .tops
            .iter()
            .zip(&self.heights)
            .position(|(page_top, height)| page_top + height > top)
            .unwrap_or(self.tops.len());
        let end = self.tops.partition_point(|page_top| *page_top < bottom);
        first..end.max(first)
    }
}

/// Width in device pixels a page is rendered at, so that it stays sharp on
/// screens with display scaling: the shown width times the scale factor.
pub fn render_width(shown_width: f64, scale_factor: i32) -> i32 {
    // Truncation to whole pixels is intended; widths are far below i32::MAX.
    #[allow(clippy::cast_possible_truncation)]
    let width = (shown_width * f64::from(scale_factor.max(1))).round() as i32;
    width.max(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    // AC4
    #[test]
    fn zoom_steps() {
        let mut zoom = 100.0;
        let mut seen = Vec::new();
        for _ in 0..7 {
            zoom = f64::from(zoom_in(zoom));
            seen.push(zoom);
        }
        assert_eq!(seen, [125.0, 150.0, 200.0, 300.0, 400.0, 400.0, 400.0]);
        let mut zoom = 100.0;
        let mut seen = Vec::new();
        for _ in 0..5 {
            zoom = f64::from(zoom_out(zoom));
            seen.push(zoom);
        }
        assert_eq!(seen, [75.0, 50.0, 25.0, 25.0, 25.0]);
        // From fit width (e.g. 87 %) to the neighbouring steps.
        assert_eq!(zoom_in(87.3), 100);
        assert_eq!(zoom_out(87.3), 75);
        assert_eq!(zoom_in(1000.0), 400);
        assert_eq!(zoom_out(10.0), 25);
    }

    // AC2
    #[test]
    fn visible_page() {
        // Three A4 pages, 800 logical pixels high each.
        let stack = PageStack::new(&[800.0, 800.0, 800.0]);
        assert_eq!(stack.height(), 12.0 + 3.0 * 812.0);
        let viewport = 600.0;

        assert_eq!(stack.visible_page(0.0, viewport), 0);
        // Page 2 starts at 824: at 500 page 1 still has most of the area.
        assert_eq!(stack.visible_page(500.0, viewport), 0);
        // At 550, page 2 shows 326 of 600 pixels.
        assert_eq!(stack.visible_page(550.0, viewport), 1);
        assert_eq!(stack.visible_page(stack.height() - viewport, viewport), 2);
        assert_eq!(stack.pages_between(500.0, 1100.0), 0..2);
    }

    // AC5
    #[test]
    fn zoom_keeps_page_in_view() {
        let at_100 = PageStack::new(&[800.0, 800.0, 800.0]);
        // A quarter into page 2.
        let offset = 824.0 + 200.0;
        let anchor = at_100.anchor(offset);
        assert_eq!(anchor.0, 1);
        assert!((anchor.1 - 0.25).abs() < 1e-9);

        let at_200 = PageStack::new(&[1600.0, 1600.0, 1600.0]);
        let new_offset = at_200.offset_of(anchor);

        assert!((new_offset - (12.0 + 1612.0 + 400.0)).abs() < 1e-9);
        assert_eq!(at_200.visible_page(new_offset, 600.0), 1);
    }

    // AC6
    #[test]
    fn render_size_is_sharp() {
        assert_eq!(render_width(800.0, 1), 800);
        assert_eq!(render_width(800.0, 2), 1600);
        // 400 % of an A4 page (595 pt) at scale factor 2.
        assert_eq!(render_width(595.0 * 4.0, 2), 4760);
    }
}

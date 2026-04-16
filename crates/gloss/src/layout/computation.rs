use bitflags::bitflags;
use geometry::{Bounded, Edges, Point, Rect, Size};

bitflags! {
    #[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
    pub struct Dirty: u8 {
        const Style   = 1 << 0;
        const Measure = 1 << 1;
        const Layout  = 1 << 2;
    }
}

#[derive(Debug, Clone)]
pub struct Computation {
    pub(crate) cache: taffy::Cache,
    pub(crate) unrounded_layout: taffy::Layout,
    pub(crate) final_layout: taffy::Layout,
    pub(crate) dirty: Dirty,
}

impl Computation {
    pub fn border(&self) -> Edges {
        Edges::new(
            self.final_layout.border.left.max(0.0) as u16,
            self.final_layout.border.top.max(0.0) as u16,
            self.final_layout.border.right.max(0.0) as u16,
            self.final_layout.border.bottom.max(0.0) as u16,
        )
    }

    pub fn padding(&self) -> Edges {
        Edges::new(
            self.final_layout.padding.left.max(0.0) as u16,
            self.final_layout.padding.top.max(0.0) as u16,
            self.final_layout.padding.right.max(0.0) as u16,
            self.final_layout.padding.bottom.max(0.0) as u16,
        )
    }

    pub fn margin(&self) -> Edges {
        Edges::new(
            self.final_layout.margin.left.max(0.0) as u16,
            self.final_layout.margin.top.max(0.0) as u16,
            self.final_layout.margin.right.max(0.0) as u16,
            self.final_layout.margin.bottom.max(0.0) as u16,
        )
    }

    /// Returns the outer bounds of the node, the "border box".
    pub fn border_bounds(&self) -> Rect {
        self.bounds()
    }

    /// Returns the inner bounds of the node, the "content box".
    pub fn content_bounds(&self) -> Rect {
        let layout = &self.final_layout;

        Rect::new(
            layout.content_box_x() as u16,
            layout.content_box_y() as u16,
            layout.content_box_width() as u16,
            layout.content_box_height() as u16,
        )
    }

    pub(crate) fn mark(&mut self, dirty: Dirty) {
        self.dirty.set(dirty, true);
    }

    pub(crate) fn unmark(&mut self, dirty: Dirty) {
        self.dirty.set(dirty, false);
    }

    pub(crate) fn is_dirty(&self) -> bool {
        !self.dirty.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.dirty = Dirty::empty();
    }
}

impl Bounded for Computation {
    type Coordinate = Point;
    type Bounds = Rect;

    fn min_x(&self) -> u16 {
        self.final_layout.location.x.max(0.0) as u16
    }

    fn min_y(&self) -> u16 {
        self.final_layout.location.y.max(0.0) as u16
    }

    fn max_x(&self) -> u16 {
        (self.final_layout.location.x + self.final_layout.size.width).max(0.0) as u16
    }

    fn max_y(&self) -> u16 {
        (self.final_layout.location.y + self.final_layout.size.height).max(0.0) as u16
    }

    fn min(&self) -> Self::Coordinate {
        Point {
            x: self.min_x(),
            y: self.min_y(),
        }
    }

    fn max(&self) -> Self::Coordinate {
        Point {
            x: self.max_x(),
            y: self.max_y(),
        }
    }

    fn bounds(&self) -> Self::Bounds {
        Rect {
            min: self.min(),
            max: self.max(),
        }
    }
}

impl Default for Computation {
    fn default() -> Self {
        Self {
            cache: taffy::Cache::default(),
            unrounded_layout: taffy::Layout::default(),
            final_layout: taffy::Layout::default(),
            dirty: Dirty::all()
        }
    }
}

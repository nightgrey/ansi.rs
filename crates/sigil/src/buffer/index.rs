use super::{Buffer, Cell};
use geometry::{Point, Position, Row};
use std::ops;
use std::ops::{Add, Mul};
use std::slice::SliceIndex;

pub trait BufferIndex: Sized {
    type Output: ?Sized;
    type SliceIndex: SliceIndex<[Cell], Output = Self::Output>;

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex>;

    unsafe fn unchecked_index_of(self, buffer: &Buffer) -> Self::SliceIndex {
        self.index_of(buffer).unwrap_unchecked()
    }

    fn within(&self, buffer: &Buffer) -> bool;

    /// Returns a shared reference to the output at this location, if in
    /// bounds.
    fn get(self, buffer: &Buffer) -> Option<&Self::Output> {
        let index = self.index_of(buffer)?;

        index.get(buffer.as_slice())
    }

    /// Returns a mutable reference to the output at this location, if in
    /// bounds.
    fn get_mut(self, buffer: &mut Buffer) -> Option<&mut Self::Output> {
        let index = self.index_of(buffer)?;

        index.get_mut(buffer.as_mut_slice())
    }

    /// Returns a pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    unsafe fn get_unchecked(self, buffer: *const Buffer) -> *const Self::Output {
        let index = self.unchecked_index_of(&*buffer);

        index.get_unchecked((&*buffer).as_slice() as *const [Cell])
    }

    /// Returns a mutable pointer to the output at this location, without
    /// performing any bounds checking.
    ///
    /// Calling this method with an out-of-bounds index or a dangling `slice` pointer
    /// is *[undefined behavior]* even if the resulting pointer is not used.
    ///
    /// [undefined behavior]: https://doc.rust-lang.org/reference/behavior-considered-undefined.html
    unsafe fn get_unchecked_mut(self, buffer: *mut Buffer) -> *mut Self::Output {
        let index = self.unchecked_index_of(&*buffer);

        index.get_unchecked_mut((&mut *buffer).as_mut_slice() as *mut [Cell])
    }

    /// Returns a shared reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    fn index(self, buffer: &Buffer) -> &Self::Output {
        let index = unsafe { self.unchecked_index_of(buffer) };

        index.index(buffer)
    }

    /// Returns a mutable reference to the output at this location, panicking
    /// if out of bounds.
    #[track_caller]
    fn index_mut(self, buffer: &mut Buffer) -> &mut Self::Output {
        let index = unsafe { self.unchecked_index_of(buffer) };

        index.index_mut(buffer)
    }
}

impl BufferIndex for usize {
    type SliceIndex = usize;
    type Output = Cell;

    fn within(&self, buffer: &Buffer) -> bool {
        *self < buffer.len()
    }

    #[inline]
    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer).then_some(self)
    }
}

impl BufferIndex for ops::Range<usize> {
    type Output = [Cell];
    type SliceIndex = ops::Range<usize>;

    fn within(&self, buffer: &Buffer) -> bool {
        self.start <= self.end && self.end <= buffer.len()
    }

    #[inline]
    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer).then_some(self)
    }
}
impl BufferIndex for ops::RangeInclusive<usize> {
    type Output = [Cell];
    type SliceIndex = ops::RangeInclusive<usize>;

    fn within(&self, buffer: &Buffer) -> bool {
        self.start() < self.end() && *self.end() < buffer.len()
    }

    #[inline]
    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer).then_some(self)
    }
}
impl BufferIndex for ops::RangeFrom<usize> {
    type Output = [Cell];
    type SliceIndex = ops::RangeFrom<usize>;

    fn within(&self, buffer: &Buffer) -> bool {
        self.start < buffer.len()
    }

    #[inline]
    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer).then_some(self)
    }
}
impl BufferIndex for ops::RangeTo<usize> {
    type Output = [Cell];
    type SliceIndex = ops::RangeTo<usize>;

    fn within(&self, buffer: &Buffer) -> bool {
        self.end <= buffer.len()
    }

    #[inline]
    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer).then_some(self)
    }
}
impl BufferIndex for ops::RangeToInclusive<usize> {
    type Output = [Cell];
    type SliceIndex = ops::RangeToInclusive<usize>;

    fn within(&self, buffer: &Buffer) -> bool {
        self.end < buffer.len()
    }

    #[inline]
    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer).then_some(self)
    }
}
impl BufferIndex for ops::RangeFull {
    type Output = [Cell];
    type SliceIndex = ops::RangeFull;

    fn within(&self, _: &Buffer) -> bool {
        true
    }

    #[inline]
    fn index_of(self, _: &Buffer) -> Option<Self::SliceIndex> {
        Some(..)
    }
}

impl BufferIndex for Position {
    type SliceIndex = usize;
    type Output = Cell;

    fn within(&self, buffer: &Buffer) -> bool {
        self.col < buffer.width && self.row < buffer.height
    }

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer)
            .then_some(self.row * buffer.width + self.col)
    }
}

impl BufferIndex for Point {
    type SliceIndex = usize;
    type Output = Cell;

    fn within(&self, buffer: &Buffer) -> bool {
        self.y < buffer.height && self.x < buffer.width
    }

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer)
            .then_some(self.y * buffer.width + self.x)
    }
}

impl BufferIndex for Row {
    type Output = [Cell];
    type SliceIndex = ops::Range<usize>;

    fn within(&self, buffer: &Buffer) -> bool {
        self.0 < buffer.height
    }
    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer)
            .then_some(*self * buffer.width..(*self + 1) * buffer.width)
    }
}

impl BufferIndex for ops::Range<Row> {
    type Output = [Cell];
    type SliceIndex = ops::Range<usize>;

    fn within(&self, buffer: &Buffer) -> bool {
        *self.start < buffer.height && *self.end < buffer.height
    }

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer)
            .then_some(*self.start * buffer.width..*self.end * buffer.width + buffer.width)
    }
}
impl BufferIndex for ops::RangeInclusive<Row> {
    type Output = [Cell];
    type SliceIndex = ops::RangeInclusive<usize>;

    fn within(&self, buffer: &Buffer) -> bool {
        **self.start() < buffer.height && **self.end() <= buffer.height
    }

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer)
            .then_some(**self.start() * buffer.width..=**self.end() * buffer.width + buffer.width)
    }
}
impl BufferIndex for ops::RangeFrom<Row> {
    type Output = [Cell];
    type SliceIndex = ops::RangeFrom<usize>;

    fn within(&self, buffer: &Buffer) -> bool {
        *self.start < buffer.height
    }

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer).then_some(*self.start * buffer.width..)
    }
}
impl BufferIndex for ops::RangeTo<Row> {
    type Output = [Cell];
    type SliceIndex = ops::RangeTo<usize>;

    fn within(&self, buffer: &Buffer) -> bool {
        *self.end < buffer.height
    }

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer)
            .then_some(..*self.end * buffer.width + buffer.width)
    }
}
impl BufferIndex for ops::RangeToInclusive<Row> {
    type Output = [Cell];
    type SliceIndex = ops::RangeToInclusive<usize>;

    fn within(&self, buffer: &Buffer) -> bool {
        *self.end <= buffer.height
    }

    fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
        self.within(buffer)
            .then_some(..=*self.end * buffer.width + buffer.width)
    }
}

impl<I: BufferIndex> ops::Index<I> for Buffer {
    type Output = <I::SliceIndex as SliceIndex<[Cell]>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        index.index(self)
    }
}

impl<I: BufferIndex> ops::IndexMut<I> for Buffer {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        index.index_mut(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use geometry::Rect;

    fn create_buffer() -> Buffer {
        Buffer::new(10, 5) // 10 cols, 5 rows = 50 cells
    }

    // === Index by usize ===

    #[test]
    fn test_index_usize() {
        let buffer = create_buffer();
        assert!(buffer.get(0).is_some());
        assert!(buffer.get(49).is_some()); // Last cell
        assert!(buffer.get(50).is_none()); // Out of bounds
    }

    #[test]
    fn test_index_usize_out_of_bounds() {
        let buffer = create_buffer();
        assert!(buffer.get(100).is_none());
    }

    // === Index by Position ===

    #[test]
    fn test_index_position() {
        let buffer = create_buffer();

        // Valid positions
        assert!(buffer.get(Position::new(0, 0)).is_some());
        assert!(buffer.get(Position::new(4, 9)).is_some()); // Last cell (row 4, col 9)

        // Out of bounds
        assert!(buffer.get(Position::new(5, 0)).is_none()); // Row too large
        assert!(buffer.get(Position::new(0, 10)).is_none()); // Col too large
    }

    #[test]
    fn test_position_to_linear_index() {
        let buffer = create_buffer();

        // Position (0, 0) should be index 0
        assert_eq!(Position::new(0, 0).index_of(&buffer), Some(0));

        // Position (0, 5) should be index 5
        assert_eq!(Position::new(0, 5).index_of(&buffer), Some(5));

        // Position (1, 0) should be index 10 (start of second row)
        assert_eq!(Position::new(1, 0).index_of(&buffer), Some(10));

        // Position (1, 5) should be index 15
        assert_eq!(Position::new(1, 5).index_of(&buffer), Some(15));
    }

    #[test]
    fn test_position_to_linear_calculation() {
        let buffer = create_buffer();
        let width = buffer.width; // 10

        // row * width + col
        let pos = Position::new(3, 7);
        let expected_index = 3 * width + 7; // = 37
        assert_eq!(pos.index_of(&buffer), Some(expected_index));
    }

    // === Index by Row ===

    #[test]
    fn test_index_row() {
        let buffer = create_buffer();

        // Row 0
        let row0 = buffer.get(Row(0));
        assert!(row0.is_some());
        assert_eq!(row0.unwrap().len(), 10); // Full row width

        // Row 4 (last row)
        let row4 = buffer.get(Row(4));
        assert!(row4.is_some());

        // Row 5 (out of bounds)
        assert!(buffer.get(Row(5)).is_none());
    }

    #[test]
    fn test_index_row_range() {
        let buffer = create_buffer();

        let rows = buffer.get(Row(0)..Row(2));
        assert!(rows.is_some());
        assert_eq!(rows.unwrap().len(), 30); // 3 rows * 10 cols (includes row at end)
    }

    // === Index by usize Range variants ===

    #[test]
    fn test_index_range_usize() {
        let buffer = create_buffer();
        let slice = buffer.get(0..10);
        assert!(slice.is_some());
        assert_eq!(slice.unwrap().len(), 10);
    }

    #[test]
    fn test_index_range_inclusive_usize() {
        let buffer = create_buffer();
        let slice = buffer.get(0..=9);
        assert!(slice.is_some());
        assert_eq!(slice.unwrap().len(), 10);
    }

    #[test]
    fn test_index_range_from_usize() {
        let buffer = create_buffer();
        let slice = buffer.get(40..);
        assert!(slice.is_some());
        assert_eq!(slice.unwrap().len(), 10); // Cells 40-49
    }

    #[test]
    fn test_index_range_to_usize() {
        let buffer = create_buffer();
        let slice = buffer.get(..10);
        assert!(slice.is_some());
        assert_eq!(slice.unwrap().len(), 10);
    }

    #[test]
    fn test_index_range_full() {
        let buffer = create_buffer();
        let slice = buffer.get(..);
        assert!(slice.is_some());
        assert_eq!(slice.unwrap().len(), 50); // All cells
    }

    // === Index mutation ===

    #[test]
    fn test_index_mut_usize() {
        let mut buffer = create_buffer();
        buffer[0].set_char('X');
        assert_eq!(buffer[0].as_str(), "X");
    }

    #[test]
    fn test_index_mut_position() {
        let mut buffer = create_buffer();
        let pos = Position::new(2, 5);
        buffer[pos].set_char('Y');
        assert_eq!(buffer[pos].as_str(), "Y");
    }

    // === Bounds checking ===

    #[test]
    fn test_buffer_contains_position() {
        let buffer = create_buffer();

        // Inside bounds
        assert!(buffer.contains(Position::new(0, 0)));
        assert!(buffer.contains(Position::new(4, 9)));

        // Outside bounds
        assert!(!buffer.contains(Position::new(5, 0)));
        assert!(!buffer.contains(Position::new(0, 10)));
    }

    // === Edge cases ===

    #[test]
    fn test_zero_width_buffer() {
        let buffer = Buffer::new(0, 5);
        assert_eq!(buffer.width, 0);
        assert!(buffer.get(Position::new(0, 0)).is_none());
    }

    #[test]
    fn test_zero_height_buffer() {
        let buffer = Buffer::new(10, 0);
        assert_eq!(buffer.height, 0);
        assert!(buffer.get(Position::new(0, 0)).is_none());
    }
}

// impl BufferIndex for ops::Range<Position> {
//     type Output = [Cell];
//     type SliceIndex = ops::Range<usize>;
//
//     fn within(&self, buffer: &Buffer) -> bool {
//         self.start.within(buffer) && self.end.within(buffer)
//     }
//
//     fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
//         self.within(buffer).then_some(
//             self.start.row * buffer.width + self.start.col
//                 ..self.end.row * buffer.width + self.end.col,
//         )
//     }
// }
// impl BufferIndex for ops::RangeInclusive<Position> {
//     type SliceIndex = ops::RangeInclusive<usize>;
//     type Output = [Cell];
//
//     fn within(&self, buffer: &Buffer) -> bool {
//         self.start().within(buffer) && self.end().within(buffer)
//     }
//
//     fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
//         self.within(buffer).then_some(
//             self.start().row * buffer.width + self.start().col
//                 ..=self.end().row * buffer.width + self.end().col,
//         )
//     }
// }
// impl BufferIndex for ops::RangeFrom<Position> {
//     type SliceIndex = ops::RangeFrom<usize>;
//     type Output = [Cell];
//     fn within(&self, buffer: &Buffer) -> bool {
//         self.start.within(buffer)
//     }
//
//     fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
//         self.within(buffer)
//             .then_some(self.start.row * buffer.width + self.start.col..)
//     }
// }
// impl BufferIndex for ops::RangeTo<Position> {
//     type SliceIndex = ops::RangeTo<usize>;
//     type Output = [Cell];
//     fn within(&self, buffer: &Buffer) -> bool {
//         self.end.within(buffer)
//     }
//
//     fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
//         self.within(buffer)
//             .then_some(..self.end.row * buffer.width + self.end.col)
//     }
// }
// impl BufferIndex for ops::RangeToInclusive<Position> {
//     type SliceIndex = ops::RangeToInclusive<usize>;
//     type Output = [Cell];
//     fn within(&self, buffer: &Buffer) -> bool {
//         self.end.within(buffer)
//     }
//
//     fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
//         self.within(buffer)
//             .then_some(..=self.end.row * buffer.width + self.end.col)
//     }
// }
//
// impl BufferIndex for ops::Range<Point> {
//     type SliceIndex = ops::Range<usize>;
//     type Output = [Cell];
//     fn within(&self, buffer: &Buffer) -> bool {
//         self.start.within(buffer) && self.end.within(buffer)
//     }
//
//     fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
//         self.within(buffer).then_some(
//             self.start.y * buffer.width + self.start.x..self.end.y * buffer.width + self.end.x,
//         )
//     }
// }
// impl BufferIndex for ops::RangeInclusive<Point> {
//     type SliceIndex = ops::RangeInclusive<usize>;
//     type Output = [Cell];
//
//     fn within(&self, buffer: &Buffer) -> bool {
//         self.start().within(buffer) && self.end().within(buffer)
//     }
//
//     fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
//         self.within(buffer).then_some(
//             self.start().y * buffer.width + self.start().x
//                 ..=self.end().y * buffer.width + self.end().x,
//         )
//     }
// }
// impl BufferIndex for ops::RangeFrom<Point> {
//     type SliceIndex = ops::RangeFrom<usize>;
//     type Output = [Cell];
//
//     fn within(&self, buffer: &Buffer) -> bool {
//         self.start.within(buffer)
//     }
//
//     fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
//         self.within(buffer)
//             .then_some(self.start.y * buffer.width + self.start.x..)
//     }
// }
// impl BufferIndex for ops::RangeTo<Point> {
//     type SliceIndex = ops::RangeTo<usize>;
//     type Output = [Cell];
//     fn within(&self, buffer: &Buffer) -> bool {
//         self.end.within(buffer)
//     }
//     fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
//         self.within(buffer)
//             .then_some(..self.end.y * buffer.width + self.end.x)
//     }
// }
// impl BufferIndex for ops::RangeToInclusive<Point> {
//     type SliceIndex = ops::RangeToInclusive<usize>;
//     type Output = [Cell];
//     fn within(&self, buffer: &Buffer) -> bool {
//         self.end.within(buffer)
//     }
//     fn index_of(self, buffer: &Buffer) -> Option<Self::SliceIndex> {
//         let end = &self.end;
//
//         self.within(buffer)
//             .then_some(..=end.y * buffer.width + end.x)
//     }
// }

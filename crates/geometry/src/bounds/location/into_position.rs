use crate::{Column,  Position, Row, Location, Bounds, Step};

/// Provides the spatial context needed to convert between positions.
pub const trait IntoLocation<T = Position> {
    fn into_index(&self, location: T) -> usize;

    fn into_position(&self, location: T) -> Position;

    fn into_row(&self, location: T) -> Row;

    fn into_col(&self, location: T) -> Column;
}

impl const IntoLocation<Position> for Bounds {
    fn into_index(&self, location: Position) -> usize {
        (location.row - self.min.row) * self.width() + (location.col - self.min.col)
    }
    fn into_position(&self, location: Position) -> Position {
        location
    }

    fn into_row(&self, location: Position) -> Row {
        Row((location.row - self.min.row) / self.width())
    }

    fn into_col(&self, location: Position) -> Column {
        Column((location.col - self.min.col) % self.width())
    }
}
impl const IntoLocation<Row> for Bounds {
    fn into_index(&self, location: Row) -> usize {
        (location.value() - self.min.row) * self.width()
    }

    fn into_position(&self, location: Row) -> Position {
        let w = self.width();
        Position::new(self.min.row + location.value() / w, self.min.col)
    }

    fn into_row(&self, location: Row) -> Row {
        Row((location.value() - self.min.row) / self.width())
    }

    fn into_col(&self, location: Row) -> Column {
        Column((location.value() - self.min.row) % self.width())
    }
}
impl const IntoLocation<Column> for Bounds {
    fn into_index(&self, location: Column) -> usize {
        (location.0) * self.width()
    }

    fn into_position(&self, location: Column) -> Position {
        Position::new(self.min.row, self.min.col + location.0 % self.width())
    }

    fn into_row(&self, location: Column) -> Row {
        Row((location.0 - self.min.col) / self.width())
    }

    fn into_col(&self, location: Column) -> Column {
        Column((location.0 - self.min.col) % self.width())
    }
}
impl const IntoLocation<usize> for Bounds {
    fn into_index(&self, location: usize) -> usize {
        location
    }

    fn into_position(&self, location: usize) -> Position {
        Position::new(self.min.row + location / self.width(), self.min.col + location % self.width())
    }

    fn into_row(&self, location: usize) -> Row {
        Row((location - self.min.row) / self.width())
    }

    fn into_col(&self, location: usize) -> Column {
        Column((location - self.min.col) % self.width())
    }
}

pub const trait IntoLocationWithin: Sized {
    #[inline]
    fn into_index(self, ctx: &Bounds) -> usize where Bounds: [const] IntoLocation<Self> {
        ctx.into_index(self)
    }

    #[inline]
    fn into_position(self, ctx: &Bounds) -> Position where Bounds: [const] IntoLocation<Self> {
        ctx.into_position(self)
    }

    #[inline]
    fn into_row(self, ctx: &Bounds) -> Row where Bounds: [const] IntoLocation<Self> {
        ctx.into_row(self)
    }

    #[inline]
    fn into_col(self, ctx: &Bounds) -> Column where Bounds: [const] IntoLocation<Self> {
        ctx.into_col(self)
    }
}

impl<S: Location> const IntoLocationWithin for S {
}
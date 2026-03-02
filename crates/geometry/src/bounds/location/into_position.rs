use crate::{Column, Position, Row, Location, Bounds, Context};

/// Provides the spatial context needed to convert between location representations.
pub const trait IntoLocation<T = Position> {
    fn into_index(&self, location: T) -> usize;

    fn into_position(&self, location: T) -> Position;

    fn into_row(&self, location: T) -> Row;

    fn into_col(&self, location: T) -> Column;
}

impl<T: [const] Context> const IntoLocation<Position> for T {
    fn into_index(&self, location: Position) -> usize {
        (location.row - self.min().row) * self.width() + (location.col - self.min().col)
    }
    fn into_position(&self, location: Position) -> Position {
        location
    }

    fn into_row(&self, location: Position) -> Row {
        Row(location.row - self.min().row)
    }

    fn into_col(&self, location: Position) -> Column {
        Column(location.col - self.min().col)
    }
}

impl<T: [const] Context> const IntoLocation<Row> for T {
    /// Index of the first cell in this row.
    fn into_index(&self, location: Row) -> usize {
        (location.value() - self.min().row) * self.width()
    }

    /// Position of the first cell in this row.
    fn into_position(&self, location: Row) -> Position {
        Position::new(location.value(), self.min().col)
    }

    fn into_row(&self, location: Row) -> Row {
        Row(location.value() - self.min().row)
    }

    fn into_col(&self, _location: Row) -> Column {
        Column(0)
    }
}

impl<T: [const] Context> const IntoLocation<Column> for T {
    /// Index of the first cell in this column (i.e. in the first row).
    fn into_index(&self, location: Column) -> usize {
        location.value() - self.min().col
    }

    /// Position of the first cell in this column.
    fn into_position(&self, location: Column) -> Position {
        Position::new(self.min().row, location.value())
    }

    fn into_row(&self, _location: Column) -> Row {
        Row(0)
    }

    fn into_col(&self, location: Column) -> Column {
        Column(location.value() - self.min().col)
    }
}

impl<T: [const] Context> const IntoLocation<usize> for T {
    fn into_index(&self, location: usize) -> usize {
        location
    }

    fn into_position(&self, location: usize) -> Position {
        Position::new(
            self.min().row + location / self.width(),
            self.min().col + location % self.width(),
        )
    }

    fn into_row(&self, location: usize) -> Row {
        Row(location / self.width())
    }

    fn into_col(&self, location: usize) -> Column {
        Column(location % self.width())
    }
}

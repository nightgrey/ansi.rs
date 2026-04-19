#[macro_export]
macro_rules! position {
    () => {
        Position::ZERO
    };
    ($row:expr, $col:expr) => {
        Position {
            row: $row as usize,
            col: $col as usize,
        }
    };
    ($value:expr) => {
        Position {
            row: $value as usize,
            col: $value as usize,
        }
    };
}

#[macro_export]
macro_rules! pos {
    () => {
        (0usize, 0usize)
    };
    ($row:expr, $col:expr) => {
        ($row as usize, $col as usize)
    };
    ($value:expr) => {
        ($value as usize, $value as usize)
    };
}

#[macro_export]
macro_rules! point {
    () => {
        Point::ZERO
    };
    ($x:expr, $y:expr) => {
        Point {
            x: $x as u16,
            y: $y as u16,
        }
    };
    ($value:expr) => {
        Point {
            x: $value as u16,
            y: $value as u16,
        }
    };
}

#[macro_export]
macro_rules! rect {
    () => {
        Rect::ZERO
    };
    ($x:expr, $y:expr, $w:expr, $h:expr) => {
        Rect::new($x as u16, $y as u16, $w as u16, $h as u16)
    };
    (($min_x:expr, $min_y:expr), ($max_x:expr, $max_y:expr)) => {
        Rect::bounds(
            Point::new($min_x as u16, $min_y as u16),
            Point::new($max_x as u16, $max_y as u16),
        )
    };
    ($min:expr, $max:expr) => {
        Rect::bounds($min, $max)
    };
}

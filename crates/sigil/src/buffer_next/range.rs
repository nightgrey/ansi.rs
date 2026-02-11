use geometry::Position;
use crate::Buffer;

trait BufferSelector: Sized {
    type Iter: Iterator<Item = Position>;

    fn select(self, buffer: &Buffer) -> Self::Iter;

    fn len(self, buffer: &Buffer) -> usize {
        self.select(buffer).count()
    }

    fn is_empty(self, buffer: &Buffer) -> bool {
        self.len(buffer) == 0
    }
}

impl BufferSelector for Position {
    type Iter = std::iter::Once<Self>;

    fn select(self, _: &Buffer) -> Self::Iter {
        std::iter::once(self)
    }
}

impl BufferSelector for usize {
    type Iter = std::iter::Once<Position>;

    fn select(self, buffer: &Buffer) -> Self::Iter {
        std::iter::once(Position::new(self % buffer.width, self / buffer.width))
    }
}
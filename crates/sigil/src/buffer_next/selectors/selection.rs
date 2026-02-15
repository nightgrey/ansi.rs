use geometry::{Bounds, Col, Position, Row};
use super::super::{Buffer, Cell, Index};
use super::{Selector};

pub enum Selection {
    Index(usize ),
    Position(Position),
    Row(Row),
    Col(Col),
    Bounds(Bounds),
}

impl Selection {
    pub fn iter(self, buffer: &Buffer) -> SelectionIter {
        match self {
            Selection::Index(i) => SelectionIter::Index(i.iter(buffer)),
            Selection::Position(p) => SelectionIter::Position(p.iter(buffer)),
            Selection::Row(r)   => SelectionIter::Row(r.iter(buffer)),
            Selection::Col(c)   => SelectionIter::Col(c.iter(buffer)),
            Selection::Bounds(r)  => SelectionIter::Bounds(r.iter(buffer)),
        }
    }
}
pub enum SelectionIter {
    Index(<usize as Selector>::Iter),
    Position(<Position as Selector>::Iter),
    Row(<Row as Selector>::Iter),
    Col(<Col as Selector>::Iter),
    Bounds(<Bounds as Selector>::Iter),
}

impl Iterator for SelectionIter {
    type Item = usize;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            SelectionIter::Index(it)  => it.next(),
            SelectionIter::Position(it) => it.next(),
            SelectionIter::Row(it) => it.next(),
            SelectionIter::Col(it) => it.next(),
            SelectionIter::Bounds(it) => it.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            SelectionIter::Index(it) => it.size_hint(),
            SelectionIter::Position(it) => it.size_hint(),
            SelectionIter::Row(it) => it.size_hint(),
            SelectionIter::Col(it) => it.size_hint(),
            SelectionIter::Bounds(it) => it.size_hint(),
        }
    }
}

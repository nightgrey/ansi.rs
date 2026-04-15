use crate::{Sides, Column, Edges, Point, PointLike, Rect, Row, Size};
use crate::{Zero, One, Min, Max};

impl<T: Zero> Zero for Point<T> { const ZERO: Self = Point { x: T::ZERO, y: T::ZERO }; }
impl<T: One> One for Point<T> { const ONE: Self = Point { x: T::ONE, y: T::ONE }; }
impl<T: Min> Min for Point<T> { const MIN: Self = Point { x: T::MIN, y: T::MIN }; }
impl<T: Max> Max for Point<T> { const MAX: Self = Point { x: T::MAX, y: T::MAX }; }

impl<T: Zero> Zero for PointLike<T> { const ZERO: Self = (T::ZERO, T::ZERO); }
impl<T: One> One for PointLike<T> { const ONE: Self = (T::ONE, T::ONE); }
impl<T: Min> Min for PointLike<T> { const MIN: Self = (T::MIN, T::MIN); }
impl<T: Max> Max for PointLike<T> { const MAX: Self = (T::MAX, T::MAX); }

impl Zero for Row { const ZERO: Self = Row(0); }
impl One for Row { const ONE: Self = Row(1); }
impl Min for Row { const MIN: Self = Row(0); }
impl Max for Row { const MAX: Self = Row(usize::MAX); }

impl Zero for Column { const ZERO: Self = Column(0); }
impl One for Column { const ONE: Self = Column(1); }
impl Min for Column { const MIN: Self = Column(0); }
impl Max for Column { const MAX: Self = Column(usize::MAX); }

impl<T: Zero> Zero for Rect<T> { const ZERO: Self = Rect { min: T::ZERO, max: T::ZERO }; }
impl<T: One> One for Rect<T> { const ONE: Self = Rect { min: T::ONE, max: T::ONE }; }
impl<T: Min> Min for Rect<T> { const MIN: Self = Rect { min: T::MIN, max: T::MIN }; }
impl<T: Max> Max for Rect<T> { const MAX: Self = Rect { min: T::MAX, max: T::MAX }; }

impl<T: Zero> Zero for Size<T> { const ZERO: Self = Size { width: T::ZERO, height: T::ZERO }; }
impl<T: One> One for Size<T> { const ONE: Self = Size { width: T::ONE, height: T::ONE }; }
impl<T: Min> Min for Size<T> { const MIN: Self = Size { width: T::MIN, height: T::MIN }; }
impl<T: Max> Max for Size<T> { const MAX: Self = Size { width: T::MAX, height: T::MAX }; }

impl<T: Zero> Zero for Edges<T> { const ZERO: Self = Edges { top: T::ZERO, right: T::ZERO, bottom: T::ZERO, left: T::ZERO }; }
impl<T: One> One for Edges<T> { const ONE: Self = Edges { top: T::ONE, right: T::ONE, bottom: T::ONE, left: T::ONE }; }
impl<T: Min> Min for Edges<T> { const MIN: Self = Edges { top: T::MIN, right: T::MIN, bottom: T::MIN, left: T::MIN }; }
impl<T: Max> Max for Edges<T> { const MAX: Self = Edges { top: T::MAX, right: T::MAX, bottom: T::MAX, left: T::MAX }; }

impl<T: Zero> Zero for Sides<T> { const ZERO: Self = Sides { horizontal: T::ZERO, vertical: T::ZERO }; }
impl<T: One> One for Sides<T> { const ONE: Self = Sides { horizontal: T::ONE, vertical: T::ONE }; }
impl<T: Min> Min for Sides<T> { const MIN: Self = Sides { horizontal: T::MIN, vertical: T::MIN }; }
impl<T: Max> Max for Sides<T> { const MAX: Self = Sides { horizontal: T::MAX, vertical: T::MAX }; }
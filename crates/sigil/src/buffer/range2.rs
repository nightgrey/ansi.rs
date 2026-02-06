use geometry::Bounds;
use crate::Buffer;
use crate::buffer::range::BufferRange;

impl Buffer {

    pub fn text2(
        &mut self,
        range: impl BufferRange,
    ) {
        dbg!(range.positions(self).collect::<Vec<_>>());
    }

}
#[test]
fn qwe() {

    let mut buffer = Buffer::new(10, 10);

    buffer.text2(Bounds::bounds(0, 0, 3, 3));
}
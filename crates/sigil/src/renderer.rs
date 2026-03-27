use crate::{Buffer, Document, ElementId, GraphemeArena, Layer, LayerId, Rasterizer};
use tree::Map;

#[derive(Debug)]
pub struct Renderer {
    pub front: Buffer,
    rasterizer: Rasterizer,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            front: Buffer::new(width, height),
            rasterizer: Rasterizer::new(width, height),
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.front.resize_inner(width, height);
        self.rasterizer.resize(width, height);
    }

    pub fn clear(&mut self) {
        self.front.clear();
        self.rasterizer.clear();
    }

    /// Composite layers into a target buffer, recursively walking children sorted by z_index.
    pub(crate) fn composite(buffer: &mut Buffer, document: &Document, id: ElementId) {
        let layer = &document.layers[id];
        for row in 0..layer.height {
            let front_row = layer.position.y + row;
            if front_row >= buffer.height {
                continue;
            }

            for col in 0..layer.width {
                let front_col = layer.position.x + col;
                if front_col >= buffer.width {
                    continue;
                }

                let cell = layer[(row, col)];
                if !cell.is_empty() {
                    buffer[(front_row, front_col)] = cell;
                }
            }
        }

        let mut children: Vec<_> = document.elements.children(id).collect();
        children.sort_by_key(|&child| document.layers[child].z_index);

        for child in children {
            Self::composite(buffer, document, child);
        }
    }

    pub(crate) fn raster(&mut self, arena: &GraphemeArena) -> std::io::Result<()> {
        self.rasterizer.raster(&self.front, arena)
    }
    pub(crate) fn flush(
        &mut self,
        arena: &GraphemeArena,
        output: &mut impl std::io::Write,
    ) -> std::io::Result<()> {
        self.rasterizer.flush(output)
    }
}

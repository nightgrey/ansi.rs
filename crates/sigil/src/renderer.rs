use crate::{Buffer, GraphemeArena, LayerId, Layers, Rasterizer};

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
    pub(crate) fn composite(buffer: &mut Buffer, layers: &Layers, id: LayerId) {
        let layer = &layers[id];
        for row in 0..layer.height {
            let front_row = layer.position.row + row;
            if front_row >= buffer.height {
                continue;
            }

            for col in 0..layer.width {
                let front_col = layer.position.col + col;
                if front_col >= buffer.width {
                    continue;
                }

                let cell = layer[(row, col)];
                if !cell.is_empty() {
                    buffer[(front_row, front_col)] = cell;
                }
            }
        }

        let mut children: Vec<_> = layers.children(id).collect();
        children.sort_by_key(|child| layers[*child].z_index);

        for child in children {
            Self::composite(buffer, layers, child);
        }
    }

    pub(crate) fn raster(&mut self, arena: &GraphemeArena) {
        self.rasterizer.render(&self.front, arena)
    }
    pub(crate) fn flush(&mut self, arena: &GraphemeArena, output: &mut impl std::io::Write) -> std::io::Result<()> {
        self.rasterizer.flush(output)
    }

}

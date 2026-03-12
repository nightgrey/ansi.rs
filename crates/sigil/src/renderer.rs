use crate::{Buffer, GraphemeArena, LayerId, Layers, Rasterizer};

#[derive(Debug)]
pub struct Renderer {
    pub front: Buffer,
    pub back: Buffer,
    pub(crate) arena: GraphemeArena,
    rasterizer: Rasterizer,
}

impl Renderer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            front: Buffer::new(width, height),
            back: Buffer::new(width, height),
            arena: GraphemeArena::new(),
            rasterizer: Rasterizer::new(width, height),
        }
    }

    pub(crate) fn composite(&mut self, layers: &mut Layers, id: LayerId) {
        let layer = &layers[id];
        for row in 0..layer.height {
            let front_row = layer.position.row + row;
            if front_row >= self.front.height {
                continue;
            }

            for col in 0..layer.width {
                let front_col = layer.position.col + col;
                if front_col >= self.front.width {
                    continue;
                }

                let cell = layer[(row, col)];
                if !cell.is_empty() {
                    self.front[(front_row, front_col)] = cell;
                }
            }
        }

        let mut children: Vec<_> = layers.children(id).collect();
        children.sort_by_key(|child| layers[*child].z_index);

        for child in children {
            self.composite(layers, child);
        }
    }

    pub(crate) fn render(&mut self, output: &mut impl std::io::Write) -> std::io::Result<()> {
        self.rasterizer.render(&self.front, &self.arena);
        self.rasterizer.flush(output)
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.front.resize_inner(width, height);
        self.back.resize_inner(width, height);
        self.rasterizer.resize(width, height);
    }

    pub fn clear(&mut self) {
        self.front.clear();
        self.back.clear();
        self.rasterizer.clear();
        self.arena.clear();
    }
}


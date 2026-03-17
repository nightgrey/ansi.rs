use std::ops::{Deref, DerefMut, Index, IndexMut};
use crate::{Element, ElementId, ElementKind, GraphemeArena, Layer};
use tree::{RootTree, Node, layout::prelude::*, id, Secondary, LayoutNode, LayoutContext, Tree, Map, Error, Layouted};
use geometry::{Rect, Size};
use grid::{Spatial};
use tree::table::Table;

pub struct DocumentNode<'a> {
    pub id: ElementId,
    pub document: &'a Document,
}

impl<'a> DocumentNode<'a> {
    pub fn new(id: ElementId, document: &'a Document) -> Self {
        Self { document, id }
    }

   pub fn layout_node(&self) -> &LayoutNode {
        &self.document.layouts[self.id]
    }

    pub fn bounds(&self) -> &Rect {
        &self.document.bounds[self.id]
    }

    pub fn layer(&self) -> &Layer {
        &self.document.layers[self.id]
    }
}

impl Deref for DocumentNode<'_> {
    type Target = Element;

    fn deref(&self) -> &Self::Target {
        &self.document[self.id]
    }
}


pub struct DocumentNodeMut<'a> {
    pub id: ElementId,
    pub document: &'a mut Document,

}

impl<'a> DocumentNodeMut<'a> {
    pub fn new(id: ElementId, document: &'a mut Document) -> Self {
        Self { id, document }
    }

    pub fn layout_node(&self) -> &LayoutNode {
        &self.document.layouts[self.id]
    }

    pub fn layout_node_mut(&mut self) -> &mut LayoutNode {
        &mut self.document.layouts[self.id]
    }

    pub fn bounds(&self) -> &Rect {
        &self.document.bounds[self.id]
    }

    pub fn bounds_mut(&mut self) -> &mut Rect {
        &mut self.document.bounds[self.id]
    }

    pub fn layer(&self) -> &Layer {
        &self.document.layers[self.id]
    }

    pub fn layer_mut(&mut self) -> &mut Layer {
        &mut self.document.layers[self.id]
    }
}

impl Deref for DocumentNodeMut<'_> {
    type Target = Element;

    fn deref(&self) -> &Self::Target {
        &self.document[self.id]
    }
}

impl DerefMut for DocumentNodeMut<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.document[self.id]
    }
}

id!(pub struct LayerId);
#[derive(Debug)]
pub struct Document {
    pub elements: RootTree<ElementId, Element>,
    pub layouts: Secondary<ElementId, LayoutNode>,
    pub bounds: Secondary<ElementId, Rect>,
    pub layers: Table<LayerId, Layer, ElementId>,
    pub arena: GraphemeArena,
}

impl Document {
    pub fn new(width: usize, height: usize) -> Self {
        let mut elements = RootTree::new(Element::Div());
        let mut layouts = Secondary::new();
        let mut bounds = Secondary::new();
        let mut layers = Table::new();

        layers.insert(elements.root_id(), Layer::new(width, height));
        layouts.insert(elements.root_id(), LayoutNode::default());

        Self { elements, layouts, layers, bounds, arena: GraphemeArena::new() }
    }

    pub fn size(&self) -> Size {
        self.layers[self.elements.root_id()].size()
    }
    pub fn root_id(&self) -> ElementId {
        self.elements.root_id()
    }

    pub fn root(&self) -> DocumentNode<'_> {
        DocumentNode::new(self.root_id(), self)
    }

    pub fn root_mut(&mut self) -> DocumentNodeMut<'_> {
        DocumentNodeMut::new(self.root_id(), self)
    }

    pub fn insert(&mut self, element: Element) -> ElementId {
        let id = self.elements.insert(element);
        self.layouts.insert(id, LayoutNode::default());
        id
    }


    pub fn compute_layers(
        &mut self,
    ) {
        let root_id = self.root_id();
        let layer_id = self.layers.get_id(root_id).unwrap();
        self.compute_layer(root_id, layer_id);
    }

    pub fn compute_layer(
        &mut self,
        id: ElementId,
        layer_id: LayerId,
    ) {
        let layer_id = self.layers.get_id(id).unwrap_or(layer_id);

        for child_id in self.elements.children(id).collect::<Vec<_>>() {
            let next_layer_id = if self.elements[child_id].is_promoting() {
                match self.layers.get_id(child_id) {
                    Some(child_layer_id) => if layer_id != child_layer_id {
                        child_layer_id
                    } else {
                        let size = self.layers[child_id].size();
                        self.layers.insert(child_id, Layer::new(size.width, size.height))
                    },
                    _ => layer_id,
                }
            } else {
                layer_id
            };

            self.compute_layer(child_id, next_layer_id);
        }
    }

    pub fn compute_layouts(&mut self) {
        let viewport = self.size();
        let root_id = self.elements.root_id();

        // Compute layout
        AsLayoutContext::compute_layout_with_measure(self, root_id, LayoutSize {
            width: AvailableSpace::Definite(viewport.width as f32),
            height: AvailableSpace::Definite(viewport.height as f32),
        }, |known_dimensions, available_space, id, node_context, layout| {
            // match &self[id].kind {
            //     ElementKind::Span(content) => {
            //         let words = content.lines().collect::<Vec<_>>();
            //         let min_line_length: usize = words.iter().map(|line| line.len()).max().unwrap_or(0);
            //         let max_line_length: usize = words.iter().map(|line| line.len()).sum();
            //
            //         let inline_axis = AbsoluteAxis::Horizontal;
            //         let inline_size =
            //             known_dimensions.get_abs(inline_axis).unwrap_or_else(|| match available_space.get_abs(inline_axis) {
            //                 AvailableSpace::MinContent => min_line_length as f32,
            //                 AvailableSpace::MaxContent => max_line_length as f32,
            //                 AvailableSpace::Definite(inline_size) => inline_size
            //                     .min(max_line_length as f32)
            //                     .max(min_line_length as f32),
            //             });
            //
            //         let block_axis = inline_axis.other_axis();
            //
            //         let block_size = known_dimensions.get_abs(block_axis).unwrap_or_else(|| {
            //             let inline_line_length = (inline_size).floor() as usize;
            //             let mut line_count = 1;
            //             let mut current_line_length = 0;
            //             for word in &words {
            //                 if current_line_length == 0 {
            //                     // first word
            //                     current_line_length = word.len();
            //                 } else if current_line_length + word.len() + 1 > inline_line_length {
            //                     // every word past the first needs to check for line length including the space between words
            //                     // note: a real implementation of this should handle whitespace characters other than ' '
            //                     // and do something more sophisticated for long words
            //                     line_count += 1;
            //                     current_line_length = word.len();
            //                 } else {
            //                     // add the word and a space
            //                     current_line_length += word.len() + 1;
            //                 };
            //             }
            //             (line_count as f32)
            //         });
            //
            //          LayoutSize { width: inline_size, height: block_size }
            //     }
            //     _ => {
            //         match (known_dimensions.width, known_dimensions.height) {
            //             (Some(width), Some(height)) => LayoutSize { width, height },
            //             (Some(width), None) => LayoutSize { width, height: f32::MAX },
            //             (None, Some(height)) => LayoutSize { width: f32::MAX, height },
            //             (None, None) => LayoutSize { width: f32::MAX, height: f32::MAX },
            //         }
            //     }
            // }

            LayoutSize { width: 5.0, height: 5.0 }
        });

        self.compute_bounds(root_id, 0.0, 0.0);
    }

    fn compute_bounds(&mut self, id: ElementId, offset_x: f32, offset_y: f32) {
        let computation = self.layouts[id].final_computation;
        let x = offset_x + computation.location.x;
        let y = offset_y + computation.location.y;
        let w = computation.size.width;
        let h = computation.size.height;

        self.bounds.insert(id, Rect::bounds(x as usize, y as usize, w as usize, h as usize));

        for child_id in self.elements.children(id).collect::<Vec<_>>() {
            self.compute_bounds(child_id, x, y);
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        let root_id = self.root_id();
        let layer = self.layers.get_mut(root_id).unwrap();
        layer.resize(width, height);
        self.bounds.insert(root_id, Rect::bounds(0, 0, width, height));
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.layers.clear();
        self.arena.clear();
        // Re-sync: clear taffy and re-add root node
        self.layouts.clear();

        let viewport = self.size();
        self.bounds.insert(self.root_id(), Rect::bounds(0, 0, viewport.width, viewport.height));
    }
}

impl Layouted for Element {
    fn layout(&self) -> &Layout {
        &self.layout
    }
}

impl AsLayoutContext<ElementId, Element, Rect> for Document {
    fn as_context<MeasureFunction: FnMut(LayoutSize<Option<f32>>, LayoutSize<AvailableSpace>, ElementId, Option<&mut Rect>, &Layout) -> LayoutSize<f32>>(&mut self, measure: MeasureFunction) -> LayoutContext<'_, ElementId, Element, Rect, MeasureFunction> {
        LayoutContext {
            tree: &mut self.elements,
            layouts: &mut self.layouts,
            contexts: &mut self.bounds,
            measure_function: measure,
        }
    }

    fn use_rounding(&self) -> bool {
        true
    }
}

impl Index<ElementId> for Document {
    type Output = Element;

    fn index(&self, index: ElementId) -> &Self::Output {
        &self.elements[index]
    }
}

impl IndexMut<ElementId> for Document {
    fn index_mut(&mut self, index: ElementId) -> &mut Self::Output {
        &mut self.elements[index]
    }
}

impl Index<LayerId> for Document {
    type Output = Layer;

    fn index(&self, index: LayerId) -> &Self::Output {
        self.layers.get_direct(index).unwrap()
    }
}

impl IndexMut<LayerId> for Document {
    fn index_mut(&mut self, index: LayerId) -> &mut Self::Output {
        self.layers.get_direct_mut(index).unwrap()
    }
}
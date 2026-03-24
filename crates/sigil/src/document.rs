use smallvec::SmallVec;
use std::ops::{Deref, DerefMut, Index, IndexMut};
use crate::{Element, ElementId, ElementKind, GraphemeArena, Layer};
use tree::{RootTree, layout::prelude::*, id, Secondary, LayoutNode, LayoutContext, At};
use geometry::{Rect, Size};
use spatial::{Spatial};
use tree::table::Table;

pub struct DocumentNode<'a> {
    pub id: ElementId,
    pub document: &'a Document,
}

impl<'a> DocumentNode<'a> {
    pub fn new(id: ElementId, document: &'a Document) -> Self {
        Self { document, id }
    }

    pub fn layout(&self) -> &LayoutNode {
        &self.document.layouts[self.id]
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

    pub fn layout(&self) -> &LayoutNode {
        &self.document.layouts[self.id]
    }

    pub fn layout_mut(&mut self) -> &mut LayoutNode {
        &mut self.document.layouts[self.id]
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
    pub layers: Table<LayerId, Layer, ElementId>,
    pub arena: GraphemeArena,
}

impl Document {
    pub fn new(width: usize, height: usize) -> Self {
        let mut elements = RootTree::new(Element::div());
        let mut layouts = Secondary::new();
        let mut layers = Table::new();

        layers.insert(elements.root_id(), Layer::new(width, height));
        layouts.insert(elements.root_id(), LayoutNode::new(Layout {
            size: layout::Size::from_lengths(width as f32, height as f32),
            ..Layout::default()
        }));

        Self { elements, layouts, layers, arena: GraphemeArena::new() }
    }

    pub fn size(&self) -> Size {
        self.layers[self.elements.root_id()].size()
    }
    pub fn root_id(&self) -> ElementId {
        self.elements.root_id()
    }

    pub fn root(&self) -> DocumentNode<'_> {
        self.get(self.root_id())
    }

    pub fn root_mut(&mut self) -> DocumentNodeMut<'_> {
        self.get_mut(self.root_id())
    }

    pub fn get(&self, id: ElementId) -> DocumentNode<'_> {
        DocumentNode::new(id, self)
    }

    pub fn get_mut(&mut self, id: ElementId) -> DocumentNodeMut<'_> {
        DocumentNodeMut::new(id, self)
    }

    pub fn get_layout(&self, id: ElementId) -> Option<&Layout> {
        self.layouts.get(id).map(|l| &l.layout)
    }

    pub fn get_layout_mut(&mut self, id: ElementId) -> Option<&mut Layout> {
        self.layouts.get_mut(id).map(|l| &mut l.layout)
    }

    pub fn get_computation(&self, id: ElementId) -> Option<&LayoutComputation> {
        self.layouts.get(id).map(|l| &l.final_computation)
    }

    pub fn get_computation_mut(&mut self, id: ElementId) -> Option<&mut LayoutComputation> {
        self.layouts.get_mut(id).map(|l| &mut l.final_computation)
    }

    /// Inserts an element as a child of root with a default layout.
    pub fn insert(&mut self, element: Element) -> ElementId {
        self.insert_with_layout(element, Layout::default())
    }

    /// Inserts an element as a child of root with the given layout.
    pub fn insert_with_layout(&mut self, element: Element, layout: Layout) -> ElementId {
        let id = self.elements.insert(element);
        self.layouts.insert(id, LayoutNode::new(layout));
        id
    }

    pub fn insert_at(&mut self, element: Element, at: At<ElementId>) -> ElementId {
        self.insert_at_with_layout(element, at, Layout::default())
    }

    pub fn insert_at_with_layout(&mut self, element: Element, at: At<ElementId>, layout: Layout) -> ElementId {
        let id = self.elements.insert_at(element, at);
        self.layouts.insert(id, LayoutNode::new(layout));
        id
    }

    /// Inserts an element as a child of `parent` with a default layout.
    pub fn insert_child(&mut self, parent: ElementId, element: Element) -> ElementId {
        self.insert_child_with_layout(parent, element, Layout::default())
    }

    /// Inserts an element as a child of `parent` with the given layout.
    pub fn insert_child_with_layout(&mut self, parent: ElementId, element: Element, layout: Layout) -> ElementId {
        self.insert_at_with_layout(element, At::Child(parent), layout)
    }

    /// Sets (or replaces) the layout for an existing element.
    pub fn set_layout(&mut self, id: ElementId, layout: Layout) {
        self.layouts.insert(id, LayoutNode::new(layout));
    }

    /// Removes an element and its descendants.
    pub fn remove(&mut self, id: ElementId) -> Option<SmallVec<ElementId, 4>> {
        // Returns `None` if `id` is the root.
        match self.elements.remove(id) {
            Some(removed) => {
                for &id in &removed {
                    self.layouts.remove(id);
                    self.layers.remove(id);
                }

                Some(removed)
            }
            None => None,
        }
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
        self.layers.relate(id, layer_id).unwrap();

        for child_id in self.elements.children(id).collect::<Vec<_>>() {
            let next_layer_id = if self.elements[child_id].is_promoting() {
                unreachable!("Element {:?} is promoting but has no layer", child_id);
            } else {
                layer_id
            };

            self.compute_layer(child_id, next_layer_id);
        }
    }

    pub fn compute_layouts(&mut self) {
        for (id, node) in &self.elements {
            node.layout.clone_into(&mut self.layouts[id].layout);
        }

        let viewport = self.size();
        let root_id = self.elements.root_id();

        // Compute layout
        AsLayoutContext::compute_layout_with_measure(self, root_id, layout::Size {
            width: AvailableSpace::Definite(viewport.width as f32),
            height: AvailableSpace::Definite(viewport.height as f32),
        }, |known_dimensions, available_space, id, element, layout| {
            match &element.kind {
                ElementKind::Span(content) => {
                    let words: Vec<&str> = content.split_whitespace().collect();
                    let min_line_length: usize = words.iter().map(|line| line.len()).max().unwrap_or(0);
                    let max_line_length: usize = words.iter().map(|line| line.len()).sum();

                    let inline_axis = AbsoluteAxis::Horizontal;
                    let inline_size =
                        known_dimensions.get_abs(inline_axis).unwrap_or_else(|| match available_space.get_abs(inline_axis) {
                            AvailableSpace::MinContent => min_line_length as f32,
                            AvailableSpace::MaxContent => max_line_length as f32,
                            AvailableSpace::Definite(inline_size) => inline_size
                                .min(max_line_length as f32)
                                .max(min_line_length as f32),
                        });

                    let block_axis = inline_axis.other_axis();

                    let block_size = known_dimensions.get_abs(block_axis).unwrap_or_else(|| {
                        let inline_line_length = (inline_size).floor() as usize;
                        let mut line_count = 1;
                        let mut current_line_length = 0;
                        for word in &words {
                            if current_line_length == 0 {
                                // first word
                                current_line_length = word.len();
                            } else if current_line_length + word.len() + 1 > inline_line_length {
                                // every word past the first needs to check for line length including the space between words
                                // note: a real implementation of this should handle whitespace characters other than ' '
                                // and do something more sophisticated for long words
                                line_count += 1;
                                current_line_length = word.len();
                            } else {
                                // add the word and a space
                                current_line_length += word.len() + 1;
                            };
                        }
                        (line_count as f32)
                    });

                     layout::Size { width: inline_size, height: block_size }
                }
                _ => {
                    match (known_dimensions.width, known_dimensions.height) {
                        (Some(width), Some(height)) => layout::Size { width, height },
                        (Some(width), None) => layout::Size { width, height: f32::MAX },
                        (None, Some(height)) => layout::Size { width: f32::MAX, height },
                        (None, None) => layout::Size { width: f32::MAX, height: f32::MAX },
                    }
                }
            }

        });
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        let mut root = self.root_mut();
        root.layer_mut().resize(width, height);
        root.layout_mut().size = layout::Size::AUTO;
        root.layout_mut().cache.clear();
    }

    pub fn clear(&mut self) {
        self.elements.clear();
        self.layers.clear();
        self.arena.clear();
        self.layouts.clear();
    }
}

impl AsLayoutContext<ElementId, Element, Rect> for Document {
    fn as_context<MeasureFunction>(
        &mut self,
        measure: MeasureFunction,
    ) -> LayoutContext<'_, ElementId, Element, MeasureFunction> where MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, ElementId, &mut Element, &Layout) -> layout::Size<f32> {
        LayoutContext {
            tree: &mut self.elements,
            layouts: &mut self.layouts,
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
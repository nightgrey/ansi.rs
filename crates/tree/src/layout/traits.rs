use crate::{Id, LayoutContext};
use crate::{prelude::*, Bridge};


pub trait AsLayoutContext<K: Id, V, Context = ()> {
    fn as_context<MeasureFunction>(
        &mut self,
        measure: MeasureFunction,
    ) -> LayoutContext<'_, K, V, MeasureFunction> where MeasureFunction: FnMut(layout::Size<Option<f32>>, layout::Size<AvailableSpace>, K, &mut V, &Layout) -> layout::Size<f32>;

    fn use_rounding(&self) -> bool;

    fn compute_layout(&mut self, id: K, available_space: layout::Size<AvailableSpace>)
    {
        let mut context = self.as_context(|_, _, _, _, _| layout::Size::ZERO);
        taffy::compute_root_layout(&mut context, id.into_layout_id(), available_space);
    }

    fn compute_layout_with_measure<MeasureFunction: FnMut(
        layout::Size<Option<f32>>,
        layout::Size<AvailableSpace>,
        K,
        &mut V,
        &Layout,
    ) -> layout::Size<f32>>(
        &mut self,
        id: K,
        available_space: layout::Size<AvailableSpace>,
        measure: MeasureFunction,
    )
    {
        let key = id.into_layout_id();
        let use_rounding = self.use_rounding();
        let mut context = self.as_context(measure);
        taffy::compute_root_layout(&mut context, key, available_space);
        if use_rounding {
            taffy::round_layout(&mut context, key);
        }
    }

    fn print_tree(&mut self, root: K)
    {
        taffy::util::print_tree(&self.as_context(|_, _, _, _, _| layout::Size::ZERO), root.into_layout_id());
    }
}

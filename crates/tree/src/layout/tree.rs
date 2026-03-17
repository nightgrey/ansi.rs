use crate::{Id, LayoutContext};
use crate::{prelude::*, Bridge};


pub trait Layouted {
    fn layout(&self) -> &Layout;
}

pub trait AsLayoutContext<K: Id, V, Context = ()> {
    fn as_context<MeasureFunction: FnMut(
        LayoutSize<Option<f32>>,
        LayoutSize<AvailableSpace>,
        K,
        Option<&mut Context>,
        &Layout,
    ) -> LayoutSize<f32>>(
        &mut self,
        measure: MeasureFunction,
    ) -> LayoutContext<'_, K, V, Context, MeasureFunction>;

    fn use_rounding(&self) -> bool;

    fn compute_layout(&mut self, id: K, available_space: LayoutSize<AvailableSpace>) where V: Layouted
    {
        let mut context = self.as_context(|_, _, _, _, _| LayoutSize::ZERO);
        taffy::compute_root_layout(&mut context, id.into_layout_id(), available_space);
    }

    fn compute_layout_with_measure<MeasureFunction: FnMut(
        LayoutSize<Option<f32>>,
        LayoutSize<AvailableSpace>,
        K,
        Option<&mut Context>,
        &Layout,
    ) -> LayoutSize<f32>>(
        &mut self,
        id: K,
        available_space: LayoutSize<AvailableSpace>,
        measure: MeasureFunction,
    ) where V: Layouted
    {
        let key = id.into_layout_id();
        let use_rounding = self.use_rounding();
        let mut context = self.as_context(measure);
        taffy::compute_root_layout(&mut context, key, available_space);
        if use_rounding {
            taffy::round_layout(&mut context, key);
        }
    }

    fn print_tree(&mut self, root: K) where V: Layouted
    {
        taffy::util::print_tree(&self.as_context(|_, _, _, _, _| LayoutSize::ZERO), root.into_layout_id());
    }
}

use ansi::Color;

use super::layout::*;
use crate::Layout;

pub trait Layouted: Sized {
    /// Returns a reference to the style memory of this element.
    fn layout(&mut self) -> &mut Layout;

    fn padding(mut self, padding: impl Into<Edges>) -> Self {
        self.layout().padding = padding.into();
        self
    }

    fn margin(mut self, margin: impl Into<Edges>) -> Self {
        self.layout().margin = margin.into();
        self
    }

    fn display(mut self, display: Display) -> Self {
        self.layout().display = display;
        self
    }

    /// Sets the display type of the element to `block`.
    /// [Docs](https://tailwindcss.com/docs/display)
    fn block(mut self) -> Self {
        self.layout().display = Display::Block;
        self
    }

    /// Sets the display type of the element to `flex`.
    /// [Docs](https://tailwindcss.com/docs/display)
    fn flex(mut self) -> Self {
        self.layout().display = Display::Flex;
        self
    }

    /// Sets the display type of the element to `none`.
    /// [Docs](https://tailwindcss.com/docs/display)
    fn hidden(mut self) -> Self {
        self.layout().display = Display::None;
        self
    }

    fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.layout().flex_direction = direction;
        self
    }

    /// Sets the flex direction of the element to `column`.
    /// [Docs](https://tailwindcss.com/docs/flex-direction#column)
    fn flex_col(mut self) -> Self {
        self.layout().flex_direction = FlexDirection::Column;
        self
    }

    /// Sets the flex direction of the element to `column-reverse`.
    /// [Docs](https://tailwindcss.com/docs/flex-direction#column-reverse)
    fn flex_col_reverse(mut self) -> Self {
        self.layout().flex_direction = FlexDirection::ColumnReverse;
        self
    }

    /// Sets the flex direction of the element to `row`.
    /// [Docs](https://tailwindcss.com/docs/flex-direction#row)
    fn flex_row(mut self) -> Self {
        self.layout().flex_direction = FlexDirection::Row;
        self
    }

    /// Sets the flex direction of the element to `row-reverse`.
    /// [Docs](https://tailwindcss.com/docs/flex-direction#row-reverse)
    fn flex_row_reverse(mut self) -> Self {
        self.layout().flex_direction = FlexDirection::RowReverse;
        self
    }

    /// Sets the element to allow a flex item to grow and shrink as needed, ignoring its initial size.
    /// [Docs](https://tailwindcss.com/docs/flex#flex-1)
    fn flex_1(mut self) -> Self {
        self.layout().flex_grow = 1.;
        self.layout().flex_shrink = 1.;
        self.layout().flex_basis = Length::Percent(0.);
        self
    }

    /// Sets the element to allow a flex item to grow and shrink, taking into account its initial size.
    /// [Docs](https://tailwindcss.com/docs/flex#auto)
    fn flex_auto(mut self) -> Self {
        self.layout().flex_grow = 1.;
        self.layout().flex_shrink = 1.;
        self.layout().flex_basis = Length::Auto;
        self
    }

    /// Sets the element to allow a flex item to shrink but not grow, taking into account its initial size.
    /// [Docs](https://tailwindcss.com/docs/flex#initial)
    fn flex_initial(mut self) -> Self {
        self.layout().flex_grow = 0.;
        self.layout().flex_shrink = 1.;
        self.layout().flex_basis = Length::Auto;
        self
    }

    /// Sets the element to prevent a flex item from growing or shrinking.
    /// [Docs](https://tailwindcss.com/docs/flex#none)
    fn flex_none(mut self) -> Self {
        self.layout().flex_grow = 0.;
        self.layout().flex_shrink = 0.;
        self
    }

    /// Sets the initial size of flex items for this element.
    /// [Docs](https://tailwindcss.com/docs/flex-basis)
    fn flex_basis(mut self, basis: impl Into<Length>) -> Self {
        self.layout().flex_basis = basis.into();
        self
    }

    /// Sets the element to allow a flex item to grow to fill any available space.
    /// [Docs](https://tailwindcss.com/docs/flex-grow)
    fn flex_grow(mut self) -> Self {
        self.layout().flex_grow = 1.;
        self
    }

    /// Sets the element to prevent a flex item from growing.
    /// [Docs](https://tailwindcss.com/docs/flex-grow#dont-grow)
    fn flex_grow_0(mut self) -> Self {
        self.layout().flex_grow = 0.;
        self
    }

    /// Sets the element to allow a flex item to shrink if needed.
    /// [Docs](https://tailwindcss.com/docs/flex-shrink)
    fn flex_shrink(mut self) -> Self {
        self.layout().flex_shrink = 1.;
        self
    }

    /// Sets the element to prevent a flex item from shrinking.
    /// [Docs](https://tailwindcss.com/docs/flex-shrink#dont-shrink)
    fn flex_shrink_0(mut self) -> Self {
        self.layout().flex_shrink = 0.;
        self
    }

    /// Sets the element to allow flex items to wrap.
    /// [Docs](https://tailwindcss.com/docs/flex-wrap#wrap-normally)
    fn flex_wrap(mut self) -> Self {
        self.layout().flex_wrap = FlexWrap::Wrap;
        self
    }

    /// Sets the element wrap flex items in the reverse direction.
    /// [Docs](https://tailwindcss.com/docs/flex-wrap#wrap-reversed)
    fn flex_wrap_reverse(mut self) -> Self {
        self.layout().flex_wrap = FlexWrap::WrapReverse;
        self
    }

    /// Sets the element to prevent flex items from wrapping, causing inflexible items to overflow the container if necessary.
    /// [Docs](https://tailwindcss.com/docs/flex-wrap#dont-wrap)
    fn flex_nowrap(mut self) -> Self {
        self.layout().flex_wrap = FlexWrap::NoWrap;
        self
    }

    /// Sets the element to align flex items to the start of the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-items#start)
    fn items_start(mut self) -> Self {
        self.layout().align_items = Some(AlignItems::Start);
        self
    }

    /// Sets the element to align flex items to the end of the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-items#end)
    fn items_end(mut self) -> Self {
        self.layout().align_items = Some(AlignItems::End);
        self
    }

    /// Sets the element to align flex items along the center of the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-items#center)
    fn items_center(mut self) -> Self {
        self.layout().align_items = Some(AlignItems::Center);
        self
    }

    /// Sets the element to align flex items along the baseline of the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-items#baseline)
    fn items_baseline(mut self) -> Self {
        self.layout().align_items = Some(AlignItems::Baseline);
        self
    }

    /// Sets the element to stretch flex items to fill the available space along the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-items#stretch)
    fn items_stretch(mut self) -> Self {
        self.layout().align_items = Some(AlignItems::Stretch);
        self
    }

    /// Sets how this specific element is aligned along the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-self#start)
    fn self_start(mut self) -> Self {
        self.layout().align_self = Some(AlignSelf::Start);
        self
    }

    /// Sets this element to align against the end of the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-self#end)
    fn self_end(mut self) -> Self {
        self.layout().align_self = Some(AlignSelf::End);
        self
    }

    /// Sets this element to align against the start of the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-self#start)
    fn self_flex_start(mut self) -> Self {
        self.layout().align_self = Some(AlignSelf::Start);
        self
    }

    /// Sets this element to align against the end of the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-self#end)
    fn self_flex_end(mut self) -> Self {
        self.layout().align_self = Some(AlignSelf::End);
        self
    }

    /// Sets this element to align along the center of the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-self#center)
    fn self_center(mut self) -> Self {
        self.layout().align_self = Some(AlignSelf::Center);
        self
    }

    /// Sets this element to align along the baseline of the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-self#baseline)
    fn self_baseline(mut self) -> Self {
        self.layout().align_self = Some(AlignSelf::Baseline);
        self
    }

    /// Sets this element to stretch to fill the available space along the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-self#stretch)
    fn self_stretch(mut self) -> Self {
        self.layout().align_self = Some(AlignSelf::Stretch);
        self
    }

    /// Sets the element to justify flex items against the start of the container's main axis.
    /// [Docs](https://tailwindcss.com/docs/justify-content#start)
    fn justify_start(mut self) -> Self {
        self.layout().justify_content = Some(JustifyContent::Start);
        self
    }

    /// Sets the element to justify flex items against the end of the container's main axis.
    /// [Docs](https://tailwindcss.com/docs/justify-content#end)
    fn justify_end(mut self) -> Self {
        self.layout().justify_content = Some(JustifyContent::End);
        self
    }

    /// Sets the element to justify flex items along the center of the container's main axis.
    /// [Docs](https://tailwindcss.com/docs/justify-content#center)
    fn justify_center(mut self) -> Self {
        self.layout().justify_content = Some(JustifyContent::Center);
        self
    }

    /// Sets the element to justify flex items along the container's main axis
    /// such that there is an equal amount of space between each item.
    /// [Docs](https://tailwindcss.com/docs/justify-content#space-between)
    fn justify_between(mut self) -> Self {
        self.layout().justify_content = Some(JustifyContent::SpaceBetween);
        self
    }

    /// Sets the element to justify items along the container's main axis such
    /// that there is an equal amount of space on each side of each item.
    /// [Docs](https://tailwindcss.com/docs/justify-content#space-around)
    fn justify_around(mut self) -> Self {
        self.layout().justify_content = Some(JustifyContent::SpaceAround);
        self
    }

    /// Sets the element to justify items along the container's main axis such
    /// that there is an equal amount of space around each item, but also
    /// accounting for the doubling of space you would normally see between
    /// each item when using justify-around.
    /// [Docs](https://tailwindcss.com/docs/justify-content#space-evenly)
    fn justify_evenly(mut self) -> Self {
        self.layout().justify_content = Some(JustifyContent::SpaceEvenly);
        self
    }

    /// Sets the element to pack content items in their default position as if no align-content value was set.
    /// [Docs](https://tailwindcss.com/docs/align-content#normal)
    fn content_none(mut self) -> Self {
        self.layout().align_content = None;
        self
    }

    /// Sets the element to pack content items in the center of the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-content#center)
    fn content_center(mut self) -> Self {
        self.layout().align_content = Some(AlignContent::Center);
        self
    }

    /// Sets the element to pack content items against the start of the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-content#start)
    fn content_start(mut self) -> Self {
        self.layout().align_content = Some(AlignContent::Start);
        self
    }

    /// Sets the element to pack content items against the end of the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-content#end)
    fn content_end(mut self) -> Self {
        self.layout().align_content = Some(AlignContent::End);
        self
    }

    /// Sets the element to pack content items along the container's cross axis
    /// such that there is an equal amount of space between each item.
    /// [Docs](https://tailwindcss.com/docs/align-content#space-between)
    fn content_between(mut self) -> Self {
        self.layout().align_content = Some(AlignContent::SpaceBetween);
        self
    }

    /// Sets the element to pack content items along the container's cross axis
    /// such that there is an equal amount of space on each side of each item.
    /// [Docs](https://tailwindcss.com/docs/align-content#space-around)
    fn content_around(mut self) -> Self {
        self.layout().align_content = Some(AlignContent::SpaceAround);
        self
    }

    /// Sets the element to pack content items along the container's cross axis
    /// such that there is an equal amount of space between each item.
    /// [Docs](https://tailwindcss.com/docs/align-content#space-evenly)
    fn content_evenly(mut self) -> Self {
        self.layout().align_content = Some(AlignContent::SpaceEvenly);
        self
    }

    /// Sets the element to allow content items to fill the available space along the container's cross axis.
    /// [Docs](https://tailwindcss.com/docs/align-content#stretch)
    fn content_stretch(mut self) -> Self {
        self.layout().align_content = Some(AlignContent::Stretch);
        self
    }

    /// Sets the background color of the element.
    fn bg<F>(mut self, color: F) -> Self
    where
        F: Into<Color>,
        Self: Sized,
    {
        self.layout().background = Some(color.into());
        self
    }

    /// Sets the border style of the element.
    fn border(mut self, border: Border) -> Self {
        self.layout().border = border;
        self
    }

    /// Sets the background color of this element.
    ///
    /// This value cascades to its child elements.
    fn background(mut self, bg: impl Into<Color>) -> Self {
        self.layout().background = Some(bg.into());
        self
    }

    /// Sets the text color of this element.
    ///
    /// This value cascades to its child elements.
    fn color(mut self, color: impl Into<Color>) -> Self {
        self.layout().color = Some(color.into());
        self
    }

    /// Sets the font weight of this element
    ///
    /// This value cascades to its child elements.
    fn font_weight(mut self, weight: FontWeight) -> Self {
        self.layout().font_weight = Some(weight);
        self
    }

    fn bold(mut self) -> Self {
        self.layout().font_weight = Some(FontWeight::Bold);
        self
    }

    /// Sets the font style of the element to italic.
    /// [Docs](https://tailwindcss.com/docs/font-style#italicizing-text)
    fn italic(mut self) -> Self {
        self.layout().font_style = Some(FontStyle::Italic);
        self
    }

    /// Sets the font style of the element to normal (not italic).
    /// [Docs](https://tailwindcss.com/docs/font-style#displaying-text-normally)
    fn normal(mut self) -> Self {
        self.layout().font_style = Some(FontStyle::Normal);
        self
    }

    /// Sets the text decoration to underline.
    /// [Docs](https://tailwindcss.com/docs/text-decoration-line#underling-text)
    fn underline(mut self) -> Self {
        self.layout().text_decoration = Some(TextDecoration::Underline);
        self
    }

    /// Sets the decoration of the text to have a line through it.
    /// [Docs](https://tailwindcss.com/docs/text-decoration-line#adding-a-line-through-text)
    fn strikethrough(mut self) -> Self {
        self.layout().text_decoration = Some(TextDecoration::Strikethrough);
        self
    }

    /// Removes the text decoration on this element.
    ///
    /// This value cascades to its child elements.
    fn text_decoration_none(mut self) -> Self {
        self.layout().text_decoration = None;
        self
    }

    // /// Draws a debug border around this element.
    // #[cfg(debug_assertions)]
    // fn debug(mut self) -> Self {
    //     self.layout().debug = Some(true);
    //     self
    // }

    // /// Draws a debug border on all conforming elements below this element.
    // #[cfg(debug_assertions)]
    // fn debug_below(mut self) -> Self {
    //     self.layout().debug_below = Some(true);
    //     self
    // }
}

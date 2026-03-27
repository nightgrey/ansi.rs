// Re-exports of [`taffy`] types, renamed to fit this crate's conventions.
//
// | Taffy name   | Re-exported as   |
// |--------------|------------------|
// | `Style`      | `Layout`         |
// | `Layout`     | `Computation` |
// | `NodeId`     | `LayoutId`       |
// | `Cache`      | `LayoutCache`    |
pub use taffy::geometry::{self as layout};

pub use taffy::{
    AbsoluteAxis, Cache as LayoutCache, CacheTree as CacheLayoutTree, Layout as LayoutComputation,
    LayoutInput, LayoutOutput, NodeId as LayoutNodeId, RunMode, Style as Layout,
    style::{
        AlignContent, AlignItems, AlignSelf, AvailableSpace, BoxSizing, CompactLength, Dimension,
        Display, FlexDirection, FlexWrap, GridAutoFlow, GridPlacement, GridTemplateComponent,
        JustifyContent, JustifyItems, JustifySelf, LengthPercentage, LengthPercentageAuto,
        MaxTrackSizingFunction, MinTrackSizingFunction, Position, RepetitionCount,
        TrackSizingFunction,
    },
    style_helpers::{
        FromFr, FromLength, FromPercent, TaffyAuto as LayoutAuto,
        TaffyFitContent as LayoutFitContent, TaffyGridLine as LayoutGridLine,
        TaffyGridSpan as LayoutGridSpan, TaffyMaxContent as LayoutMaxContent,
        TaffyMinContent as LayoutMinContent, TaffyZero as LayoutZero, auto, fit_content, length,
        max_content, min_content, percent, zero,
    },
    tree::{
        LayoutBlockContainer, LayoutFlexboxContainer, LayoutGridContainer, LayoutPartialTree,
        PrintTree as PrintLayoutTree, RoundTree as RoundLayoutTree,
        TraversePartialTree as TraverseLayoutPartialTree, TraverseTree as TraverseLayoutTree,
    },
};

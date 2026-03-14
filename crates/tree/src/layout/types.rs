// Re-export and re-name taffy types to adapt to the crate's `LayoutTree` ergonomics.

pub use taffy::{
    Style as Layout, Cache as LayoutCache, Layout as Computation, NodeId as InternalLayoutId,
    LayoutInput, LayoutOutput, RunMode,
    geometry::{Line, Rect, Size},
    style::{
        AlignContent, AlignItems, AlignSelf, AvailableSpace, BoxSizing, CompactLength, Dimension, Display,
        JustifyContent, JustifyItems, JustifySelf, LengthPercentage, LengthPercentageAuto, Position,
        FlexDirection, FlexWrap,

        GridAutoFlow, GridPlacement, GridTemplateComponent, MaxTrackSizingFunction, MinTrackSizingFunction,
        RepetitionCount, TrackSizingFunction,
    },
    style_helpers::{
        auto, fit_content, length, max_content, min_content, percent, zero, FromFr, FromLength, FromPercent,
        TaffyAuto as LayoutAuto,
        TaffyFitContent as LayoutFitContent, TaffyMaxContent as LayoutMaxContent, TaffyMinContent as LayoutMinContent, TaffyZero as LayoutZero,
        TaffyGridLine as LayoutGridLine, TaffyGridSpan as LayoutGridSpan,
    },
    CacheTree as CacheLayoutTree,
    tree::{
        LayoutPartialTree, PrintTree as PrintLayoutTree, RoundTree as RoundLayoutTree, TraversePartialTree as TraverseLayoutPartialTree, TraverseTree as TraverseLayoutTree,
        LayoutFlexboxContainer, LayoutGridContainer, LayoutBlockContainer,
    },
};
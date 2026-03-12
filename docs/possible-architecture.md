# Possible Architecture Overview

## Pipeline

```
   User-facing API
───────────────────────────────────────────
        |
        │  Core
        │
────────┼──────────────────────────────────
        ▼
┌───────────────┐
│     Tree      │  Id → Node { kind, children, layer_id }
|               │  Manages relationships between nodes.
└───────┬───────┘
        ▼
┌───────────────┐
│               │  Id → Layout (computed bounds, styles)
│     Layout    │  Constraint solving, measuring text
|               │  
└───────┬───────┘
        ▼
┌───────────────┐
│               │  Id → Layer { buffer, is_dirty, position }
│     Layers    │  Promoted nodes get own layer, rest inherit parent's
|               │  Inspired by the DOM and notcurses planes/piles.
└───────┬───────┘
        ▼
┌───────────────┐
│     Paint     │  Walk tree, write cells into each node's layer
│               │  Skip clean layers entirely
└───────┬───────┘
        ▼
┌───────────────┐
│   Composite   │  Flatten layers back→front into final buffer
└───────┬───────┘
        ▼
┌───────────────┐
│               │  Final buffer vs last frame
│      Diff     │  Track terminal state for color/style elision
|               │  See notcurses elision. 
└───────┬───────┘
        ▼
    Terminal
```
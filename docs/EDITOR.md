# Proofer — Real-Time Editor and Geometry Canvas

## 1. Product Goal

The editor should let the user reason in two modes without leaving one mathematical model:

```text
write proof in code
        ↕
explore proof in canvas
```

The defining feature is bidirectional synchronization.

---

## 2. Shared Document Model

Conceptually:

```rust
struct DocumentModel {
    source,
    syntax,
    semantic_graph,
    proof_graph,
    geometry_scene,
    diagnostics,
}
```

The canvas is not a second database of mathematical truth.

---

## 3. Source → Canvas

Given:

```text
triangle ABC
construct M as midpoint of BC
```

the compiler builds the semantic model and the renderer draws the figure.

---

## 4. Canvas → Source

A user action such as:

```text
Geometry Tool → Midpoint → BC
```

creates a mathematical operation.

That operation is converted to a source patch such as:

```text
construct M as midpoint of BC
```

The patch is applied through the normal compiler pipeline.

---

## 5. Selection Mapping

The mapping layer connects:

```text
source span
semantic ID
canvas ID
```

Therefore:

- clicking a point can jump to its declaration,
- clicking a proof fact can highlight its geometry,
- hovering source can highlight the corresponding figure.

---

## 6. Proof Visualization

Every proof fact may have geometry references.

Example:

```text
h3 : triangle ABM congruent triangle ACM
```

The editor can highlight both triangles and display the rule:

```text
SSS
```

alongside the premises.

---

## 7. Goal Visualization

For:

```text
show angle ABC = angle BCA
```

show the goal in both the proof panel and canvas.

The canvas can highlight the two angles and all candidate objects related to them.

---

## 8. Drag Semantics

Dragging generally changes visual state, not theorem semantics.

The renderer may continuously solve layout constraints.

If a drag produces a visual configuration inconsistent with a symbolic condition, display a warning rather than silently modifying the theorem.

---

## 9. Transactions

Every semantic canvas operation should become a document transaction.

A transaction may include:

- source edits,
- semantic updates,
- proof graph updates,
- visual updates.

Undo/redo must operate at the transaction level.

---

## 10. Incremental Updates

The editor must support stale-result rejection.

```text
revision 100
    ↓ edit
revision 101
```

A verification result produced for revision 100 must never be shown as verification for revision 101.

---

## 11. Rendering Boundary

The long-term implementation may use a Rust-native renderer such as `wgpu` plus an immediate-mode UI layer.

Keep renderer types separate from mathematical types.

```text
Point
```

is not:

```text
VisualPoint { x: f32, y: f32 }
```

---

## 12. First Vertical Slice

Before building a large UI, implement one complete loop:

```text
source:
    triangle ABC

       ↓

semantic model

       ↓

canvas draws ABC

       ↓

canvas constructs midpoint M

       ↓

source patch generated

       ↓

compiler reparses

       ↓

canvas + proof state update
```

This vertical slice is more valuable than implementing dozens of disconnected UI tools.

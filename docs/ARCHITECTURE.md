# Proofer — System Architecture

## 1. Core Principle

Proofer is one mathematical system with multiple interfaces.

```text
                 Mathematical Model
                  /       |        \
                 /        |         \
            Source      Proof      Geometry
             View        View         View
                 \        |         /
                  \       |        /
                    Proof Kernel
```

The text editor and geometry canvas must never become two independent sources of truth.

---

## 2. Compiler Pipeline

```text
source text
   ↓
lexer
   ↓
CST / AST
   ↓
resolver
   ↓
HIR
   ↓
elaborator
   ↓
proof object
   ↓
kernel
   ↓
verified theorem
```

Geometry follows:

```text
HIR
 ↓
Geometry IR
 ↓
Rule engine / solver
 ↓
Proof certificate
 ↓
Kernel
```

---

## 3. Representation Layers

### CST

Preserves syntax needed by tooling:

- source ranges
- comments
- formatting information where practical
- token identity

### AST

Represents language syntax.

### HIR / Mathematical IR

Represents mathematical meaning independently of exact syntax.

### Kernel IR

Represents only objects needed for trusted verification.

---

## 4. Stable IDs

Every persistent semantic object gets an ID:

```rust
struct FileId(u32);
struct SymbolId(u32);
struct TheoremId(u32);
struct FactId(u32);
struct PointId(u32);
struct SegmentId(u32);
struct CircleId(u32);
struct ConstructionId(u32);
```

IDs must survive source edits whenever the underlying entity survives.

---

## 5. Source Spans

Use:

```rust
struct Span {
    file: FileId,
    start: u32,
    end: u32,
}
```

Diagnostics should use spans rather than manually computed line numbers stored on AST nodes.

Line/column display is derived from source text.

---

## 6. Proof Graph

Proofs should be represented as a DAG, not merely as an ordered list.

```text
h1      h2
 \      /
  \    /
   rule
     |
     h3
```

Each fact stores:

- proposition
- proof node
- dependencies
- source span
- optional geometry references

This enables navigation, incremental verification, and visualization.

---

## 7. Trust Boundary

Untrusted:

- parser
- resolver
- elaborator
- geometry solver
- automation
- renderer
- editor
- future AI assistant

Trusted:

- proof kernel

The trusted computing base must stay deliberately small.

---

## 8. Geometry and Visual State

Geometry has two separate representations.

### Mathematical state

```text
AB = AC
M midpoint of BC
AB perpendicular BC
```

### Visual state

```text
A = (100.4, 210.8)
B = (50.2, 400.1)
C = (500.0, 401.0)
```

Visual state exists for rendering and interaction.

Visual coordinates are never evidence for a theorem.

---

## 9. Editor Synchronization

Text edit:

```text
text edit
 ↓
parse affected region
 ↓
elaborate affected nodes
 ↓
update shared model
 ↓
update canvas
```

Canvas action:

```text
canvas gesture
 ↓
semantic operation
 ↓
source patch
 ↓
parse/elaborate
 ↓
update shared model
```

Never mutate an isolated hidden canvas model and try to reverse-engineer source afterward.

---

## 10. Revisions

Every compilation state carries a revision number.

```rust
struct Revision(u64);
```

Asynchronous verification results must record their revision.

The UI must discard results for stale revisions.

---

## 11. Crate Direction

Long-term crate layout:

```text
proofer-syntax
proofer-parser
proofer-hir
proofer-resolve
proofer-elab
proofer-kernel
proofer-geometry
proofer-solver
proofer-editor
proofer-protocol
proofer-cli
```

The implementation may begin in a single crate, but module boundaries should match these ownership boundaries from the beginning.

---

## 12. Important Negative Decisions

Do not:

- make the canvas the source of truth,
- make floating-point coordinates part of proof semantics,
- let automation bypass the kernel,
- make proof identity depend on array indexes,
- couple renderer types to mathematical types,
- implement dependent type theory before the first kernel works,
- build a giant tactic framework before basic proof objects are correct.

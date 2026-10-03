# Proofer — Engineering Roadmap

## 0. Purpose

Proofer is a mathematical proof language and interactive proof environment written in Rust.

The project has two defining properties:

1. Mathematical proofs are represented as machine-checkable proof objects.
2. A real-time geometry canvas and the source editor are two views over the same mathematical model.

The system must not become “a Lean clone with a drawing tool.” The syntax, intermediate representations, proof workflow, and editor model are designed specifically for Proofer.

---

## 1. Non-Negotiable Architecture

```text
Source
  ↓
Lexer
  ↓
CST / AST
  ↓
Name Resolution
  ↓
HIR / Mathematical IR
  ↓
Elaboration
  ↓
Proof Object
  ↓
Small Kernel
  ↓
Verified
```

Geometry follows the same trust boundary:

```text
Geometry Source / Canvas
          ↓
     Geometry IR
          ↓
  Solver / Rule Engine
          ↓
  Proof Certificate
          ↓
      Small Kernel
```

The renderer, geometry solver, automation, and AI assistant are not trusted.

The kernel is the final authority.

---

# 2. Stage Plan

## Stage 0 — Repository and Architecture Foundation

### Goal
Restructure the prototype so future compiler work has clean ownership boundaries.

### Deliverables

- `span.rs`
- source/file database
- stable IDs
- diagnostics infrastructure
- module boundaries
- basic regression tests
- documented invariants

### Exit Criteria

- Existing lexer behavior is preserved.
- Every syntax node can carry a source span.
- Diagnostics can identify a file and source range.
- Stable IDs do not depend on vector positions or memory addresses.

### Do not build yet

- geometry UI
- theorem automation
- advanced type theory
- full kernel

---

## Stage 1 — Complete the Frontend

### Goal
Turn the parser prototype into a real parser for Proofer’s first language version.

### Deliverables

- complete proposition grammar
- precedence handling
- terms and applications
- quantifiers
- theorem declarations
- proof-block parsing
- structured parse errors
- CST preservation where practical

### Exit Criteria

The compiler can parse representative examples covering:

- propositions
- quantified propositions
- definitions
- proof blocks
- named assumptions
- derived facts
- theorem endings

No semantic verification is required yet.

---

## Stage 2 — HIR, Scope Resolution, and Elaboration

### Goal
Separate syntax from mathematical meaning.

### Deliverables

- HIR
- symbol tables
- lexical scopes
- theorem IDs
- fact IDs
- point/object IDs
- name resolution
- elaborated proposition forms

### Exit Criteria

Two syntactically different but semantically equivalent source constructs can elaborate to the same semantic representation where intended.

The source representation remains available for diagnostics and editing.

---

## Stage 3 — Minimal Proof Kernel

### Goal
Build the smallest trustworthy verifier.

### Initial logical rules

- assumption
- implication introduction/elimination
- conjunction introduction/elimination
- disjunction introduction/elimination
- negation handling through the chosen logic profile
- universal introduction/elimination
- existential introduction/elimination
- equality reflexivity
- equality substitution

### Exit Criteria

The kernel can independently verify proof objects without invoking the parser, editor, or solver.

Kernel tests must include intentionally malformed proof objects.

---

## Stage 4 — Proofer Proof Language

### Goal
Make mathematical proof writing pleasant and explicit.

### Core commands

```text
 take
 suppose
 let
 have
 construct
 show
 derive
 use
 therefore
 cases
 choose
 contradict
```

### Exit Criteria

Users can write elementary logical proofs from source code and obtain verified proof objects.

---

## Stage 5 — Geometry Mathematical Core

### Goal
Introduce Euclidean geometry as a first-class mathematical domain.

### Objects

```text
Point
Line
Segment
Ray
Angle
Circle
Triangle
Quadrilateral
```

### Relations

```text
on
between
collinear
parallel
perpendicular
equal
congruent
similar
tangent
cyclic
midpoint
bisects
```

### Exit Criteria

Geometry statements can be represented symbolically, type-checked, and linked into proof objects.

No interactive UI is required yet.

---

## Stage 6 — Synthetic Geometry Rule Library

### Goal
Support the geometry taught in high school through explicit theorem/rule certificates.

### First rule families

#### Triangles

- angle sum
- exterior angle
- isosceles triangle theorem
- SSS
- SAS
- ASA
- AAS
- RHS
- similarity
- median / altitude / bisector properties

#### Circles

- central angle
- inscribed angle
- equal chords
- tangent-radius perpendicularity
- tangent/chord relationships
- cyclic quadrilateral properties

#### Quadrilaterals

- parallelogram
- rectangle
- square
- rhombus
- kite
- trapezium
- diagonal properties

### Exit Criteria

At least several textbook-style geometry proofs can be kernel-verified end-to-end.

---

## Stage 7 — Geometry Solver and Proof Certificates

### Goal
Allow Proofer to discover proof steps without making the solver trusted.

### Capabilities

- backward goal decomposition
- forward fact propagation
- matching congruence/similarity rules
- angle chasing
- simple equality propagation
- construction suggestions

### Rule

```text
Solver result != proof result
```

The solver must emit a proof certificate which the kernel checks.

### Exit Criteria

The solver can automatically discover selected geometry proofs and the kernel independently verifies the generated certificate.

---

## Stage 8 — Real-Time Editor and Geometry Canvas

### Goal
Build the defining Proofer experience.

### Requirements

- source editor
- proof-state panel
- geometry canvas
- stable semantic IDs
- source ↔ semantic mapping
- semantic ↔ canvas mapping
- canvas actions generating source edits
- source edits updating canvas
- undo/redo as document transactions
- goal highlighting
- proof dependency highlighting

### Exit Criteria

A user can:

1. write geometry source,
2. see the figure appear live,
3. manipulate the figure,
4. create constructions from the canvas,
5. see corresponding source edits,
6. write proof steps,
7. see dependencies highlighted in the figure,
8. receive live kernel verification results.

---

## Stage 9 — Incremental Compilation and Performance

### Goal
Make the interactive environment scale beyond toy proofs.

### Deliverables

- incremental lexing
- incremental parsing
- dependency invalidation
- incremental elaboration
- proof-state caching
- cancellation of stale work
- revision-aware diagnostics

### Exit Criteria

Editing a local region does not force a full rebuild when unaffected declarations can be reused.

---

## Stage 10 — Automation and User Extensions

### Goal
Allow sophisticated proof assistance while preserving kernel trust.

Potential features:

- rewrite
- simplification
- algebraic normalization
- search
- geometry automation
- user-defined rules
- macros
- theorem libraries

Every automated result must terminate in a kernel-checkable proof object.

---

## Stage 11 — Productization

### Deliverables

- CLI
- REPL / interactive mode
- project files
- imports and modules
- `.proof` files
- standard theorem library
- editor packaging
- persistent geometry layouts
- documentation
- examples and benchmark suite

---

# 3. Dependency Graph

```text
Stage 0
   ↓
Stage 1
   ↓
Stage 2
   ↓
Stage 3
   ↓
Stage 4
   ↓
Stage 5
   ↓
Stage 6
   ↓
Stage 7
   ↓
Stage 8
   ↓
Stage 9
   ↓
Stage 10
   ↓
Stage 11
```

Stage 8 should not be started early just because it is visually exciting. The editor depends on stable semantic identity, proof state, and geometry IR.

---

# 4. Definition of Done for Every Stage

Every stage must include:

1. implementation,
2. unit tests,
3. integration tests,
4. negative/error tests,
5. documentation updates,
6. a recorded list of architectural decisions,
7. an explicit statement of what remains intentionally unimplemented.

A stage is complete only when the exit criteria are satisfied.


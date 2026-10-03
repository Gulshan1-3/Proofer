# Proofer — Language Design Specification

## 1. Language Identity

Proofer is a proof language designed for mathematical reasoning.

Its syntax is intentionally distinct from Lean, Coq, and tactic-heavy theorem provers.

The language should read like structured mathematics while remaining precise enough for compilation and kernel verification.

---

## 2. Program Structure

```text
file
 ├── imports
 ├── definitions
 ├── figures
 ├── theorems
 └── rules
```

---

## 3. Core Mathematical Vocabulary

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
choose
cases
contradict
```

---

## 4. Logical Core

Initial proposition forms:

```text
P and Q
P or Q
P -> Q
not P
P <-> Q
forall x : T, P
exists x : T, P
P = Q
```

Unicode notation may also be supported:

```text
∧ ∨ → ¬ ↔ ∀ ∃
```

ASCII and Unicode should map to the same AST/HIR nodes.

---

## 5. Logic Profile

Proofer should make the chosen logic explicit.

Example:

```text
logic classical
```

or:

```text
logic constructive
```

A theorem requiring classical principles must not silently obtain them from the implementation.

---

## 6. Example

```proofer
theorem identity:
    forall P : Prop, P -> P

proof
    take P : Prop
    suppose h : P
    therefore P from h
end
```

---

## 7. Geometry Declarations

```proofer
figure triangle_example
    triangle ABC
    given AB = AC
end
```

A figure declaration creates semantic objects and constraints, not pixels.

---

## 8. Geometry Constructions

```proofer
construct M as midpoint of BC
construct AM
construct line AD perpendicular_to BC through A
```

The compiler elaborates constructions into symbolic geometry facts.

---

## 9. Derived Facts

```proofer
have h1 : BM = MC from midpoint(M, BC)

derive h2 : triangle ABM congruent triangle ACM
    from AB = AC, h1, AM = AM
    using SSS
```

The syntax is surface language only. The kernel receives a proof object.

---

## 10. Goals

```proofer
show angle ABC = angle BCA
```

The current proof state contains:

```text
Context
Facts
Goal
```

---

## 11. Geometry Types

Initial domain types:

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

Potential future additions:

```text
Arc
Polygon
Plane
Sphere
3DPoint
```

---

## 12. Naming and Identity

Source names are resolved into stable internal IDs.

```text
A -> PointId
h1 -> FactId
triangle ABC -> TriangleId
```

Renaming syntax should update references semantically where tooling supports it.

---

## 13. Error Philosophy

Diagnostics should explain mathematical failures, not merely parser failures.

Example:

```text
cannot apply SSS

first triangle has:
    AB
    BM
    AM

second triangle has:
    AC
    CM
    AM

missing equality:
    BM = CM
```

The editor may attach the same diagnostic visually to the affected geometric objects.

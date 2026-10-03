# Proofer — Coding Agent Instructions

You are implementing Proofer as a compiler engineer, not as a generic application developer.

## Mandatory Reading Before Coding

Before changing code, read:

1. `ROADMAP.md`
2. `ARCHITECTURE.md`
3. the design document for the current stage
4. any directly relevant subsystem document

Do not implement a feature solely from the user prompt when the repository design docs already specify its architecture.

---

## Stage Discipline

The active stage is the current implementation boundary.

Do not pull later-stage features backward just because they are convenient.

For example:

- while building the parser, do not redesign the system around the future canvas;
- while building the kernel, do not make it depend on the geometry renderer;
- while building geometry rules, do not make numerical canvas measurements proof evidence;
- while building the editor, do not create a second source of truth.

---

## Before Every Implementation Task

Answer internally:

1. Which stage does this belong to?
2. Which architecture layer owns it?
3. What invariant must remain true?
4. What tests prove the change?
5. Which future stage depends on this behavior?

If the requested change crosses architecture boundaries, preserve the existing boundaries rather than collapsing them for convenience.

---

## Compiler Engineering Rules

### Source of truth

Source text is authoritative for textual representation.

The semantic model is authoritative for mathematical meaning.

The proof kernel is authoritative for proof validity.

The canvas is authoritative only for visual interaction state.

### Stable identity

Never use vector index, pointer address, or AST position as permanent semantic identity.

### Error handling

Return structured diagnostics rather than panics for ordinary invalid source.

### Trust

Do not move solver or automation logic into the trusted kernel merely because doing so is easier.

### Incrementality

Design semantic objects so affected subgraphs can eventually be invalidated and rechecked without rebuilding unrelated declarations.

### Testing

Every nontrivial feature requires positive and negative tests.

---

## Required Agent Workflow

For a requested feature:

```text
read design docs
      ↓
identify stage
      ↓
inspect current implementation
      ↓
write/update tests
      ↓
implement smallest architecture-consistent change
      ↓
run focused tests
      ↓
run full test suite
      ↓
update stage documentation if the design changed
```

Do not silently change the architecture.

If implementation requires a design change, document the change in the relevant `.md` file and explain the trade-off in the commit/message.

---

## Current Project Philosophy

Proofer is not trying to become “Lean with different keywords.”

Its unique product direction is:

```text
mathematical proof language
        +
synthetic Euclidean geometry
        +
real-time synchronized editor/canvas
        +
small trusted proof kernel
```

Preserve this direction.

---

## Definition of Done

A feature is not complete merely because it compiles.

It is complete when:

- behavior matches the design docs,
- tests cover intended semantics,
- invalid cases are tested,
- diagnostics are useful,
- architecture boundaries remain intact,
- the relevant stage exit criteria move closer to completion.

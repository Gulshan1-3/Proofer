# Proofer — Stage Execution Sheets

These are the practical implementation sheets for the coding agent.

Each stage should be completed in order.

---

# Stage 0 — Foundation

## Read

- `ROADMAP.md`
- `ARCHITECTURE.md`
- `AGENTS.md`

## Implement

- `Span`
- `FileId`
- source database
- diagnostics
- stable IDs
- module layout

## Tests

- span boundaries
- unicode source spans
- diagnostic rendering
- ID stability

## Exit

Foundation APIs exist without changing proof semantics.

---

# Stage 1 — Parser

## Read

- `ROADMAP.md`
- `ARCHITECTURE.md`
- `LANGUAGE_SPEC.md`

## Implement

- proposition parser
- terms
- quantifiers
- theorem declarations
- proof statements
- precedence
- recovery

## Tests

Create golden examples for:

- simple proposition
- nested implication
- conjunction/disjunction
- quantifiers
- geometry declarations
- proof blocks
- malformed input

## Exit

A representative `.proof` file parses without hardcoded AST values.

---

# Stage 2 — HIR / Resolver

## Read

- `ARCHITECTURE.md`
- `LANGUAGE_SPEC.md`

## Implement

- symbol table
- scopes
- theorem/fact IDs
- elaborated terms
- proposition HIR
- reference resolution

## Tests

- shadowing
- unknown names
- invalid references
- stable semantic identity

## Exit

The semantic model no longer depends on the parser's implementation details.

---

# Stage 3 — Kernel

## Read

- `PROOF_KERNEL.md`
- `ARCHITECTURE.md`

## Implement

Start with the smallest useful logical rule set.

## Tests

For every rule:

- valid object,
- wrong premise,
- wrong conclusion,
- malformed object.

## Exit

Kernel can be tested without invoking parser/editor/geometry code.

---

# Stage 4 — Proof Language

## Read

- `LANGUAGE_SPEC.md`
- `PROOF_KERNEL.md`

## Implement

Map high-level commands such as `suppose`, `have`, `derive`, and `therefore` into kernel proof objects.

## Exit

Elementary source proofs are accepted only through kernel verification.

---

# Stage 5 — Geometry IR

## Read

- `GEOMETRY.md`
- `LANGUAGE_SPEC.md`
- `PROOF_KERNEL.md`

## Implement

- geometry object IDs
- relations
- constraints
- constructions
- geometry propositions

## Exit

Geometry can be represented and checked symbolically without a renderer.

---

# Stage 6 — Geometry Rules

## Read

- `GEOMETRY.md`
- `PROOF_KERNEL.md`

## Implement

Start with a narrow high-confidence rule set.

Suggested first vertical theorem:

```text
AB = AC
→ base angles are equal
```

Then expand to SSS/SAS/etc.

## Exit

At least one nontrivial school-level geometry proof reaches the kernel as a certificate.

---

# Stage 7 — Geometry Solver

## Read

- `GEOMETRY.md`
- `PROOF_KERNEL.md`

## Implement

- backward search
- forward chaining
- theorem pattern matching
- certificate generation

## Do not

Return a raw boolean as the solver's final product.

## Exit

Automatically discovered proofs are independently kernel-verified.

---

# Stage 8 — Editor / Canvas

## Read

- `EDITOR.md`
- `ARCHITECTURE.md`
- `INCREMENTAL_COMPILER.md`

## Implement first

One complete vertical slice:

```text
triangle ABC
→ render
→ construct midpoint M
→ source patch
→ recompile
→ update proof state
```

## Then

- selections
- highlighting
- proof navigation
- more construction tools
- goal visualization
- undo/redo

## Exit

Text and canvas operate on the same mathematical state.

---

# Stage 9 — Incremental Compiler

## Read

- `INCREMENTAL_COMPILER.md`
- `ARCHITECTURE.md`

## Implement

- revisions
- dependency graph
- invalidation
- cancellation
- caches

## Exit

Local source changes do not unnecessarily invalidate unrelated work.

---

# Stage 10 — Automation

## Read

- `PROOF_KERNEL.md`
- `GEOMETRY.md`

## Implement

Automation as certificate generation.

## Exit

Automation may be sophisticated, but the trust boundary remains unchanged.

---

# Stage 11 — Productization

## Read

all design docs before making cross-cutting changes.

## Implement

CLI, REPL, modules, imports, theorem library, editor packaging, documentation, examples.

## Exit

Proofer is usable as a coherent language/tool rather than a collection of compiler experiments.

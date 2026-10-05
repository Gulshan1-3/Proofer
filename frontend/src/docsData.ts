export interface DocSection {
  id: string;
  title: string;
  category: string;
  summary: string;
  keywords: string[];
  content: string;
  runnableExample?: {
    title: string;
    code: string;
    isGeometry: boolean;
  };
}

export interface DocCategory {
  name: string;
  description: string;
  sections: DocSection[];
}

export const DOC_CATEGORIES: DocCategory[] = [
  {
    name: 'Getting Started',
    description: 'Quickstart, philosophy, and installation guide.',
    sections: [
      {
        id: 'intro-philosophy',
        title: 'Introduction & Core Philosophy',
        category: 'Getting Started',
        summary: 'Why Proofer was created and how it redefines theorem proving for humans and machines.',
        keywords: ['intro', 'philosophy', 'lcf', 'overview', 'mission'],
        content: `
# Introduction & Core Philosophy

**Proofer** is an interactive theorem proving environment and mathematical instrument designed for clarity, instant verification, and bidirectional visual feedback.

### The Problem with Traditional Provers
For decades, formal proof systems such as Lean, Coq, and Isabelle have delivered unmatched logical power—at the cost of human legibility:
- **Tactic Spaghetti:** Real-world proof scripts consist of opaque sequences of imperatives (\`simp\`, \`omega\`, \`linarith\`, \`ring\`, \`rcases\`) that describe *how the solver searched*, rather than *why the mathematics is true*.
- **Opaque Proof States:** A reader inspecting a source file cannot understand the state without running a language server step-by-step.
- **Disconnected Visuals:** When reasoning about geometry, topology, or distributed systems, mathematicians and engineers sketch diagrams. In traditional systems, diagrams are entirely external to the proof.

### The Proofer Paradigm
Proofer unites three previously separated planes into a single unified source of truth:
1. **Readable Declarative Math:** Proofs read like rigorous mathematical prose: \`suppose\`, \`derive ... using Rule\`, \`therefore\`.
2. **Microsecond Trusted Kernel:** Verification checks completed in under 10 microseconds per step using pure first-order natural deduction.
3. **Synchronized Visual Feedback:** Synthetic geometry, topology, and execution traces render in real time alongside code. Dragging a point on the canvas immediately updates parametric coordinates while preserving logical invariants.
        `,
      },
      {
        id: 'quickstart-5min',
        title: '5-Minute Quickstart',
        category: 'Getting Started',
        summary: 'Write and verify your first geometric theorem in 5 minutes.',
        keywords: ['quickstart', 'tutorial', 'first proof', 'thales'],
        runnableExample: {
          title: "Thales' Semicircle Theorem",
          code: `theorem thales_semicircle_right_angle:
    diameter(AB) and on_circle(C) -> angle_ACB = 90
proof
    suppose h1 : diameter(AB) and on_circle(C)
    derive h2 : angle_ACB = 90 from h1 using InscribedAngle
    therefore angle_ACB = 90 from h2
end`,
          isGeometry: true,
        },
        content: `
# 5-Minute Quickstart Tutorial

Let's prove a classical result in synthetic geometry: **Thales' Theorem**, which states that any triangle inscribed in a semicircle with the diameter as its hypotenuse has a right angle ($90^\\circ$) at the vertex on the circle.

### 1. The Theorem Declaration
In Proofer, theorems begin with a goal proposition:
\`\`\`proofer
theorem thales_semicircle_right_angle:
    diameter(AB) and on_circle(C) -> angle_ACB = 90
\`\`\`

### 2. The Proof Block
The proof proceeds using declarative natural deduction steps:
- \`suppose h1 : ...\` introduces the hypothesis into the local context.
- \`derive h2 : ... using InscribedAngle\` applies a verified geometric certificate.
- \`therefore ... from h2\` matches the root goal and closes the proof.

\`\`\`proofer
proof
    suppose h1 : diameter(AB) and on_circle(C)
    derive h2 : angle_ACB = 90 from h1 using InscribedAngle
    therefore angle_ACB = 90 from h2
end
\`\`\`

### 3. Immediate Visual Verification
When you enter this proof in the Proofer workstation, the geometry solver identifies points $A$, $B$, and $C$, positions the circle with diameter $AB$, and renders the inscribed triangle with the right angle symbol automatically!
        `,
      },
      {
        id: 'installation',
        title: 'Installation & Tooling',
        category: 'Getting Started',
        summary: 'Install the VS Code extension, the Rust CLI, or run via Docker.',
        keywords: ['install', 'vscode', 'cli', 'docker', 'cargo', 'setup'],
        content: `
# Installation & Setup

Proofer provides multiple distribution formats for local development, IDE workflows, and containerized deployment.

### 1. VS Code Extension (Recommended)
The official Proofer extension includes the full 3-pane interactive workstation, timeline scrubber, and language server support.

\`\`\`bash
# Install the packaged VSIX extension
code --install-extension proofer-vscode-0.1.0.vsix
\`\`\`

Features:
- Syntax highlighting for \`.proof\` files.
- Command: \`Proofer: Open Verification Workstation\` (<kbd>Ctrl+Shift+P</kbd>).
- Live hypothesis state inspector on cursor hover.

### 2. Native Rust CLI
Install the high-performance CLI compiler and LSP server directly from source using Cargo:

\`\`\`bash
# Build and install the proofer binary
cargo install --path proof

# Verify installation
proofer --help
\`\`\`

### 3. Running with Docker
Run the complete web workstation and headless verification daemon in a container:

\`\`\`bash
docker run -d -p 8086:8086 --name proofer-app proofer:latest
\`\`\`
Then visit \`http://localhost:8086\` in any web browser.
        `,
      },
    ],
  },
  {
    name: 'Language Reference',
    description: 'Syntax, grammar, logical connectives, and proof steps.',
    sections: [
      {
        id: 'lang-structure',
        title: 'Program Structure & Grammar',
        category: 'Language Reference',
        summary: 'Anatomy of a .proof file: imports, definitions, theorems, and proof blocks.',
        keywords: ['grammar', 'syntax', 'file structure', 'tokens', 'ast'],
        content: `
# Program Structure & Grammar

A Proofer source file (\`.proof\`) consists of modular sections:

\`\`\`text
file
 ├── imports       (import geometry.circles)
 ├── definitions   (define right_triangle(A, B, C) := ...)
 ├── figures       (figure triangle_abc { points: A, B, C })
 ├── theorems      (theorem name : proposition proof ... end)
 └── rules         (rule SSS : premises -> conclusion)
\`\`\`

### Declarative Vocabulary
Proofer uses a fixed mathematical vocabulary designed to match natural mathematical literature:

| Keyword | Role | Example |
| :--- | :--- | :--- |
| \`theorem\` | Declares a named goal proposition | \`theorem pythagoras : ...\` |
| \`suppose\` | Introduces an assumption | \`suppose h1 : AB = AC\` |
| \`derive\` | Inters an intermediate fact via a rule | \`derive h2 : angle_B = angle_C using Isosceles\` |
| \`therefore\`| Closes the proof with conclusion | \`therefore angle_B = angle_C from h2\` |
| \`take\` | Universal quantifier introduction | \`take P : Prop\` |
| \`cases\` | Disjunction elimination | \`cases h1 : P or Q\` |
| \`contradict\`| Negation / contradiction elimination | \`contradict h_false\` |
        `,
      },
      {
        id: 'lang-logic',
        title: 'Propositional & First-Order Logic',
        category: 'Language Reference',
        summary: 'Connectives, quantifiers, equality relations, and Unicode operators.',
        keywords: ['logic', 'connectives', 'quantifiers', 'and', 'or', 'implies', 'forall', 'exists'],
        runnableExample: {
          title: "Modus Ponens Identity",
          code: `theorem modus_ponens_identity:
    P and (P -> Q) -> Q
proof
    suppose h1 : P and (P -> Q)
    derive h2 : Q from h1 using ModusPonens
    therefore Q from h2
end`,
          isGeometry: false,
        },
        content: `
# Propositional & First-Order Logic

Proofer natively supports standard first-order logic propositions. Both ASCII and Unicode notations are supported and map to identical kernel AST representations.

### Logical Connectives
| Operation | ASCII Syntax | Unicode | Meaning |
| :--- | :--- | :--- | :--- |
| Conjunction | \`P and Q\` | \`P ∧ Q\` | Both $P$ and $Q$ hold |
| Disjunction | \`P or Q\` | \`P ∨ Q\` | Either $P$ or $Q$ (or both) |
| Implication | \`P -> Q\` | \`P → Q\` | If $P$ then $Q$ |
| Negation | \`not P\` | \`¬P\` | $P$ is false |
| Equivalence | \`P <-> Q\` | \`P ↔ Q\` | $P$ if and only if $Q$ |
| Equality | \`t1 = t2\` | \`t1 = t2\` | Structural term equality |

### First-Order Quantifiers
- **Universal:** \`forall x : T, P(x)\` or \`∀x:T. P(x)\`
- **Existential:** \`exists x : T, P(x)\` or \`∃x:T. P(x)\`

### Explicit Logic Profiles
Proofer requires the proof environment to declare whether non-constructive reasoning is permitted:
\`\`\`proofer
logic classical      // Allows Law of Excluded Middle and Double Negation Elimination
// or
logic constructive   // Intuitionistic logic only
\`\`\`
        `,
      },
    ],
  },
  {
    name: 'Trusted Kernel',
    description: 'The microsecond trusted logical core and LCF-style architecture.',
    sections: [
      {
        id: 'kernel-architecture',
        title: 'LCF-Style Architecture',
        category: 'Trusted Kernel',
        summary: 'How Proofer guarantees zero hallucinations and mathematical soundness.',
        keywords: ['kernel', 'lcf', 'trusted core', 'soundness', 'verifier'],
        content: `
# Trusted Proof Kernel Architecture

The central tenet of Proofer is **uncompromising logical soundness**.

### The Single Trusted Gatekeeper
The kernel (\`proof/src/kernel/checker.rs\`) answers exactly ONE question:
> **Is this proof object a valid derivation of this proposition under the active logical rules?**

Everything else in the system is untrusted:
- The parser, lexer, and CST/AST elaborators are untrusted.
- The AI auto-infill engine is untrusted.
- The geometry solver and constraint engines are untrusted.
- The IDE frontend and Monaco editors are untrusted.

If any untrusted component suggests an invalid, hallucinatory, or unsound step, the kernel immediately rejects the certificate.

\`\`\`
+--------------------------------------------------------+
| UNTRUSTED LAYER: Editor, AI Prover, Geometry Solvers  |
+--------------------------------------------------------+
                           |
             Generates Proof Object DAG
                           |
                           v
+--------------------------------------------------------+
| TRUSTED KERNEL: Natural Deduction + Certificate Checker |
| - Independent Rust module (< 800 LOC)                  |
| - Pure Functions, Zero I/O, Zero External Crates       |
| - Execution Latency: < 10 microseconds per step        |
+--------------------------------------------------------+
                           |
                     Valid / Invalid
\`\`\`
        `,
      },
      {
        id: 'kernel-rules',
        title: 'Inference Rules & Proof Objects',
        category: 'Trusted Kernel',
        summary: 'Formal definition of ProofNode DAG rules and equality substitution.',
        keywords: ['inference rules', 'proof node', 'natural deduction', 'modus ponens', 'equality'],
        content: `
# Inference Rules & Proof Objects

Proofs in Proofer are directed acyclic graphs (DAGs) of \`ProofNode\` inference steps.

### Implication Rules
- **$\\to$ Introduction (\`ImpIntro\`):** Given an assumption $P$ and a derivation of $Q$, yields $P \\to Q$.
- **$\\to$ Elimination (\`ImpElim\` / Modus Ponens):** From $P \\to Q$ and $P$, yields $Q$.

### Conjunction Rules
- **$\\land$ Introduction (\`AndIntro\`):** From $P$ and $Q$, yields $P \\land Q$.
- **$\\land$ Elimination (\`AndElimLeft\`, \`AndElimRight\`):** From $P \\land Q$, yields $P$ (or $Q$).

### Disjunction Rules
- **$\\lor$ Introduction (\`OrIntroLeft\`, \`OrIntroRight\`):** From $P$, yields $P \\lor Q$.
- **$\\lor$ Elimination (\`OrElim\`):** Case analysis over $P \\lor Q$.

### Equality Substitution (\`EqSubst\`)
From an equality $a = b$ and a proven proposition $P(a)$, the kernel derives $P(b)$ using rigorous term substitution with congruence closure.
        `,
      },
    ],
  },
  {
    name: 'Synthetic Geometry Engine',
    description: 'Euclidean geometric primitives, predicates, and verified certificates.',
    sections: [
      {
        id: 'geo-primitives',
        title: 'Geometric Primitives & Predicates',
        category: 'Synthetic Geometry Engine',
        summary: 'Points, lines, segments, circles, and relations in Proofer geometry.',
        keywords: ['geometry', 'primitives', 'points', 'circles', 'triangles', 'predicates'],
        runnableExample: {
          title: "Isosceles Base Angles",
          code: `theorem isosceles_base_angles:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles
    therefore angle_ABC = angle_ACB from h2
end`,
          isGeometry: true,
        },
        content: `
# Synthetic Geometry Primitives & Predicates

Proofer features a first-class synthetic Euclidean geometry engine capable of translating geometric predicates into visual CAD constraints and formal logical certificates.

### Built-in Predicates
- **\`diameter(AB)\`**: Segment $AB$ passes through the center of the bounding circle.
- **\`on_circle(C)\`**: Point $C$ lies on the circumference.
- **\`cyclic(ABCD)\`**: Vertices $A, B, C, D$ lie on a common circle (cyclic quadrilateral).
- **\`midpoint(M, AB)\`**: Point $M$ bisects segment $AB$ ($AM = MB$).
- **\`parallel(L1, L2)\`**: Lines $L_1$ and $L_2$ are parallel.
- **\`perpendicular(L1, L2)\`**: Lines $L_1$ and $L_2$ meet at $90^\\circ$.
- **\`tangent(L, C)\`**: Line $L$ touches circle $C$ at exactly one point.
- **\`triangle(ABC)\`**: The non-collinear triangle formed by vertices $A, B, C$.
        `,
      },
      {
        id: 'geo-certificates',
        title: 'Verified Geometric Certificates',
        category: 'Synthetic Geometry Engine',
        summary: 'How Euclidean theorems like SSS, SAS, and CyclicQuad are certified.',
        keywords: ['certificates', 'sss', 'sas', 'cyclic quad', 'inscribed angle', 'euclid'],
        runnableExample: {
          title: "Cyclic Quadrilateral Opposite Angles",
          code: `theorem cyclic_quad_opposite_angles:
    cyclic(ABCD) -> angle_DAB + angle_BCD = 180
proof
    suppose h1 : cyclic(ABCD)
    derive h2 : angle_DAB + angle_BCD = 180 from h1 using CyclicQuad
    therefore angle_DAB + angle_BCD = 180 from h2
end`,
          isGeometry: true,
        },
        content: `
# Verified Geometric Certificates

Rather than trusting a floating-point geometry solver to determine truth, Proofer requires solvers to produce **symbolic certificates** checked by the kernel:

### Core Geometric Rules
1. **\`InscribedAngle\`**: An angle subtended by an arc at the center is twice the angle subtended at the circumference; inscribed angle in a semicircle is $90^\\circ$.
2. **\`IsoscelesBaseAngles\`**: In triangle $ABC$, $AB = AC \\implies \\angle ABC = \\angle ACB$.
3. **\`SSS\`**: Congruence of triangles via three equal sides ($AB=DE \\land BC=EF \\land CA=FD$).
4. **\`SAS\`**: Congruence via two sides and the included angle.
5. **\`CyclicQuad\`**: Opposite angles of a concyclic quadrilateral sum to $180^\\circ$.
6. **\`TangentPerpendicularRadius\`**: The tangent line to a circle is perpendicular to the radius at the point of contact.
7. **\`MidpointBisects\`**: The segment connecting midpoints of two triangle sides is parallel to the third side.
        `,
      },
    ],
  },
  {
    name: 'Workstation Guide',
    description: 'Mastering the 3-pane IDE, time-travel scrubber, and VCS.',
    sections: [
      {
        id: 'workstation-layout',
        title: 'The 3-Pane Workstation Architecture',
        category: 'Workstation Guide',
        summary: 'Synchronized Code Editor, Verified Execution Trace, and Visual CAD Canvas.',
        keywords: ['workstation', '3-pane', 'ui', 'ide', 'timeline', 'canvas'],
        content: `
# The 3-Pane Workstation Architecture

The Proofer Workstation is engineered as a high-precision mathematical instrument with zero clutter.

\`\`\`
+---------------------+---------------------+---------------------+
|     PANE 1          |      PANE 2         |      PANE 3         |
|   Code Editor       |  Proof Trace &      |  Geometric CAD      |
|   (Monaco/Syntax)   |  Hypothesis State   |  Canvas Visualizer  |
|                     |  & Timeline Scrubber|  & Parametric Drag  |
+---------------------+---------------------+---------------------+
\`\`\`

### 1. Code Editor (Left)
- Live syntax highlighting and real-time error markers.
- Auto-completion for rules, predicates, and hypothesis IDs.
- Keyboard shortcuts for one-click verification (<kbd>Ctrl+Enter</kbd> / <kbd>Cmd+Enter</kbd>).

### 2. Proof Inspector & Timeline (Center)
- Displays step-by-step verified execution trace.
- **Time-Travel Scrubber:** Move the scrubber left and right to step through time.
- Inspect active hypotheses and known facts at each derivation step.

### 3. Visual CAD Canvas (Right)
- Dynamically rendered SVG/Canvas showing the exact geometric state.
- Interactive points: Drag vertices to test whether your proof holds for arbitrary non-degenerate configurations.
- Color palette: Luminous amber circles and cool sky blue triangles designed for high-contrast visibility.
        `,
      },
      {
        id: 'workstation-vcs',
        title: 'Built-in Version Control (VCS)',
        category: 'Workstation Guide',
        summary: 'Instant local branching, immutable snapshots, and proof diffing.',
        keywords: ['vcs', 'git', 'version control', 'commits', 'history', 'snapshots'],
        content: `
# Built-in Version Control System (VCS)

Mathematical discovery is non-linear. Mathematicians frequently explore multiple proof trajectories before finding the cleanest path.

Proofer includes an **in-memory, zero-latency version control engine**:
- **Automatic Snapshots:** Every successful verification automatically creates an immutable checkpoint.
- **Branching & Diffing:** Compare two proof formulations side-by-side to see which uses fewer inference steps or fewer auxiliary hypotheses.
- **Persistent Local Storage:** Workspaces survive browser reloads and offline sessions without external servers.
        `,
      },
    ],
  },
  {
    name: 'Comparison Matrix',
    description: 'Detailed analysis: Proofer vs. Lean 4 vs. Coq vs. Isabelle.',
    sections: [
      {
        id: 'comparison-lean4',
        title: 'Proofer vs. Lean 4, Coq, and Isabelle',
        category: 'Comparison Matrix',
        summary: 'Latency, syntax, visual feedback, and use-case comparison.',
        keywords: ['lean 4', 'coq', 'isabelle', 'comparison', 'benchmarks', 'speed'],
        content: `
# Detailed Comparison: Proofer vs. Traditional Provers

| Metric / Feature | Proofer | Lean 4 | Coq (ROC) | Isabelle/HOL |
| :--- | :--- | :--- | :--- | :--- |
| **Primary Domain** | Synthetic Geometry, First-Order Logic, Protocols | General Programming & Pure Math (CIC) | Constructive Calculus of Inductive Constructions | Higher-Order Logic (HOL) |
| **Verification Latency** | **< 10 µs** per step | ~10 ms – 500 ms (elaboration overhead) | ~50 ms – 1 s | ~100 ms – 2 s |
| **Visual Feedback** | **Native Synced 2D CAD Canvas** | None (Third-party or text widgets) | None | None |
| **Syntax Style** | **Declarative Prose** (\`suppose\`, \`derive\`, \`therefore\`) | Tactic-based (\`by simp, ring\`) & Functional | Tactic-based (Ltac, SSReflect) | Isar Declarative / Tactics |
| **Learning Curve** | **10 minutes** for high-school math | Months (dependent types, tactics) | Months (CIC, vernacular) | Weeks to months |
| **AI Infilling** | **Built-in verified synthesis** | External LLM plugins (LeanCopilot) | External plugins | Sledgehammer (SMT/ATP) |
| **Browser Runtime** | **Native WebAssembly (< 250 KB)** | Heavy WebAssembly (~30 MB WASM) | JsCoq (~40 MB) | JVM-dependent (No native web) |
| **Deterministic Replay** | **100% bit-for-bit reproducible** | Highly reproducible | Highly reproducible | Highly reproducible |
        `,
      },
    ],
  },
  {
    name: 'Standard Library (Mathlib)',
    description: 'Pre-certified theorems across triangles, circles, and propositional logic.',
    sections: [
      {
        id: 'mathlib-overview',
        title: 'Mathlib Geometry & Logic Modules',
        category: 'Standard Library (Mathlib)',
        summary: 'Explore the verified theorems packaged with Proofer.',
        keywords: ['mathlib', 'triangles', 'circles', 'quadrilaterals', 'library', 'standard library'],
        runnableExample: {
          title: "Parallelogram Opposite Sides",
          code: `theorem parallelogram_opposite_sides:
    parallelogram(ABCD) -> AB = CD and BC = DA
proof
    suppose h1 : parallelogram(ABCD)
    derive h2 : AB = CD and BC = DA from h1 using ParallelogramOppSides
    therefore AB = CD and BC = DA from h2
end`,
          isGeometry: true,
        },
        content: `
# Proofer Mathlib

Proofer ships with an open-source standard library of verified mathematical theorems organized into domain modules:

### 1. \`geometry.triangles\`
- \`isosceles_base_angles\`: Equal legs imply equal base angles.
- \`triangle_angle_sum\`: Sum of interior angles of a triangle equals $180^\\circ$.
- \`triangle_midpoint_theorem\`: Line segment connecting two midpoints is parallel to base.
- \`sss_congruence\`, \`sas_congruence\`: Triangle congruence criteria.

### 2. \`geometry.circles\`
- \`thales_theorem\`: Angle inscribed in a semicircle is a right angle.
- \`tangent_perpendicular_radius\`: Tangent line is perpendicular to radius at point of contact.
- \`inscribed_angle_subtended\`: Measure of inscribed angle vs. central angle.

### 3. \`geometry.quadrilaterals\`
- \`parallelogram_opposite_sides\`: Opposite sides of a parallelogram are congruent.
- \`cyclic_quad_opposite_angles\`: Opposite angles of a cyclic quad sum to $180^\\circ$.

### 4. \`logic.propositional\`
- \`modus_ponens_identity\`: $(P \\land (P \\to Q)) \\to Q$.
- \`double_negation\`: $\\neg(\\neg P) \\to P$ (Classical logic profile).
- \`de_morgan_conjunction\`: $\\neg(P \\land Q) \\leftrightarrow (\\neg P \\lor \\neg Q)$.
        `,
      },
    ],
  },
  {
    name: 'Developer Reference',
    description: 'CLI commands, LSP server, REST API daemon, and WebAssembly.',
    sections: [
      {
        id: 'dev-cli-lsp',
        title: 'CLI & Language Server Protocol (LSP)',
        category: 'Developer Reference',
        summary: 'Integrate Proofer into editors, CI/CD pipelines, and automated graders.',
        keywords: ['cli', 'lsp', 'language server', 'api', 'daemon', 'ci/cd'],
        content: `
# CLI & Language Server Protocol (LSP)

### CLI Command Reference
The \`proofer\` binary provides command-line verification for CI/CD pipelines and scripts:

\`\`\`bash
# Check a single proof file
proofer check path/to/theorem.proof

# Format a proof file with standard indentation
proofer fmt path/to/theorem.proof

# Start the JSON-RPC Language Server Protocol (LSP) over stdio
proofer lsp

# Start the HTTP/REST daemon server for the web workstation
proofer --server --port 8086
\`\`\`

### REST API Daemon Specification
When running with \`--server\`, Proofer exposes an HTTP verification endpoint:
- **Endpoint:** \`POST /api/verify\`
- **Payload:**
\`\`\`json
{
  "code": "theorem thales: ... proof ... end"
}
\`\`\`
- **Response:**
\`\`\`json
{
  "verified": true,
  "execution_time_us": 8,
  "steps": [
    { "step_index": 0, "hypothesis": "diameter(AB) and on_circle(C)", "rule": "suppose" },
    { "step_index": 1, "hypothesis": "angle_ACB = 90", "rule": "InscribedAngle" }
  ],
  "diagnostics": []
}
\`\`\`
        `,
      },
    ],
  },
];

export const ALL_DOC_SECTIONS: DocSection[] = DOC_CATEGORIES.flatMap((c) => c.sections);

export function getDocSectionById(id: string): DocSection | undefined {
  return ALL_DOC_SECTIONS.find((s) => s.id === id);
}

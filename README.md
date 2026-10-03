# Proofer: Interactive Theorem Prover and Synthetic Geometry Workstation

Proofer is a formal verification system and interactive geometry assistant written in Rust. It combines a small, trustworthy proof kernel based on natural deduction with a symbolic geometry representation and a bidirectional, reactive web workstation.

The goal of this project is to provide a complete pipeline from concrete syntax to mechanical verification, paired with an interactive visual environment where geometric diagrams and formal proof statements remain synchronized in real time.

---

## Architecture Overview

The system is structured as a layered compiler pipeline with a strict trust boundary.

```
+-----------------------------------------------------------------------+
|                           Untrusted Frontend                          |
|  Source Text -> Lexer -> Recursive Descent Parser -> File AST         |
+-----------------------------------------------------------------------+
                                    |
                                    v
+-----------------------------------------------------------------------+
|                     High-Level Intermediate (HIR)                     |
|  Symbol Resolution, Lexical Scoping, Disambiguation, SymbolId Binding  |
+-----------------------------------------------------------------------+
                                    |
                                    v
+-----------------------------------------------------------------------+
|                         Elaboration Layer                             |
|  Translates Proof Steps -> Directed Acyclic Graph (DAG) Proof Object  |
+-----------------------------------------------------------------------+
                                    |
                                    v
========================== TRUST BOUNDARY ===============================
+-----------------------------------------------------------------------+
|                        Trusted Proof Kernel                           |
|  Kernel Types (KProp, KTerm) -> 15 Logical Inference Rules            |
|  Geometric Certificate Validation (Isosceles, SSS, SAS, Congruence)   |
|  Strict Type & Equality Checking, Proof Object DAG Traversal          |
+-----------------------------------------------------------------------+
                                    |
                                    v
+-----------------------------------------------------------------------+
|                    Bidirectional Synchronization                      |
|  DocumentModel (Editor Revision <-> Geometry Scene <-> Visual SVG)    |
+-----------------------------------------------------------------------+
```

### Trust Boundary Design

Formal verification systems like Lean, Coq, and Isabelle follow the LCF architecture: all complex automation, parsing, syntactic transformation, and user interaction are untrusted. Only the core kernel is trusted.

In Proofer:
- Untrusted components:
  - Lexer and Parser: Translate plain text into AST data structures.
  - Resolver: Resolves identifiers into scoped symbols.
  - Elaborator: Translates high-level proof syntax (`suppose`, `have`, `derive`, `therefore`) into an explicit proof DAG.
  - Geometry solver: Searches for deductions through forward chaining.
  - Canvas interface: Handles user interaction and geometric rendering.
- Trusted component:
  - Proof Kernel (`src/kernel/`): Independent of AST and HIR types. It receives a `ProofObject` (an arena of `ProofNode` variants) and a typing `Context`, then traverses the inference DAG. If any rule premise or conclusion fails to match, verification is rejected.

---

## 1. How Proof Verification Works

### Formal Logic Foundation

The core logic implements first-order intuitionistic logic using Gentzen-style natural deduction. The trusted kernel defines 15 primitive inference rules:

1. Hypothesis / Assumption: Context lookup.
2. Implication Introduction: Assume $P$, derive $Q$, conclude $P \rightarrow Q$.
3. Implication Elimination (Modus Ponens): From $P \rightarrow Q$ and $P$, conclude $Q$.
4. Conjunction Introduction: From $P$ and $Q$, conclude $P \land Q$.
5. Conjunction Elimination Left: From $P \land Q$, conclude $P$.
6. Conjunction Elimination Right: From $P \land Q$, conclude $Q$.
7. Disjunction Introduction Left: From $P$, conclude $P \lor Q$.
8. Disjunction Introduction Right: From $Q$, conclude $P \lor Q$.
9. Disjunction Elimination (Proof by Cases): From $P \lor Q$, $(P \vdash R)$, and $(Q \vdash R)$, conclude $R$.
10. Negation Introduction: Assume $P$, derive $\bot$, conclude $\neg P$.
11. Negation Elimination: From $P$ and $\neg P$, conclude $\bot$.
12. Falsity Elimination (Ex Falso Quodlibet): From $\bot$, conclude any proposition $P$.
13. Universal Introduction: From $P(x)$ where $x$ is not free in active assumptions, conclude $\forall x.\; P(x)$.
14. Universal Elimination: From $\forall x.\; P(x)$, instantiate with term $t$ to obtain $P(t)$.
15. Equality Reflexivity and Substitution (Leibniz equality): From $t = t$, or from $a = b$ and $P(a)$, conclude $P(b)$.

### The Proof Object DAG

Proofs are stored as a flat arena of nodes:

```rust
pub struct ProofObject {
    pub nodes: Vec<ProofNode>,
    pub root: ProofNodeId,
    pub conclusion: KProp,
}
```

Each step references preceding node indices. The checker verifies the graph bottom-up using memoization, verifying each sub-derivation exactly once:

```rust
pub fn check_node(&mut self, id: ProofNodeId) -> Result<KProp, CheckError>
```

When an invalid step is introduced (for instance, claiming a conclusion that does not follow from the rule premise), the kernel halts and returns a structured typed error such as `ConclusionMismatch`, `InvalidRulePremise`, or `NotAnImplication`.

---

## 2. Geometric Representation and Rule Certificates

Proofer uses synthetic (axiomatic) geometry rather than purely analytic (coordinate-based) geometry. This mirrors the axiomatic system of Euclid and Hilbert.

### Symbolic Geometry Entities

The intermediate representation defines first-class geometric objects:
- Points (`PointId`)
- Lines and Segments (`SegmentId(PointId, PointId)`)
- Angles (`Angle(PointId, PointId, PointId)`)
- Triangles (`TriangleId(PointId, PointId, PointId)`)
- Geometric relations: `equal_length`, `equal_angle`, `congruent`, `midpoint_of`, `perpendicular`, `parallel`, `collinear`, `between`.

### Geometric Deduction Certificates

Geometric deductions enter the proof kernel via `ProofNode::GeoCertificate`:

```rust
ProofNode::GeoCertificate {
    rule: String,
    premises: Vec<ProofNodeId>,
    conclusion: KProp,
}
```

The kernel validates each certificate rule explicitly against strictly checked premise types:
- `IsoscelesBaseAngles`: Premise: $AB = AC$. Conclusion: $\angle ABC = \angle ACB$.
- `SSS`: Pairwise side equalities between two triangles. Conclusion: $\triangle ABC \cong \triangle DEF$.
- `SAS`: Two side equalities and included angle equality. Conclusion: $\triangle ABC \cong \triangle DEF$.
- `CPCTC` / `CongruentTrianglesAngles`: Premise: $\triangle ABC \cong \triangle DEF$. Conclusion: Corresponding parts.
- `TriangleAngleSum` / `AngleSum180`: Premise: $\triangle ABC$. Conclusion: $\angle A + \angle B + \angle C = 180^\circ$.
- `VerticalAngles`: Premise: $\text{intersect}(AB, CD)$. Conclusion: $\angle AEC = \angle BED$.
- `AlternateInteriorAngles`: Premise: $\text{parallel}(L_1, L_2)$. Conclusion: Alternate interior angle equality.
- `InscribedAngle` / `Thales`: Premise: $\text{diameter}(AB) \land \text{on\_circle}(C)$. Conclusion: $\angle ACB = 90^\circ$.
- `TangentPerpendicularRadius`: Premise: $\text{tangent}(L, O) \land \text{radius}(OT, O)$. Conclusion: $L \perp OT$.
- `CyclicQuad` / `CyclicOppositeAngles`: Premise: $\text{cyclic}(ABCD)$. Conclusion: $\angle A + \angle C = 180^\circ$.
- `ParallelogramOppSides`: Premise: $\text{parallelogram}(ABCD)$. Conclusion: $AB = CD \land BC = DA$.
- `MidpointBisects`: Premise: $\text{midpoint}(M, AB)$. Conclusion: $AM = MB$.
- `RhombusDiagonals`: Premise: $\text{rhombus}(ABCD)$. Conclusion: Diagonals are perpendicular bisectors.
- `Transitivity` / `EqTrans`: Premise: $a = b \land b = c$. Conclusion: $a = c$.
- `Symmetry` / `EqSymm`: Premise: $a = b$. Conclusion: $b = a$.

Any claim made with an unsupported rule or missing premises is immediately rejected by the trusted checker.

---

### Advanced Compiler & Verification Subsystems

1. **Incremental Compilation Engine (`proof::incremental`)**:
   - Computes AST content hashes per theorem.
   - Detects local proof edits vs. signature changes; signature changes invalidate dependent theorems while body edits re-verify only the edited theorem (cache hit latency $\sim 120\ \mu\text{s}$).
2. **Tactics Engine (`proof::tactic`)**:
   - `congruence_closure`: Discharges transitive equalities and symmetric relations automatically.
   - `angle_chase`: Linear arithmetic equation solver over geometric angle sums ($\sum = 180^\circ$).
   - `auto_geometry`: Forward-chaining geometric deduction solver.
3. **AI Co-Prover & Proof Synthesizer (`proof::synthesizer`)**:
   - Zero-hallucination inference synthesizer (`proof synthesize <file>`).
   - Every synthesized step is evaluated and certified by the formal kernel before being returned.
4. **Standard Library & Package Manager (`proof::package`)**:
   - Package manager CLI: `proof new <dir>` and `proof build [dir]`.
   - Manifest: `proof.toml` with module resolution and cycle detection.
   - Standard Library (`mathlib/`): 13 formally verified theorems across Triangles, Circles, Quadrilaterals, and Logic.
5. **Language Server Protocol (`proof::lsp`)**:
   - Native LSP server (`proof lsp`): auto-completion, hover type inspection, go-to-definition, and real-time error diagnostics.

---

## 3. Real-Time Editor and Canvas Synchronization

One of the central design challenges in interactive formal geometry is maintaining consistency between three representations:
1. The formal textual proof script.
2. The compiled mathematical model in the kernel.
3. The interactive graphical canvas.

### The Single Source of Truth Model

Proofer enforces the rule that the text document is the single source of truth. The canvas never maintains a divergent mathematical state.

```
User Types in Editor                     Canvas Gesture (Drag, Tool)
        |                                             |
        v                                             v
Incremental Recompile (Lex/Parse/Elab)       Synthesize Source Patch
        |                                             |
        v                                             v
Verified Kernel State & AST              Insert Text into Editor Document
        \                                            /
         \                                          /
          v                                        v
     Unified DocumentModel (Atomic Revision Increment)
                          |
                          v
         Render Vector SVG Scene & Feedback Matrix
```

### Bi-Directional Workflows

#### 1. Text to Canvas (Dynamic Equality Coloring)
As the user enters statements into the editor:
- When `AB = AC` is parsed as an assumption or hypothesis:
  - Segments $AB$ and $AC$ are grouped into an equivalence class.
  - The canvas renders both segments with identical stroke colors and matching congruence tick marks.
- When an angle equality is derived (`angle_ABC = angle_ACB`):
  - Base angles $\angle B$ and $\angle C$ dynamically receive matching highlight colors and arc fills.
- If the statement is modified to `AB = BC`, the equivalence class updates on the subsequent compilation pass, and colors adjust without requiring a page reload.

#### 2. Canvas to Text (Construction Synthesis)
When a user clicks a tool on the canvas (such as "Midpoint (M)"):
- The canvas engine does not silently inject an unverified point into memory.
- Instead, it formats a syntactic source patch:
  ```text
  construct M as midpoint of BC
  have h3 : triangle ABM congruent triangle ACM from h1, AM = AM using SSS
  ```
- The patch is applied to the document model, incrementing the document revision.
- The full compilation and verification cycle executes. If verified, the updated scene displays the newly constructed median line $AM$, midpoint point $M$, and the decomposed sub-angles $\angle BAM$ and $\angle CAM$.

#### 3. Step-by-Step Proof Graph Navigation
The bottom panel displays the formal proof steps returned directly by the elaboration pass:
- Selecting Step 1 (`suppose h1 : AB = AC`) highlights segments $AB$ and $AC$ in amber.
- Selecting Step 2 (`derive h2 : angle_ABC = angle_ACB using IsoscelesBaseAngles`) highlights base angles $\angle B$ and $\angle C$ in emerald green.
- Selecting a rejected step highlights the contradictory entities in red and displays the kernel error diagnostic.

#### 4. Parametric Triangle Deformation with Law of Cosines
The canvas allows manual input for side lengths $AB$, $AC$, and $BC$ as well as direct vertex dragging:
- When dragging the apex vertex $A$, the coordinate solver keeps $A$ on the perpendicular bisector line of $BC$ if the triangle is isosceles.
- When side values are altered, the client uses the Law of Cosines to calculate internal angles:
  $$\cos(A) = \frac{b^2 + c^2 - a^2}{2bc}, \quad \cos(B) = \frac{a^2 + c^2 - b^2}{2ac}$$
- The system enforces the triangle inequality ($a + b > c$) and angle sum ($\sum = 180^\circ$) to prevent illegal geometric deformations.

---

## 4. Verification Protocol and Daemon

The backend daemon runs an HTTP and JSON-RPC verification server on port 8086.

### Request Payload
```json
{
  "code": "theorem isosceles_base_angles:\n    AB = AC -> angle_ABC = angle_ACB\nproof\n    suppose h1 : AB = AC\n    derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles\n    therefore angle_ABC = angle_ACB from h2\nend"
}
```

### Response Payload
```json
{
  "status": "Ok",
  "verified": true,
  "theorems": [
    {
      "name": "isosceles_base_angles",
      "status": "Verified",
      "proven": "((AB = AC) -> (angle_ABC = angle_ACB))",
      "steps": [
        { "id": 1, "text": "suppose h1 : (AB = AC)", "status": "Valid", "conclusion": "(AB = AC)" },
        { "id": 2, "text": "derive h2 : (angle_ABC = angle_ACB) from h1 using IsoscelesBaseAngles", "status": "Valid", "rule": "IsoscelesBaseAngles", "conclusion": "(angle_ABC = angle_ACB)" },
        { "id": 3, "text": "therefore (angle_ABC = angle_ACB) from h2", "status": "Valid", "conclusion": "(angle_ABC = angle_ACB)" }
      ]
    }
  ],
  "geometryScene": {
    "name": "isosceles_triangle",
    "points": [
      { "id": "pt#4", "x": 300, "y": 120 },
      { "id": "pt#5", "x": 120, "y": 380 },
      { "id": "pt#6", "x": 480, "y": 380 }
    ]
  }
}
```

When an invalid proof is evaluated, the kernel returns:
```json
{
  "status": "Ok",
  "verified": false,
  "theorems": [
    {
      "name": "invalid_proof",
      "status": "Rejected",
      "errors": [
        "KernelRejected { theorem: \"invalid_proof\", error: ConclusionMismatch { ... } }"
      ],
      "steps": [...]
    }
  ]
}
```

## 5. Repository Layout

```
.
|-- .gitignore                  # Git ignore rules for build and documentation outputs
|-- README.md                   # System documentation and architecture guide
|-- Dockerfile                  # Multi-stage production container build
|-- docker-compose.yml          # Local container orchestration
|-- DEPLOYMENT_WEB.md           # Cloud deployment instructions (Vercel, Render, Koyeb)
|-- mathlib/                    # Standard Mathematical Library (Package Manager)
|   |-- proof.toml              # Package manifest
|   |-- triangles.proof         # Formally verified triangle theorems
|   |-- circles.proof           # Formally verified circle theorems
|   |-- quadrilaterals.proof    # Formally verified quadrilateral theorems
|   `-- logic.proof             # Formally verified propositional logic theorems
|-- frontend/                   # Modern React 19 + TypeScript + Vite Workstation
|   |-- src/                    # Workstation UI components, CAD canvas, debug trace
|   |-- src-tauri/              # Tauri native desktop app configuration & Rust bridge
|   |-- package.json            # Node dependencies and scripts
|   `-- vite.config.ts          # Vite build and dev server config
|-- vscode-extension/           # Native VS Code extension (proofer-vscode)
|   |-- syntaxes/               # TextMate formal language grammar (.tmLanguage.json)
|   |-- snippets/               # Scaffolding snippet templates
|   `-- src/                    # Language Client, LSP bridge, companion webview
|-- proof-wasm/                 # WebAssembly bindings for client-side zero-latency verification
|   `-- src/lib.rs              # Wasm exports for in-browser kernel checking
`-- proof/                      # Core Rust crate (verification kernel, compiler, server)
    |-- Cargo.toml              # Crate dependencies
    |-- src/
    |   |-- lib.rs              # Library entry point and module exports
    |   |-- main.rs             # CLI, REPL, LSP server, and HTTP verification daemon
    |   |-- token.rs / lexer.rs # Token types and streaming lexer
    |   |-- ast.rs / parser.rs  # Abstract Syntax Tree and error-recovering parser
    |   |-- diag/ / syntax/     # Diagnostic engine and source span tracking
    |   |-- hir/                # Scoping, symbol resolution, and HIR definitions
    |   |-- elab/               # Untrusted proof elaboration into kernel DAG
    |   |-- kernel/             # Trusted verification core (types.rs, checker.rs, proof_object.rs)
    |   |-- geometry/           # Synthetic geometry IR and relations
    |   |-- solver/             # Forward chaining deduction search
    |   |-- editor/             # Bidirectional DocumentModel and scene mapping
    |   |-- incremental/        # Incremental compilation engine and query caching
    |   |-- tactic/             # Tactic engine (congruence closure, angle chase, auto-geometry)
    |   |-- module/             # Module system, dependency DAG, and cyclic import detection
    |   |-- layout/             # Persistent geometric layout serialization
    |   |-- package.rs          # Package manager (`proof new`, `proof build`)
    |   |-- synthesizer/        # Zero-hallucination AI co-prover & deduction infilling
    |   |-- lsp/                # Language Server Protocol implementation (completion, hover, goto)
    |   `-- api.rs              # JSON API request/response processor
    `-- tests/                  # 84+ automated unit, integration, and performance tests
```

## 6. Local Installation and Running Guide

Proofer is designed as a **local-first workstation** and does not require cloud hosting. You can run it as a **Native Desktop App (Tauri)**, an **Interactive Web Workstation**, inside **VS Code**, or via the **Command Line Interface (CLI)**.

### Prerequisites

Ensure you have the following installed on your machine:
- **Rust & Cargo** (1.80+ or latest stable):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Node.js & npm** (Node 18+ or 20+ recommended):
  ```bash
  # Check versions
  node -v
  npm -v
  ```
- **Linux GUI Libraries** (for native desktop app windowing):
  - Ubuntu/Debian: `sudo apt install libwebkit2gtk-4.0-dev libgtk-3-dev libsoup2.4-dev`
  - Fedora/RHEL: `sudo dnf install webkit2gtk3-devel gtk3-devel libsoup-devel`
- *(Optional)* **VS Code** for the native theorem proving extension.

---

### Method A: Native Desktop Application (Tauri — Recommended)

Run Proofer as a standalone desktop program without any web browser, localhost port, or VS Code needed:

```bash
cd frontend
npm install
npm run tauri:dev
```
> This opens a dedicated native desktop window (`1440x900`) with direct in-process IPC to the Rust verification kernel (0ms latency, zero HTTP overhead).

To compile a standalone release executable:
```bash
cd frontend
npm run tauri:build
```

---

### Method B: Interactive Web Workstation

If you prefer running inside your web browser (Chrome, Firefox, Edge):

#### 1. Start the Rust Verification Daemon
In your first terminal, start the kernel daemon on port 8086:
```bash
cd proof
cargo run --release -- --server
```
> The daemon will start listening on `http://127.0.0.1:8086`.

#### 2. Start the Frontend Dev Server
In a second terminal, launch Vite:
```bash
cd frontend
npm install
npm run dev
```

#### 3. Open in Browser
Open your browser to:
```
http://localhost:5173
```

---

### Method C: Native VS Code Extension

For an integrated theorem-proving experience directly inside VS Code:

1. **Build and install the extension (`.vsix`)**:
   ```bash
   cd vscode-extension
   npm install
   npm run build
   npx @vscode/vsce package
   code --install-extension proofer-vscode-0.1.0.vsix
   ```

2. **Start the verification daemon**:
   ```bash
   cargo run --manifest-path proof/Cargo.toml --release -- --server 127.0.0.1:8086
   ```

3. **Open any `.proof` file**:
   Open files such as `mathlib/triangles.proof`. Use:
   - `Proofer: Open Workstation Panel` from the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`).
   - `Infill Step (Co-Prover)` to synthesize deductions with kernel guardrails.

---

### Method D: Command-Line Interface (CLI)

You can build and install the `proof` binary directly to your PATH:

```bash
cd proof
cargo install --path .
```

#### Available CLI Commands:

- **Check a single proof file**:
  ```bash
  proof check mathlib/triangles.proof
  # Or with JSON output:
  proof check --json mathlib/triangles.proof
  ```

- **Interactive REPL**:
  ```bash
  proof repl
  ```

- **Package Manager (`mathlib`)**:
  ```bash
  # Create a new proof project
  proof new my_theorems

  # Build and verify all proofs in a package
  proof build mathlib/
  ```

- **Synthesize the next proof step (AI Co-Prover)**:
  ```bash
  proof synthesize mathlib/circles.proof
  ```

---

### Running Tests

To run the complete test suite (84+ tests across formal kernel, geometry solvers, incremental compiler, adversarial soundness, and package manager):

```bash
cd proof
cargo test
```

---

## 7. Verification Invariant Summary

1. Proof safety: No mathematical fact is certified unless checked by `src/kernel/checker.rs`.
2. Clean separation: Elaboration, parser backtracking, and solver search reside strictly outside the trust boundary.
3. Deterministic execution: Verification is deterministic, free of hidden side effects, and operates purely on explicit proof objects.


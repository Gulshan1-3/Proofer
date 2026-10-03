# Proofer for Visual Studio Code

**Proofer** is a high-precision formal-proof programming language and interactive theorem-proving IDE. This extension brings the Proofer language to Visual Studio Code with native syntax highlighting, real-time kernel verification diagnostics, an interactive precision CAD geometry canvas, and a debugger-style execution trace.

---

## Features

### 1. Dual Workstation Workflows
- **Companion Workstation Panel**: Docks beside your `.proof` editor (`Proofer: Open Workstation Panel`). As you type or move your cursor across proof steps, the geometry canvas and debugger execution trace update in real time.
- **Integrated Mathematical Workstation**: Right-click any `.proof` file $\to$ **Open With...** $\to$ **Proofer Mathematical Workstation** to launch the complete 3-column workstation inside a VS Code editor tab.

### 2. Live Kernel Verification & Inline Diagnostics
- As you write proofs, the Proofer kernel checks each hypothesis, deduction step, and rule application.
- Failed derivations or syntax errors produce native inline VS Code error squiggles with diagnostic messages.
- Real-time status bar telemetry: `$(pass-filled) Proofer: Verified (3/3 steps)`.

### 3. Precision CAD Geometry Canvas
- Renders geometric constructions (triangles, semicircles, inscribed angles, altitudes, bisectors) with a technical CAD drafting grid and coordinate telemetry HUD.
- Automatically collapses into clean 2-pane view when writing pure propositional logic.

### 4. Debugger-Style Execution Trace
- Displays the target goal turnstile ($\vdash P \to Q$).
- Numbered step cards with semantic verbs (`SUPPOSE`, `DERIVE`, `CONSTRUCT`, `THEREFORE`), hypothesis tags, and inference rule attributions (`IsoscelesBaseAngles`, `Thales`, etc.).
- Clicking any step in the trace or element on the canvas jumps the VS Code editor cursor directly to that line.

### 5. First-Class Language Support
- Syntax highlighting for keywords, rules, predicates, logic connectives, hypotheses, and geometric entities.
- Auto-closing pairs and indentation rules.
- Snippet library (`thm`, `geom_tri`, `geom_circle`, `suppose`, `derive`, `therefore`, `construct`).

---

## Installation & Setup

### Install Dependencies & Build
From the `vscode-extension` directory:
```bash
npm install
npm run build
```

### Run in VS Code / Cursor Extension Host
1. Open the `vscode-extension` directory in VS Code.
2. Press `F5` to start a new **Extension Development Host** window.
3. Open any `.proof` file (or examples from `proof/examples/euclidean.proof`).
4. Click the **Proofer** icon in the editor title bar to open the Workstation Panel!

### Package as `.vsix`
```bash
npx @vscode/vsce package
code --install-extension proofer-vscode-0.1.0.vsix
```

---

## Extension Settings

| Setting | Default | Description |
| :--- | :--- | :--- |
| `proofer.serverUrl` | `http://127.0.0.1:8086` | URL to the Proofer verification daemon |
| `proofer.executablePath` | `""` | Path to custom `proof` CLI binary (falls back to workspace build) |
| `proofer.verifyOnChange` | `true` | Automatically verify proofs in real-time as you type |
| `proofer.verifyOnSave` | `true` | Trigger verification whenever the file is saved |

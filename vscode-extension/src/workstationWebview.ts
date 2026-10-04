import * as vscode from 'vscode';
import { VerificationResponse } from './kernelBridge';

export class WorkstationWebviewPanel {
  public static currentPanel: WorkstationWebviewPanel | undefined;
  private readonly _panel: vscode.WebviewPanel;
  private readonly _extensionUri: vscode.Uri;
  private _disposables: vscode.Disposable[] = [];

  public static createOrShow(extensionUri: vscode.Uri) {
    const column = vscode.window.activeTextEditor
      ? vscode.ViewColumn.Beside
      : vscode.ViewColumn.One;

    if (WorkstationWebviewPanel.currentPanel) {
      WorkstationWebviewPanel.currentPanel._panel.reveal(column);
      return WorkstationWebviewPanel.currentPanel;
    }

    const panel = vscode.window.createWebviewPanel(
      'prooferWorkstation',
      'Proofer Workstation: Canvas & Trace',
      column,
      {
        enableScripts: true,
        retainContextWhenHidden: true,
        localResourceRoots: [extensionUri],
      }
    );

    WorkstationWebviewPanel.currentPanel = new WorkstationWebviewPanel(panel, extensionUri);
    return WorkstationWebviewPanel.currentPanel;
  }

  private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
    this._panel = panel;
    this._extensionUri = extensionUri;

    this._panel.webview.html = this._getHtmlForWebview();

    this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

    this._panel.webview.onDidReceiveMessage(
      message => {
        switch (message.type) {
          case 'jumpToLine': {
            const editor = vscode.window.activeTextEditor;
            if (editor && typeof message.line === 'number') {
              const line = Math.max(0, message.line - 1);
              const position = new vscode.Position(line, 0);
              editor.selection = new vscode.Selection(position, position);
              editor.revealRange(new vscode.Range(position, position), vscode.TextEditorRevealType.InCenter);
            }
            break;
          }
          case 'reverify': {
            vscode.commands.executeCommand('proofer.verifyProof');
            break;
          }
          case 'synthesizeStep': {
            vscode.commands.executeCommand('proofer.synthesizeStep');
            break;
          }
        }
      },
      null,
      this._disposables
    );
  }

  public updateState(code: string, res: VerificationResponse, activeLine: number) {
    this._panel.webview.postMessage({
      type: 'update',
      code,
      verification: res,
      activeLine,
    });
  }

  public dispose() {
    WorkstationWebviewPanel.currentPanel = undefined;
    this._panel.dispose();
    while (this._disposables.length) {
      const x = this._disposables.pop();
      if (x) x.dispose();
    }
  }

  private _getHtmlForWebview(): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline'; img-src data: https:;">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>Proofer Workstation</title>
  <style>
    :root {
      --bg-base: #080b11;
      --bg-surface: #0e131f;
      --bg-subtle: #141c2e;
      --border-base: #1c2638;
      --border-focus: #24324a;
      --text-main: #f1f5f9;
      --text-muted: #8492a6;
      --text-dim: #475569;
      --accent-blue: #38bdf8;
      --accent-green: #10b981;
      --accent-amber: #f59e0b;
      --accent-red: #ef4444;
      --font-mono: 'JetBrains Mono', 'Fira Code', Menlo, Consolas, monospace;
    }

    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: var(--bg-base);
      color: var(--text-main);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      height: 100vh;
      overflow: hidden;
      display: flex;
      flex-direction: column;
    }

    header {
      background: var(--bg-surface);
      border-bottom: 1px solid var(--border-base);
      padding: 8px 14px;
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-size: 11px;
    }
    .hud-title {
      font-family: var(--font-mono);
      font-weight: 700;
      letter-spacing: 0.5px;
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .status-badge {
      font-family: var(--font-mono);
      font-size: 10px;
      font-weight: 700;
      padding: 2px 7px;
      border-radius: 3px;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }
    .status-verified {
      background: rgba(16, 185, 129, 0.15);
      color: var(--accent-green);
      border: 1px solid rgba(16, 185, 129, 0.3);
    }
    .status-rejected {
      background: rgba(239, 68, 68, 0.15);
      color: var(--accent-red);
      border: 1px solid rgba(239, 68, 68, 0.3);
    }

    .main-container {
      flex: 1;
      display: grid;
      grid-template-rows: 1fr 1fr;
      overflow: hidden;
    }

    /* Top: Geometry Canvas */
    .canvas-pane {
      background: var(--bg-base);
      border-bottom: 1px solid var(--border-base);
      position: relative;
      overflow: hidden;
      display: flex;
      flex-direction: column;
    }
    .pane-header {
      padding: 6px 12px;
      background: var(--bg-surface);
      border-bottom: 1px solid var(--border-base);
      font-size: 10px;
      font-family: var(--font-mono);
      font-weight: 600;
      color: var(--text-muted);
      text-transform: uppercase;
      letter-spacing: 0.5px;
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    .canvas-wrap {
      flex: 1;
      position: relative;
      display: flex;
      align-items: center;
      justify-content: center;
    }
    svg.cad-grid {
      width: 100%;
      height: 100%;
    }
    .telemetry-hud {
      position: absolute;
      bottom: 8px;
      left: 10px;
      font-family: var(--font-mono);
      font-size: 9px;
      color: var(--text-dim);
      background: rgba(14, 19, 31, 0.85);
      border: 1px solid var(--border-base);
      padding: 3px 6px;
      border-radius: 2px;
      pointer-events: none;
    }

    /* Bottom: Proof Trace */
    .trace-pane {
      background: var(--bg-surface);
      overflow-y: auto;
      padding: 12px;
      display: flex;
      flex-direction: column;
      gap: 8px;
    }
    .goal-banner {
      background: var(--bg-subtle);
      border: 1px solid var(--border-base);
      border-left: 3px solid var(--accent-blue);
      padding: 8px 12px;
      border-radius: 4px;
      font-family: var(--font-mono);
      font-size: 11px;
      color: var(--text-main);
      display: flex;
      align-items: center;
      justify-content: space-between;
    }
    .trace-item {
      background: var(--bg-base);
      border: 1px solid var(--border-base);
      border-radius: 4px;
      padding: 8px 10px;
      font-family: var(--font-mono);
      font-size: 11px;
      cursor: pointer;
      transition: border-color 0.15s, background 0.15s;
      display: flex;
      flex-direction: column;
      gap: 4px;
    }
    .trace-item:hover {
      border-color: var(--border-focus);
      background: var(--bg-subtle);
    }
    .trace-item.active {
      border-color: var(--accent-blue);
      background: rgba(56, 189, 248, 0.05);
    }
    .trace-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
    }
    .step-counter {
      color: var(--text-dim);
      font-size: 10px;
      font-weight: 700;
    }
    .verb-badge {
      font-size: 9px;
      font-weight: 700;
      padding: 1px 5px;
      border-radius: 2px;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }
    .verb-suppose { background: #1e293b; color: #94a3b8; }
    .verb-derive { background: #0c4a6e; color: #38bdf8; }
    .verb-therefore { background: #064e3b; color: #34d399; }
    .verb-construct { background: #451a03; color: #fbbf24; }

    .formula-line {
      font-size: 11px;
      color: var(--text-main);
      font-weight: 500;
    }
    .meta-line {
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 10px;
      color: var(--text-muted);
    }
    .rule-tag {
      background: var(--bg-subtle);
      border: 1px solid var(--border-base);
      padding: 1px 4px;
      border-radius: 2px;
      color: #7dd3fc;
    }

    .qed-banner {
      background: rgba(16, 185, 129, 0.1);
      border: 1px solid rgba(16, 185, 129, 0.3);
      padding: 10px;
      border-radius: 4px;
      text-align: center;
      font-family: var(--font-mono);
      font-size: 11px;
      font-weight: 700;
      color: var(--accent-green);
      letter-spacing: 1px;
      margin-top: 4px;
    }
  </style>
</head>
<body>
  <header>
    <div class="hud-title">
      <span style="color: var(--accent-blue)">▲</span>
      <span>PROOFER WORKSTATION</span>
      <span id="active-thm-name" style="color: var(--text-muted)"></span>
    </div>
    <div style="display: flex; gap: 8px; align-items: center;">
      <button id="synth-btn" style="background: rgba(56, 189, 248, 0.15); border: 1px solid rgba(56, 189, 248, 0.4); color: #38bdf8; font-family: var(--font-mono); font-size: 10px; font-weight: 600; padding: 3px 8px; border-radius: 3px; cursor: pointer;">Infill Step</button>
      <div id="status-badge" class="status-badge status-verified">KERNEL VERIFIED</div>
    </div>
  </header>

  <div class="main-container">
    <!-- Top: CAD Geometry Canvas -->
    <div class="canvas-pane">
      <div class="pane-header">
        <span>Geometry Canvas (Precision CAD)</span>
        <span id="geometry-mode">EUCLIDEAN 2D</span>
      </div>
      <div class="canvas-wrap">
        <svg id="cad-svg" class="cad-grid" viewBox="0 0 600 450">
          <defs>
            <pattern id="smallGrid" width="20" height="20" patternUnits="userSpaceOnUse">
              <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#121824" stroke-width="0.8"/>
            </pattern>
            <pattern id="grid" width="100" height="100" patternUnits="userSpaceOnUse">
              <rect width="100" height="100" fill="url(#smallGrid)"/>
              <path d="M 100 0 L 0 0 0 100" fill="none" stroke="#1c2638" stroke-width="1.2"/>
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)" />
          <g id="geometry-elements"></g>
        </svg>
        <div class="telemetry-hud" id="telemetry-hud">ORIGIN: (0, 0) | SCALE: 1.0x | HUD ACTIVE</div>
      </div>
    </div>

    <!-- Bottom: Debugger Execution Trace -->
    <div class="trace-pane" id="trace-container">
      <div class="goal-banner" id="goal-banner">
        <span>GOAL: ⊢ A → A</span>
        <span style="color: var(--accent-green)">Q.E.D.</span>
      </div>
      <div id="steps-list" style="display: flex; flex-direction: column; gap: 8px;"></div>
    </div>
  </div>

  <script>
    const vscode = acquireVsCodeApi();
    let currentActiveLine = 1;
    let currentCode = '';
    let currentVer = null;

    // Dynamic Geometry State
    let points = {
      A: { x: 300, y: 110 },
      B: { x: 140, y: 360 },
      C: { x: 460, y: 360 }
    };
    window.points = points;
    let draggingPoint = null;
    let hoveredPoint = null;

    window.addEventListener('message', event => {
      const msg = event.data;
      if (msg.type === 'update') {
        currentCode = msg.code || '';
        currentVer = msg.verification || null;
        renderState(msg.code, msg.verification, msg.activeLine);
      }
    });

    // Vector Math & Constraint Utilities
    function vecSub(a, b) { return { x: a.x - b.x, y: a.y - b.y }; }
    function vecDist(a, b) { return Math.hypot(a.x - b.x, a.y - b.y); }
    function vecDot(a, b) { return a.x * b.x + a.y * b.y; }

    function projectToCircle(raw, center, radius) {
      const d = vecSub(raw, center);
      let angle = Math.atan2(d.y, d.x);
      if (angle > 0) angle = angle > Math.PI / 2 ? -Math.PI : 0;
      return {
        x: center.x + radius * Math.cos(angle),
        y: center.y + radius * Math.sin(angle),
      };
    }

    function projectToPerpendicularBisector(rawA, ptB, ptC) {
      const midBC = { x: (ptB.x + ptC.x) / 2, y: (ptB.y + ptC.y) / 2 };
      const bc = vecSub(ptC, ptB);
      const bcLen = Math.hypot(bc.x, bc.y);
      if (bcLen < 1e-6) return rawA;
      const normal = { x: -bc.y / bcLen, y: bc.x / bcLen };
      const toRaw = vecSub(rawA, midBC);
      const dist = vecDot(toRaw, normal);
      return {
        x: midBC.x + normal.x * dist,
        y: midBC.y + normal.y * dist,
      };
    }

    function clampViewport(pt) {
      return {
        x: Math.max(30, Math.min(570, pt.x)),
        y: Math.max(30, Math.min(420, pt.y)),
      };
    }

    function createRightAngleSquare(h, base, normal, size = 11) {
      const uDx = base.x - h.x, uDy = base.y - h.y;
      const uLen = Math.hypot(uDx, uDy) || 1;
      const vDx = normal.x - h.x, vDy = normal.y - h.y;
      const vLen = Math.hypot(vDx, vDy) || 1;
      const ux = uDx / uLen, uy = uDy / uLen;
      const vx = vDx / vLen, vy = vDy / vLen;

      const p1 = { x: h.x + size * ux, y: h.y + size * uy };
      const p2 = { x: h.x + size * ux + size * vx, y: h.y + size * uy + size * vy };
      const p3 = { x: h.x + size * vx, y: h.y + size * vy };
      return \`\${p1.x},\${p1.y} \${p2.x},\${p2.y} \${p3.x},\${p3.y}\`;
    }

    function createArcPath(v, p1, p2, r = 24) {
      const a1 = Math.atan2(p1.y - v.y, p1.x - v.x);
      const a2 = Math.atan2(p2.y - v.y, p2.x - v.x);
      let diff = a2 - a1;
      while (diff < -Math.PI) diff += 2 * Math.PI;
      while (diff > Math.PI) diff -= 2 * Math.PI;
      const sweep = diff > 0 ? 1 : 0;
      const sx = v.x + r * Math.cos(a1);
      const sy = v.y + r * Math.sin(a1);
      const ex = v.x + r * Math.cos(a2);
      const ey = v.y + r * Math.sin(a2);
      const deg = Math.round(Math.abs(diff) * 180 / Math.PI);
      const midA = a1 + diff / 2;
      return {
        path: \`M \${sx} \${sy} A \${r} \${r} 0 0 \${sweep} \${ex} \${ey}\`,
        deg,
        labelPos: { x: v.x + (r + 14) * Math.cos(midA), y: v.y + (r + 14) * Math.sin(midA) }
      };
    }

    function renderState(code, ver, activeLine) {
      currentActiveLine = activeLine;
      const statusBadge = document.getElementById('status-badge');
      const thmNameEl = document.getElementById('active-thm-name');
      const stepsList = document.getElementById('steps-list');
      const goalBanner = document.getElementById('goal-banner');

      const isVerified = ver && ver.verified;
      statusBadge.className = 'status-badge ' + (isVerified ? 'status-verified' : 'status-rejected');
      statusBadge.textContent = isVerified ? 'KERNEL VERIFIED' : 'REJECTED';

      const thm = (ver && ver.theorems && ver.theorems[0]) || null;
      if (thm) {
        thmNameEl.textContent = ':: ' + thm.name;
        if (thm.proven) {
          goalBanner.innerHTML = '<span>GOAL: ⊢ ' + escapeHtml(thm.proven) + '</span><span style="color: ' + (isVerified ? 'var(--accent-green)' : 'var(--accent-red)') + '">' + (isVerified ? 'DISCHARGED' : 'OPEN') + '</span>';
        }
      }

      // Render Steps
      stepsList.innerHTML = '';
      const steps = (thm && thm.steps) || [];
      steps.forEach((step, idx) => {
        const item = document.createElement('div');
        item.className = 'trace-item' + ((idx + 2 === activeLine) ? ' active' : '');
        
        let verb = 'STEP';
        let verbClass = 'verb-derive';
        if (step.text.startsWith('suppose')) { verb = 'SUPPOSE'; verbClass = 'verb-suppose'; }
        else if (step.text.startsWith('derive')) { verb = 'DERIVE'; verbClass = 'verb-derive'; }
        else if (step.text.startsWith('therefore')) { verb = 'THEREFORE'; verbClass = 'verb-therefore'; }
        else if (step.text.startsWith('construct')) { verb = 'CONSTRUCT'; verbClass = 'verb-construct'; }

        item.innerHTML = \`
          <div class="trace-header">
            <span class="step-counter">\${String(idx + 1).padStart(2, '0')}</span>
            <span class="verb-badge \${verbClass}">\${verb}</span>
          </div>
          <div class="formula-line">\${escapeHtml(step.conclusion || step.text)}</div>
          \${step.rule ? '<div class="meta-line"><span class="rule-tag">rule: ' + escapeHtml(step.rule) + '</span></div>' : ''}
        \`;

        item.addEventListener('click', () => {
          vscode.postMessage({ type: 'jumpToLine', line: idx + 2 });
        });

        stepsList.appendChild(item);
      });

      if (isVerified && steps.length > 0) {
        const qed = document.createElement('div');
        qed.className = 'qed-banner';
        qed.textContent = '✓ Q.E.D. — THEOREM DISCHARGED BY KERNEL';
        stepsList.appendChild(qed);
      }

      // Initialize default points per theorem type if not dragging
      const isCircle = code.includes('circle') || code.includes('diameter') || code.includes('on_circle') || code.includes('thales');
      if (isCircle && points.A.y === 110) {
        points.A = { x: 140, y: 240 };
        points.B = { x: 460, y: 240 };
        points.C = { x: 260, y: 92 };
      }

      renderGeometry();
    }

    function renderGeometry() {
      window.renderGeometry = renderGeometry;
      const geomGroup = document.getElementById('geometry-elements');
      const modeLabel = document.getElementById('geometry-mode');
      const telemetryEl = document.getElementById('telemetry-hud');
      if (!geomGroup) return;

      const code = currentCode.toLowerCase();
      const isCircle = code.includes('circle') || code.includes('diameter') || code.includes('on_circle') || code.includes('thales');
      const isTriangle = code.includes('triangle') || code.includes('angle_') || code.includes('ab = ac') || code.includes('isosceles');

      modeLabel.textContent = isCircle ? 'CIRCLE (THALES)' : isTriangle ? 'TRIANGLE PLANE' : 'EUCLIDEAN 2D';
      telemetryEl.textContent = \`A(\${Math.round(points.A.x)}, \${Math.round(points.A.y)}) | B(\${Math.round(points.B.x)}, \${Math.round(points.B.y)}) | C(\${Math.round(points.C.x)}, \${Math.round(points.C.y)})\`;

      let html = '';

      if (isCircle) {
        const oX = (points.A.x + points.B.x) / 2;
        const oY = (points.A.y + points.B.y) / 2;
        const radius = vecDist(points.A, points.B) / 2;
        const thalesSq = createRightAngleSquare(points.C, points.A, points.B, 13);
        const arcA = createArcPath(points.A, points.B, points.C, 22);
        const arcB = createArcPath(points.B, points.A, points.C, 22);

        html = \`
          <!-- Circumcircle - Luminous Warm Amber -->
          <circle cx="\${oX}" cy="\${oY}" r="\${radius}" fill="rgba(245, 158, 11, 0.035)" stroke="#f59e0b" stroke-width="1.8" />
          
          <!-- Inscribed Triangle ABC Fill -->
          <polygon points="\${points.A.x},\${points.A.y} \${points.C.x},\${points.C.y} \${points.B.x},\${points.B.y}" fill="rgba(56, 189, 248, 0.06)" />

          <!-- Diameter AB - Slate Silver Baseline -->
          <line x1="\${points.A.x}" y1="\${points.A.y}" x2="\${points.B.x}" y2="\${points.B.y}" stroke="#94a3b8" stroke-width="1.8" />
          
          <!-- Inscribed Chords AC and BC - Cool Sky Blue -->
          <line x1="\${points.A.x}" y1="\${points.A.y}" x2="\${points.C.x}" y2="\${points.C.y}" stroke="#38bdf8" stroke-width="2" />
          <line x1="\${points.B.x}" y1="\${points.B.y}" x2="\${points.C.x}" y2="\${points.C.y}" stroke="#38bdf8" stroke-width="2" />
          
          <!-- Inscribed Right Angle Square at C -->
          <polyline points="\${thalesSq}" fill="rgba(56, 189, 248, 0.18)" stroke="#38bdf8" stroke-width="1.8" />
          
          <!-- 90° Badge at C -->
          <g transform="translate(\${points.C.x}, \${points.C.y - 18})">
            <rect x="-18" y="-12" width="36" height="15" rx="3" fill="#0d1117" stroke="#38bdf8" stroke-width="1" />
            <text x="0" y="-1" fill="#38bdf8" font-size="10" font-family="Inter, sans-serif" font-weight="700" text-anchor="middle">90°</text>
          </g>

          <!-- Angle Arcs at A and B -->
          <path d="\${arcA.path}" fill="none" stroke="#64748b" stroke-width="1.2" />
          <text x="\${arcA.labelPos.x}" y="\${arcA.labelPos.y}" fill="#94a3b8" font-size="9" font-family="monospace" text-anchor="middle">\${arcA.deg}°</text>
          
          <path d="\${arcB.path}" fill="none" stroke="#64748b" stroke-width="1.2" />
          <text x="\${arcB.labelPos.x}" y="\${arcB.labelPos.y}" fill="#94a3b8" font-size="9" font-family="monospace" text-anchor="middle">\${arcB.deg}°</text>

          <!-- Center O -->
          <circle cx="\${oX}" cy="\${oY}" r="3.5" fill="#f59e0b" stroke="#0a0e17" stroke-width="1" />
          <text x="\${oX}" y="\${oY + 14}" fill="#fbbf24" font-size="10" font-family="monospace" font-weight="bold" text-anchor="middle">O</text>
        \`;
      } else if (isTriangle) {
        const midBC = { x: (points.B.x + points.C.x) / 2, y: (points.B.y + points.C.y) / 2 };
        const arcB = createArcPath(points.B, points.A, points.C, 24);
        const arcC = createArcPath(points.C, points.A, points.B, 24);
        const arcA = createArcPath(points.A, points.B, points.C, 24);

        html = \`
          <!-- Triangle ABC -->
          <polygon points="\${points.A.x},\${points.A.y} \${points.B.x},\${points.B.y} \${points.C.x},\${points.C.y}" fill="rgba(56, 189, 248, 0.04)" stroke="#388bfd" stroke-width="2" />
          
          <!-- Altitude / Median Line -->
          <line x1="\${points.A.x}" y1="\${points.A.y}" x2="\${midBC.x}" y2="\${midBC.y}" stroke="#475569" stroke-width="1.5" stroke-dasharray="3 3" />
          <circle cx="\${midBC.x}" cy="\${midBC.y}" r="3.5" fill="#94a3b8" />
          <text x="\${midBC.x}" y="\${midBC.y + 15}" fill="#94a3b8" font-size="10" font-family="monospace" text-anchor="middle">M</text>

          <!-- Angle Arcs at B and C -->
          <path d="\${arcB.path}" fill="none" stroke="#58a6ff" stroke-width="1.5" />
          <text x="\${arcB.labelPos.x}" y="\${arcB.labelPos.y}" fill="#58a6ff" font-size="9" font-family="monospace" font-weight="bold" text-anchor="middle">\${arcB.deg}°</text>

          <path d="\${arcC.path}" fill="none" stroke="#58a6ff" stroke-width="1.5" />
          <text x="\${arcC.labelPos.x}" y="\${arcC.labelPos.y}" fill="#58a6ff" font-size="9" font-family="monospace" font-weight="bold" text-anchor="middle">\${arcC.deg}°</text>

          <!-- Apex Angle Arc at A -->
          <path d="\${arcA.path}" fill="none" stroke="#94a3b8" stroke-width="1.2" />
          <text x="\${arcA.labelPos.x}" y="\${arcA.labelPos.y}" fill="#94a3b8" font-size="9" font-family="monospace" text-anchor="middle">\${arcA.deg}°</text>
        \`;
      } else {
        html = \`
          <text x="300" y="225" fill="#475569" font-size="12" font-family="monospace" text-anchor="middle">
            [GEOMETRY CANVAS DORMANT — PURE PROPOSITIONAL LOGIC]
          </text>
        \`;
      }

      // Add Draggable Vertex Handles for A, B, C
      ['A', 'B', 'C'].forEach(name => {
        const pt = points[name];
        const isDragging = draggingPoint === name;
        const isHovered = hoveredPoint === name;

        html += \`
          <g id="handle-\${name}" transform="translate(\${pt.x}, \${pt.y})" style="cursor: \${isDragging ? 'grabbing' : 'grab'}">
            <!-- Invisible Touch/Click Area -->
            <circle r="18" fill="transparent" />
            <!-- Active Outer Ring -->
            \${(isDragging || isHovered) ? '<circle r="12" fill="none" stroke="#38bdf8" stroke-width="1.5" stroke-dasharray="3 2" />' : ''}
            <!-- Technical Vertex Node -->
            <circle r="\${isDragging ? 7 : 5.5}" fill="#0d1117" stroke="\${isDragging ? '#38bdf8' : '#e6edf3'}" stroke-width="2.5" />
            <circle r="2.5" fill="\${isDragging ? '#38bdf8' : '#e6edf3'}" />
            <!-- Label -->
            <g transform="translate(10, -8)">
              <rect x="-3" y="-12" width="16" height="15" rx="3" fill="#161b22" stroke="#30363d" stroke-width="0.8" />
              <text x="0" y="0" fill="#e6edf3" font-size="11" font-family="sans-serif" font-weight="600" font-style="italic">\${name}</text>
            </g>
          </g>
        \`;
      });

      geomGroup.innerHTML = html;

      // Attach Pointer Drag Handlers to Handles
      ['A', 'B', 'C'].forEach(name => {
        const handle = document.getElementById('handle-' + name);
        if (!handle) return;

        handle.addEventListener('pointerenter', () => {
          hoveredPoint = name;
          renderGeometry();
        });
        handle.addEventListener('pointerleave', () => {
          if (!draggingPoint) hoveredPoint = null;
          renderGeometry();
        });

        handle.addEventListener('pointerdown', (e) => {
          e.stopPropagation();
          draggingPoint = name;
          handle.setPointerCapture(e.pointerId);
          renderGeometry();
        });

        handle.addEventListener('pointermove', (e) => {
          if (draggingPoint !== name) return;
          const svg = document.getElementById('cad-svg');
          const ctm = svg.getScreenCTM();
          if (!ctm) return;
          const pt = svg.createSVGPoint();
          pt.x = e.clientX;
          pt.y = e.clientY;
          const raw = pt.matrixTransform(ctm.inverse());
          const clamped = clampViewport(raw);

          const code = currentCode.toLowerCase();
          const isCircle = code.includes('circle') || code.includes('diameter') || code.includes('on_circle') || code.includes('thales');
          const hasIsosceles = code.includes('ab = ac') || code.includes('isosceles');

          if (isCircle) {
            const center = { x: (points.A.x + points.B.x) / 2, y: (points.A.y + points.B.y) / 2 };
            const radius = vecDist(points.A, points.B) / 2;
            if (name === 'C') {
              points.C = projectToCircle(clamped, center, radius);
            } else {
              points[name] = clamped;
              const newCenter = { x: (points.A.x + points.B.x) / 2, y: (points.A.y + points.B.y) / 2 };
              const newRadius = vecDist(points.A, points.B) / 2;
              points.C = projectToCircle(points.C, newCenter, newRadius);
            }
          } else if (hasIsosceles) {
            if (name === 'A') {
              points.A = clampViewport(projectToPerpendicularBisector(clamped, points.B, points.C));
            } else {
              points[name] = clamped;
              points.A = clampViewport(projectToPerpendicularBisector(points.A, points.B, points.C));
            }
          } else {
            points[name] = clamped;
          }

          renderGeometry();
        });

        handle.addEventListener('pointerup', (e) => {
          if (draggingPoint === name) {
            try { handle.releasePointerCapture(e.pointerId); } catch (_) {}
            draggingPoint = null;
            renderGeometry();
          }
        });
      });

      const synthBtn = document.getElementById('synth-btn');
      if (synthBtn) {
        synthBtn.addEventListener('click', () => {
          vscode.postMessage({ type: 'synthesizeStep' });
        });
      }
    }

    function escapeHtml(str) {
      if (!str) return '';
      return String(str)
        .replace(/&/g, '&amp;')
        .replace(/</g, '&lt;')
        .replace(/>/g, '&gt;')
        .replace(/"/g, '&quot;')
        .replace(/'/g, '&#39;');
    }
  </script>
</body>
</html>`;
  }
}

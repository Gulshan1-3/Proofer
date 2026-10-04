import * as vscode from 'vscode';
import { KernelBridge, VerificationResponse } from './kernelBridge';

export class ProoferCustomEditorProvider implements vscode.CustomTextEditorProvider {
  public static readonly viewType = 'proofer.proofEditor';

  constructor(
    private readonly context: vscode.ExtensionContext,
    private readonly kernel: KernelBridge
  ) {}

  public static register(context: vscode.ExtensionContext, kernel: KernelBridge): vscode.Disposable {
    const provider = new ProoferCustomEditorProvider(context, kernel);
    return vscode.window.registerCustomEditorProvider(
      ProoferCustomEditorProvider.viewType,
      provider,
      {
        webviewOptions: {
          retainContextWhenHidden: true,
        },
      }
    );
  }

  public async resolveCustomTextEditor(
    document: vscode.TextDocument,
    webviewPanel: vscode.WebviewPanel,
    _token: vscode.CancellationToken
  ): Promise<void> {
    webviewPanel.webview.options = {
      enableScripts: true,
    };

    webviewPanel.webview.html = this.getHtmlForWebview(webviewPanel.webview);

    let debounceTimer: NodeJS.Timeout | null = null;
    const triggerVerify = async () => {
      const code = document.getText();
      const res = await this.kernel.verifyProof(code);
      webviewPanel.webview.postMessage({
        type: 'update',
        code,
        verification: res,
      });
    };

    // Initial load
    await triggerVerify();

    // Listen to document changes in VS Code
    const changeDocumentSubscription = vscode.workspace.onDidChangeTextDocument(e => {
      if (e.document.uri.toString() === document.uri.toString()) {
        if (debounceTimer) clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
          triggerVerify();
        }, 150);
      }
    });

    // Handle messages sent from the Webview (e.g. edits or commands)
    webviewPanel.webview.onDidReceiveMessage(e => {
      switch (e.type) {
        case 'edit': {
          const edit = new vscode.WorkspaceEdit();
          edit.replace(
            document.uri,
            new vscode.Range(0, 0, document.lineCount, 0),
            e.text
          );
          vscode.workspace.applyEdit(edit);
          break;
        }
      }
    });

    webviewPanel.onDidDispose(() => {
      changeDocumentSubscription.dispose();
      if (debounceTimer) clearTimeout(debounceTimer);
    });
  }

  private getHtmlForWebview(_webview: vscode.Webview): string {
    return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'unsafe-inline'; script-src 'unsafe-inline';">
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
      --accent-blue: #38bdf8;
      --accent-green: #10b981;
      --font-mono: 'JetBrains Mono', 'Fira Code', Menlo, Consolas, monospace;
    }
    * { box-sizing: border-box; margin: 0; padding: 0; }
    body {
      background: var(--bg-base);
      color: var(--text-main);
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
      height: 100vh;
      display: grid;
      grid-template-columns: 1fr 1fr 340px;
      overflow: hidden;
    }
    .panel {
      height: 100vh;
      overflow: hidden;
      display: flex;
      flex-direction: column;
      border-right: 1px solid var(--border-base);
    }
    .panel-header {
      padding: 8px 12px;
      background: var(--bg-surface);
      border-bottom: 1px solid var(--border-base);
      font-family: var(--font-mono);
      font-size: 11px;
      font-weight: 700;
      color: var(--text-muted);
      display: flex;
      justify-content: space-between;
      align-items: center;
    }
    #code-editor {
      flex: 1;
      background: var(--bg-base);
      color: var(--text-main);
      font-family: var(--font-mono);
      font-size: 13px;
      line-height: 1.6;
      padding: 12px;
      border: none;
      resize: none;
      outline: none;
      white-space: pre;
    }
    .canvas-pane {
      background: var(--bg-base);
      position: relative;
      display: flex;
      align-items: center;
      justify-content: center;
    }
    svg.cad-grid { width: 100%; height: 100%; }
    .trace-pane {
      background: var(--bg-surface);
      overflow-y: auto;
      padding: 12px;
      display: flex;
      flex-direction: column;
      gap: 8px;
    }
    .status-pill {
      font-family: var(--font-mono);
      font-size: 10px;
      font-weight: 700;
      padding: 2px 6px;
      border-radius: 3px;
    }
    .status-ok { background: rgba(16, 185, 129, 0.2); color: var(--accent-green); }
    .status-fail { background: rgba(239, 68, 68, 0.2); color: #ef4444; }
  </style>
</head>
<body>
  <!-- Column 1: Proof Script Editor -->
  <div class="panel">
    <div class="panel-header">
      <span>PROOF SCRIPT</span>
      <span id="save-status">LIVE SYNC</span>
    </div>
    <textarea id="code-editor" spellcheck="false"></textarea>
  </div>

  <!-- Column 2: CAD Geometry Canvas -->
  <div class="panel">
    <div class="panel-header">
      <span>GEOMETRY CANVAS</span>
      <span style="color: var(--accent-blue)">CAD 2D</span>
    </div>
    <div class="canvas-pane" style="flex: 1;">
      <svg id="cad-svg" class="cad-grid" viewBox="0 0 500 400">
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
        <g id="geom-layer"></g>
      </svg>
    </div>
  </div>

  <!-- Column 3: Debugger Execution Trace -->
  <div class="panel" style="border-right: none;">
    <div class="panel-header">
      <span>EXECUTION TRACE</span>
      <span id="ver-badge" class="status-pill status-ok">VERIFIED</span>
    </div>
    <div class="trace-pane" id="trace-pane"></div>
  </div>

  <script>
    const vscode = acquireVsCodeApi();
    const editor = document.getElementById('code-editor');
    const geomLayer = document.getElementById('geom-layer');
    const tracePane = document.getElementById('trace-pane');
    const verBadge = document.getElementById('ver-badge');

    let isTyping = false;
    editor.addEventListener('input', () => {
      isTyping = true;
      vscode.postMessage({ type: 'edit', text: editor.value });
    });

    window.addEventListener('message', e => {
      const msg = e.data;
      if (msg.type === 'update') {
        if (!isTyping || editor.value === '') {
          editor.value = msg.code;
        }
        isTyping = false;
        renderUpdate(msg.code, msg.verification);
      }
    });

    function renderUpdate(code, ver) {
      const isVerified = ver && ver.verified;
      verBadge.className = 'status-pill ' + (isVerified ? 'status-ok' : 'status-fail');
      verBadge.textContent = isVerified ? 'KERNEL VERIFIED' : 'REJECTED';

      // Render Steps
      tracePane.innerHTML = '';
      const thm = (ver && ver.theorems && ver.theorems[0]) || null;
      const steps = (thm && thm.steps) || [];

      steps.forEach((s, i) => {
        const d = document.createElement('div');
        d.style.padding = '8px';
        d.style.background = '#080b11';
        d.style.border = '1px solid #1c2638';
        d.style.borderRadius = '4px';
        d.style.fontFamily = 'monospace';
        d.style.fontSize = '11px';
        d.innerHTML = '<span style="color:#64748b">[' + (i + 1) + ']</span> <strong>' + escapeHtml(s.text) + '</strong>';
        tracePane.appendChild(d);
      });

      // Render Geometry
      geomLayer.innerHTML = '';
      if (code.includes('circle') || code.includes('diameter')) {
        geomLayer.innerHTML = \`
          <circle cx="250" cy="200" r="120" fill="none" stroke="#24324a" stroke-width="1.5" stroke-dasharray="3 3"/>
          <line x1="130" y1="200" x2="370" y2="200" stroke="#475569" stroke-width="1.5"/>
          <line x1="130" y1="200" x2="220" y2="90" stroke="#38bdf8" stroke-width="1.5"/>
          <line x1="220" y1="90" x2="370" y2="200" stroke="#38bdf8" stroke-width="1.5"/>
          <circle cx="130" cy="200" r="4" fill="#f1f5f9"/>
          <circle cx="370" cy="200" r="4" fill="#f1f5f9"/>
          <circle cx="220" cy="90" r="4" fill="#38bdf8"/>
        \`;
      } else if (code.includes('triangle') || code.includes('AB = AC')) {
        geomLayer.innerHTML = \`
          <polygon points="250,80 120,290 380,290" fill="rgba(56,189,248,0.05)" stroke="#38bdf8" stroke-width="2"/>
          <circle cx="250" cy="80" r="4" fill="#f1f5f9"/>
          <circle cx="120" cy="290" r="4" fill="#f1f5f9"/>
          <circle cx="380" cy="290" r="4" fill="#f1f5f9"/>
        \`;
      }
    }

    function escapeHtml(s) {
      return String(s || '')
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

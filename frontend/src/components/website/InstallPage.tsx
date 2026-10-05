import React, { useState } from 'react';
import { 
  Terminal, 
  Layers, 
  Copy, 
  Check, 
  Play, 
  Globe, 
  Cpu
} from 'lucide-react';
import { WebsiteView } from './Navbar';

interface InstallPageProps {
  onNavigate: (view: WebsiteView) => void;
}

export const InstallPage: React.FC<InstallPageProps> = ({ onNavigate }) => {
  const [copiedId, setCopiedId] = useState<string | null>(null);

  const handleCopy = (cmd: string, id: string) => {
    navigator.clipboard.writeText(cmd);
    setCopiedId(id);
    setTimeout(() => setCopiedId(null), 2000);
  };

  return (
    <div className="install-page">
      {/* Header */}
      <section className="install-hero">
        <div className="section-container">
          <div className="install-badge">Multi-Platform Distribution</div>
          <h1 className="install-hero-title">Install Proofer</h1>
          <p className="install-hero-lead">
            Get started with Proofer in your favorite workflow: inside VS Code, as a high-performance native CLI, in a Docker container, or directly in your web browser.
          </p>
        </div>
      </section>

      {/* Main Options Grid */}
      <section className="install-options-section">
        <div className="section-container">
          <div className="install-cards-grid">
            {/* Option 1: VS Code Extension */}
            <div className="install-card primary-card">
              <div className="card-top">
                <div className="install-icon-box">
                  <Layers size={24} className="text-blue" />
                </div>
                <div className="recommend-tag">Recommended for Developers</div>
              </div>
              <h2 className="install-card-title">VS Code Extension</h2>
              <p className="install-card-desc">
                The official extension brings the complete 3-pane interactive workstation, step-by-step timeline scrubber, and dynamic CAD canvas straight into Visual Studio Code.
              </p>

              <div className="install-code-box">
                <div className="code-box-label">Command Line Installation</div>
                <div className="code-line-wrapper">
                  <code>code --install-extension proofer-vscode-0.1.0.vsix</code>
                  <button 
                    className="copy-btn"
                    onClick={() => handleCopy('code --install-extension proofer-vscode-0.1.0.vsix', 'vscode')}
                  >
                    {copiedId === 'vscode' ? <Check size={14} className="text-emerald" /> : <Copy size={14} />}
                  </button>
                </div>
              </div>

              <div className="card-features-list">
                <div className="feat-item">✓ Full 3-pane Webview Workstation</div>
                <div className="feat-item">✓ Language Server Protocol (LSP) diagnostics</div>
                <div className="feat-item">✓ Live cursor hover hypothesis inspector</div>
              </div>
            </div>

            {/* Option 2: Web Browser (Instant) */}
            <div className="install-card">
              <div className="card-top">
                <div className="install-icon-box">
                  <Globe size={24} className="text-emerald" />
                </div>
                <div className="instant-tag">Zero Installation Required</div>
              </div>
              <h2 className="install-card-title">Web Workstation (WASM)</h2>
              <p className="install-card-desc">
                Run Proofer entirely inside your browser using client-side WebAssembly. No toolchains, compilers, or local setup required.
              </p>

              <div className="install-action-box">
                <button className="open-web-ide-cta" onClick={() => onNavigate('playground')}>
                  <Play size={14} fill="currentColor" />
                  <span>Launch Web Workstation Now</span>
                </button>
              </div>

              <div className="card-features-list">
                <div className="feat-item">✓ Lightweight (&lt; 250 KB WASM bundle)</div>
                <div className="feat-item">✓ 100% offline-capable with local storage</div>
                <div className="feat-item">✓ Instant theorem sharing and state export</div>
              </div>
            </div>

            {/* Option 3: Native Rust CLI */}
            <div className="install-card">
              <div className="card-top">
                <div className="install-icon-box">
                  <Terminal size={24} className="text-amber" />
                </div>
              </div>
              <h2 className="install-card-title">Native Rust CLI</h2>
              <p className="install-card-desc">
                Install the high-performance CLI compiler and LSP daemon directly from source with Cargo for automated testing and CI/CD pipelines.
              </p>

              <div className="install-code-box">
                <div className="code-box-label">Build &amp; Install from Source</div>
                <div className="code-line-wrapper">
                  <code>cargo install --path proof</code>
                  <button 
                    className="copy-btn"
                    onClick={() => handleCopy('cargo install --path proof', 'cargo')}
                  >
                    {copiedId === 'cargo' ? <Check size={14} className="text-emerald" /> : <Copy size={14} />}
                  </button>
                </div>
              </div>

              <div className="card-features-list">
                <div className="feat-item">✓ Microsecond batch verification: <code>proofer check</code></div>
                <div className="feat-item">✓ JSON-RPC Language Server: <code>proofer lsp</code></div>
                <div className="feat-item">✓ REST daemon: <code>proofer --server --port 8086</code></div>
              </div>
            </div>

            {/* Option 4: Docker Container */}
            <div className="install-card">
              <div className="card-top">
                <div className="install-icon-box">
                  <Cpu size={24} className="text-purple" />
                </div>
              </div>
              <h2 className="install-card-title">Docker &amp; Cloud</h2>
              <p className="install-card-desc">
                Deploy the full Proofer server and web workstation in an Alpine-based production container (&lt; 30 MB).
              </p>

              <div className="install-code-box">
                <div className="code-box-label">Run Container</div>
                <div className="code-line-wrapper">
                  <code>docker run -d -p 8086:8086 proofer:latest</code>
                  <button 
                    className="copy-btn"
                    onClick={() => handleCopy('docker run -d -p 8086:8086 proofer:latest', 'docker')}
                  >
                    {copiedId === 'docker' ? <Check size={14} className="text-emerald" /> : <Copy size={14} />}
                  </button>
                </div>
              </div>

              <div className="card-features-list">
                <div className="feat-item">✓ One-click deployment to Fly.io or Cloud Run</div>
                <div className="feat-item">✓ Embedded static assets with healthchecks</div>
                <div className="feat-item">✓ Hardened non-root runtime environment</div>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* Verification & System Requirements */}
      <section className="install-reqs-section">
        <div className="section-container">
          <div className="reqs-card">
            <h3 className="reqs-title">System Compatibility</h3>
            <div className="reqs-grid">
              <div className="req-col">
                <span className="req-head">Supported Operating Systems</span>
                <p>Linux (x86_64, aarch64), macOS (Apple Silicon &amp; Intel), Windows (WSL2 &amp; Native).</p>
              </div>
              <div className="req-col">
                <span className="req-head">Supported Browsers</span>
                <p>Chrome, Firefox, Safari, Edge, Brave (WebAssembly &amp; WebGL / Canvas enabled).</p>
              </div>
              <div className="req-col">
                <span className="req-head">Toolchain Requirements (CLI only)</span>
                <p>Rust 1.80+ with <code>cargo</code> and <code>musl-tools</code> (optional for static binaries).</p>
              </div>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};

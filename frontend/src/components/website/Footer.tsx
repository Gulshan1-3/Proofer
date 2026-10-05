import React from 'react';
import { ExternalLink } from 'lucide-react';
import { WebsiteView } from './Navbar';

interface FooterProps {
  onNavigate: (view: WebsiteView, docSectionId?: string) => void;
}

export const Footer: React.FC<FooterProps> = ({ onNavigate }) => {
  return (
    <footer className="site-footer">
      <div className="site-footer-container">
        <div className="footer-columns">
          {/* Brand & Mission Column */}
          <div className="footer-col brand-col">
            <div className="footer-brand">
              <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" className="footer-logo">
                <polygon points="12 2 2 22 22 22 12 2" />
              </svg>
              <span className="footer-brand-title">PROOFER</span>
            </div>
            <p className="footer-description">
              The high-performance interactive theorem proving workstation with microsecond formal verification, readable declarative mathematical syntax, and real-time visual CAD feedback.
            </p>
            <div className="footer-stats">
              <div className="stat-pill">
                <span className="stat-label">Verification:</span>
                <span className="stat-val">&lt; 10 µs</span>
              </div>
              <div className="stat-pill">
                <span className="stat-label">Kernel:</span>
                <span className="stat-val">100% Deterministic</span>
              </div>
            </div>
          </div>

          {/* Documentation Links */}
          <div className="footer-col">
            <h4 className="footer-heading">Documentation</h4>
            <ul className="footer-links">
              <li><button onClick={() => onNavigate('docs', 'intro-philosophy')}>Core Philosophy</button></li>
              <li><button onClick={() => onNavigate('docs', 'quickstart-5min')}>5-Minute Quickstart</button></li>
              <li><button onClick={() => onNavigate('docs', 'lang-structure')}>Language Reference</button></li>
              <li><button onClick={() => onNavigate('docs', 'kernel-architecture')}>Trusted LCF Kernel</button></li>
              <li><button onClick={() => onNavigate('docs', 'geo-certificates')}>Synthetic Geometry Rules</button></li>
              <li><button onClick={() => onNavigate('docs', 'comparison-lean4')}>Proofer vs. Lean 4 / Coq</button></li>
            </ul>
          </div>

          {/* Ecosystem & Tooling */}
          <div className="footer-col">
            <h4 className="footer-heading">Ecosystem & Workstation</h4>
            <ul className="footer-links">
              <li><button onClick={() => onNavigate('playground')}>Live Web IDE Workstation</button></li>
              <li><button onClick={() => onNavigate('showcase')}>6 Showcase Domains</button></li>
              <li><button onClick={() => onNavigate('install')}>VS Code Extension (.vsix)</button></li>
              <li><button onClick={() => onNavigate('install')}>Native Rust CLI (Cargo)</button></li>
              <li><button onClick={() => onNavigate('install')}>Docker & Cloud Deploy</button></li>
              <li><button onClick={() => onNavigate('docs', 'mathlib-overview')}>Proofer Mathlib</button></li>
            </ul>
          </div>

          {/* Community & Open Source */}
          <div className="footer-col">
            <h4 className="footer-heading">Open Source</h4>
            <ul className="footer-links">
              <li>
                <a href="https://github.com/Gulshan1-3/Proofer" target="_blank" rel="noopener noreferrer" className="external-link">
                  <span>GitHub Repository</span>
                  <ExternalLink size={12} />
                </a>
              </li>
              <li>
                <a href="https://github.com/Gulshan1-3/Proofer/issues" target="_blank" rel="noopener noreferrer" className="external-link">
                  <span>Issue Tracker</span>
                  <ExternalLink size={12} />
                </a>
              </li>
              <li>
                <a href="https://github.com/Gulshan1-3/Proofer/blob/main/LICENSE" target="_blank" rel="noopener noreferrer" className="external-link">
                  <span>Apache 2.0 / MIT License</span>
                  <ExternalLink size={12} />
                </a>
              </li>
              <li>
                <button onClick={() => onNavigate('docs', 'dev-cli-lsp')}>Language Server Protocol (LSP)</button>
              </li>
            </ul>
          </div>
        </div>

        {/* Bottom Bar */}
        <div className="footer-bottom">
          <div className="copyright">
            © {new Date().getFullYear()} Proofer Project. Built with Rust, WebAssembly, and React. Zero AI Hallucinations in Proof Checking.
          </div>
          <div className="footer-tags">
            <span className="tag">First-Order Logic</span>
            <span className="tag">Synthetic Geometry</span>
            <span className="tag">LCF Architecture</span>
          </div>
        </div>
      </div>
    </footer>
  );
};

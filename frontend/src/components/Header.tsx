import React from 'react';
import { Cpu, Folder, GitBranch, Eye, EyeOff, Search, FileCode } from 'lucide-react';
import { VerificationResponse } from '../types';

interface HeaderProps {
  verification: VerificationResponse | null;
  daemonOnline: boolean;
  onSelectExample: (exampleKey: string) => void;
  activeFileName: string;
  filesCount: number;
  commitCount: number;
  isFilesOpen: boolean;
  isVcsOpen: boolean;
  onToggleFiles: () => void;
  onToggleVcs: () => void;
  showCanvas: boolean;
  onToggleCanvas: () => void;
  onOpenCommandPalette: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  verification,
  daemonOnline,
  onSelectExample,
  activeFileName,
  filesCount,
  commitCount,
  isFilesOpen,
  isVcsOpen,
  onToggleFiles,
  onToggleVcs,
  showCanvas,
  onToggleCanvas,
  onOpenCommandPalette,
}) => {
  const isVerified = verification?.verified ?? false;

  return (
    <header className="workstation-header">
      {/* Brand & Desktop Menu Bar */}
      <div className="header-left">
        <div className="brand-lockup">
          <svg className="brand-logo" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="#e6edf3" strokeWidth="2.5">
            <polygon points="12 2 2 22 22 22 12 2" />
          </svg>
          <span className="brand-title">PROOFER</span>
          <span className="brand-version">v0.4.2</span>
        </div>

        {/* Workstation Menus */}
        <nav className="header-menu-nav">
          <button className={`menu-nav-btn ${isFilesOpen ? 'active' : ''}`} onClick={onToggleFiles}>
            <Folder size={13} />
            <span>Files ({filesCount})</span>
          </button>

          <button className={`menu-nav-btn ${isVcsOpen ? 'active' : ''}`} onClick={onToggleVcs}>
            <GitBranch size={13} />
            <span>VCS ({commitCount})</span>
          </button>

          <button className={`menu-nav-btn ${showCanvas ? 'active' : ''}`} onClick={onToggleCanvas}>
            {showCanvas ? <Eye size={13} /> : <EyeOff size={13} />}
            <span>Canvas ({showCanvas ? '3-Pane' : '2-Pane'})</span>
          </button>
        </nav>

        {/* Active File Name Indicator */}
        <div className="header-active-file">
          <FileCode size={12} className="file-icon-bullet" />
          <span className="active-file-title">{activeFileName}</span>
        </div>
      </div>

      {/* Center: Command Palette Trigger & Theorem Selector */}
      <div className="header-center">
        <button className="command-palette-trigger" onClick={onOpenCommandPalette} title="Open Command Palette (Ctrl+K)">
          <Search size={13} />
          <span className="trigger-text">Search commands, proofs...</span>
          <kbd className="trigger-kbd">⌘K</kbd>
        </button>

        <div className="theorem-select-wrap">
          <label htmlFor="example-select" className="select-label">THEOREM:</label>
          <select
            id="example-select"
            className="theorem-select"
            defaultValue=""
            onChange={(e) => {
              if (e.target.value) {
                onSelectExample(e.target.value);
                e.target.value = '';
              }
            }}
          >
            <option value="" disabled>Load Theorem Template...</option>
            <optgroup label="Geometry — Triangles">
              <option value="isosceles">Isosceles Base Angles Theorem</option>
            </optgroup>
            <optgroup label="Geometry — Circles">
              <option value="thales">Thales' Theorem (Inscribed Right Angle)</option>
            </optgroup>
            <optgroup label="Geometry — Quadrilaterals">
              <option value="cyclic_quad">Cyclic Quadrilateral Opposite Angles (180°)</option>
              <option value="parallelogram">Parallelogram Opposite Sides Congruence</option>
            </optgroup>
            <optgroup label="Non-Geometry — Pure Logic">
              <option value="logic_identity">Identity Law (P → P)</option>
              <option value="propositional_logic">Hypothetical Syllogism (P → Q ∧ Q → R → P → R)</option>
            </optgroup>
            <optgroup label="Standard Math Library (mathlib)">
              <option value="mathlib_triangles">mathlib/triangles.proof</option>
              <option value="mathlib_circles">mathlib/circles.proof</option>
              <option value="mathlib_quads">mathlib/quadrilaterals.proof</option>
              <option value="mathlib_logic">mathlib/logic.proof</option>
            </optgroup>
            <optgroup label="Negative Rejection Tests">
              <option value="invalid_rule">Invalid Geometric Rule (Kernel Rejection)</option>
            </optgroup>
          </select>
        </div>
      </div>

      {/* Right: Technical Instrumentation Telemetry */}
      <div className="header-right">
        <div className={`telemetry-item kernel-telemetry ${isVerified ? 'verified' : 'rejected'}`}>
          <span className="telemetry-label">KERNEL</span>
          <span className="telemetry-val">
            <span className={`telemetry-dot ${isVerified ? 'verified' : 'rejected'}`} />
            {isVerified ? 'VERIFIED' : 'PENDING'}
          </span>
        </div>

        <div className={`telemetry-item daemon-telemetry ${daemonOnline ? 'online' : 'local'}`}>
          <span className="telemetry-label">ENGINE</span>
          <span className="telemetry-val">
            <Cpu size={12} />
            {daemonOnline ? 'DAEMON ONLINE' : 'LOCAL ENGINE'}
          </span>
        </div>
      </div>
    </header>
  );
};

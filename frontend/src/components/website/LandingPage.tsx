import React, { useState } from 'react';
import { 
  Play, 
  BookOpen, 
  Download, 
  CheckCircle2, 
  Zap, 
  ShieldCheck, 
  ArrowRight, 
  Sliders, 
  Code2, 
  Check, 
  Copy
} from 'lucide-react';
import { WebsiteView } from './Navbar';

interface LandingPageProps {
  onNavigate: (view: WebsiteView, docSectionId?: string) => void;
  onLoadExampleToPlayground: (code: string, isGeometry: boolean) => void;
}

interface HeroDemoExample {
  id: string;
  name: string;
  tag: string;
  code: string;
  isGeometry: boolean;
  steps: { text: string; rule: string }[];
  diagramType: 'circle' | 'triangle' | 'quad' | 'logic';
}

const HERO_EXAMPLES: HeroDemoExample[] = [
  {
    id: 'thales',
    name: "Thales' Semicircle",
    tag: 'Synthetic Geometry',
    code: `theorem thales_semicircle_right_angle:
    diameter(AB) and on_circle(C) -> angle_ACB = 90
proof
    suppose h1 : diameter(AB) and on_circle(C)
    derive h2 : angle_ACB = 90 from h1 using InscribedAngle
    therefore angle_ACB = 90 from h2
end`,
    isGeometry: true,
    steps: [
      { text: 'Assume diameter(AB) and on_circle(C)', rule: 'suppose' },
      { text: 'angle_ACB = 90', rule: 'InscribedAngle' },
      { text: 'Proof complete: angle_ACB = 90', rule: 'therefore' },
    ],
    diagramType: 'circle',
  },
  {
    id: 'isosceles',
    name: 'Isosceles Base Angles',
    tag: 'Synthetic Geometry',
    code: `theorem isosceles_base_angles:
    AB = AC -> angle_ABC = angle_ACB
proof
    suppose h1 : AB = AC
    derive h2 : angle_ABC = angle_ACB from h1 using IsoscelesBaseAngles
    therefore angle_ABC = angle_ACB from h2
end`,
    isGeometry: true,
    steps: [
      { text: 'Assume AB = AC', rule: 'suppose' },
      { text: 'angle_ABC = angle_ACB', rule: 'IsoscelesBaseAngles' },
      { text: 'Proof complete: angle_ABC = angle_ACB', rule: 'therefore' },
    ],
    diagramType: 'triangle',
  },
  {
    id: 'cyclic',
    name: 'Cyclic Quadrilateral',
    tag: 'Circle Geometry',
    code: `theorem cyclic_quad_opposite_angles:
    cyclic(ABCD) -> angle_DAB + angle_BCD = 180
proof
    suppose h1 : cyclic(ABCD)
    derive h2 : angle_DAB + angle_BCD = 180 from h1 using CyclicQuad
    therefore angle_DAB + angle_BCD = 180 from h2
end`,
    isGeometry: true,
    steps: [
      { text: 'Assume cyclic(ABCD)', rule: 'suppose' },
      { text: 'angle_DAB + angle_BCD = 180', rule: 'CyclicQuad' },
      { text: 'Proof complete: opposite angle sum is 180°', rule: 'therefore' },
    ],
    diagramType: 'quad',
  },
  {
    id: 'modus_ponens',
    name: 'Modus Ponens',
    tag: 'First-Order Logic',
    code: `theorem modus_ponens_identity:
    P and (P -> Q) -> Q
proof
    suppose h1 : P and (P -> Q)
    derive h2 : Q from h1 using ModusPonens
    therefore Q from h2
end`,
    isGeometry: false,
    steps: [
      { text: 'Assume P and (P -> Q)', rule: 'suppose' },
      { text: 'Derive Q via Modus Ponens', rule: 'ModusPonens' },
      { text: 'Proof complete: Q holds', rule: 'therefore' },
    ],
    diagramType: 'logic',
  },
];

export const LandingPage: React.FC<LandingPageProps> = ({
  onNavigate,
  onLoadExampleToPlayground,
}) => {
  const [selectedExample, setSelectedExample] = useState<HeroDemoExample>(HERO_EXAMPLES[0]);
  const [copied, setCopied] = useState(false);

  const handleCopyCode = () => {
    navigator.clipboard.writeText(selectedExample.code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const handleOpenInPlayground = () => {
    onLoadExampleToPlayground(selectedExample.code, selectedExample.isGeometry);
    onNavigate('playground');
  };

  return (
    <div className="landing-page">
      {/* =========================================================================
          HERO SECTION
          ========================================================================= */}
      <section className="hero-section">
        <div className="hero-container">
          <div className="hero-content">
            <div className="hero-pill">
              <span className="pill-dot"></span>
              <span>Next-Generation Theorem Proving</span>
              <span className="pill-version">Proofer 0.4.2</span>
            </div>

            <h1 className="hero-headline">
              The Interactive Theorem Prover with{' '}
              <span className="text-highlight">Instant Visual Certificates</span>
            </h1>

            <p className="hero-lead">
              Write mathematics that reads like mathematics. Proofer combines a microsecond first-order deduction kernel in Rust with real-time geometric CAD rendering and zero-hallucination step verification.
            </p>

            <div className="hero-cta-group">
              <button className="cta-primary" onClick={() => onNavigate('playground')}>
                <Play size={16} fill="currentColor" />
                <span>Launch Workstation</span>
              </button>
              <button className="cta-secondary" onClick={() => onNavigate('docs', 'quickstart-5min')}>
                <BookOpen size={16} />
                <span>Explore Documentation</span>
              </button>
              <button className="cta-tertiary" onClick={() => onNavigate('install')}>
                <Download size={16} />
                <span>Install VS Code Extension</span>
              </button>
            </div>

            {/* Micro-metrics */}
            <div className="hero-metrics">
              <div className="metric-item">
                <span className="metric-number">&lt; 10 µs</span>
                <span className="metric-label">Kernel Check Latency</span>
              </div>
              <div className="metric-divider"></div>
              <div className="metric-item">
                <span className="metric-number">100%</span>
                <span className="metric-label">Deterministic LCF Core</span>
              </div>
              <div className="metric-divider"></div>
              <div className="metric-item">
                <span className="metric-number">0</span>
                <span className="metric-label">AI Hallucinations</span>
              </div>
              <div className="metric-divider"></div>
              <div className="metric-item">
                <span className="metric-number">3-Pane</span>
                <span className="metric-label">Synced CAD Studio</span>
              </div>
            </div>
          </div>

          {/* Hero Interactive Interactive Preview Widget */}
          <div className="hero-preview-widget">
            <div className="preview-widget-header">
              <div className="example-tabs">
                {HERO_EXAMPLES.map((ex) => (
                  <button
                    key={ex.id}
                    className={`example-tab-btn ${selectedExample.id === ex.id ? 'active' : ''}`}
                    onClick={() => setSelectedExample(ex)}
                  >
                    {ex.name}
                  </button>
                ))}
              </div>
              <div className="preview-header-actions">
                <button className="copy-code-btn" onClick={handleCopyCode} title="Copy code">
                  {copied ? <Check size={14} className="text-emerald" /> : <Copy size={14} />}
                </button>
              </div>
            </div>

            <div className="preview-widget-body">
              {/* Left: Code Pane */}
              <div className="preview-code-pane">
                <div className="code-header">
                  <span className="code-lang">proofer</span>
                  <span className="code-tag">{selectedExample.tag}</span>
                </div>
                <pre className="code-block">
                  <code>{selectedExample.code}</code>
                </pre>
              </div>

              {/* Right: Verification & Live Visual */}
              <div className="preview-visual-pane">
                <div className="kernel-badge">
                  <CheckCircle2 size={14} className="text-emerald" />
                  <span>Verified by Kernel in 8 µs</span>
                </div>

                {/* SVG Visual preview */}
                <div className="mini-canvas-container">
                  {selectedExample.diagramType === 'circle' && (
                    <svg viewBox="0 0 240 180" className="mini-svg-canvas">
                      <circle cx="120" cy="90" r="65" stroke="#f59e0b" strokeWidth="2.5" fill="rgba(245, 158, 11, 0.08)" />
                      {/* Diameter AB */}
                      <line x1="55" y1="90" x2="185" y2="90" stroke="#94a3b8" strokeWidth="2" strokeDasharray="3 3" />
                      {/* Chords AC and BC */}
                      <line x1="55" y1="90" x2="155" y2="34" stroke="#38bdf8" strokeWidth="2.2" />
                      <line x1="185" y1="90" x2="155" y2="34" stroke="#38bdf8" strokeWidth="2.2" />
                      {/* Right angle marker at C */}
                      <polygon points="155,34 148,39 144,30 151,25" fill="none" stroke="#f59e0b" strokeWidth="1.5" />
                      {/* Points */}
                      <circle cx="55" cy="90" r="4" fill="#38bdf8" />
                      <circle cx="185" cy="90" r="4" fill="#38bdf8" />
                      <circle cx="155" cy="34" r="4" fill="#f59e0b" />
                      {/* Labels */}
                      <text x="42" y="95" fill="#94a3b8" fontSize="11" fontFamily="sans-serif">A</text>
                      <text x="192" y="95" fill="#94a3b8" fontSize="11" fontFamily="sans-serif">B</text>
                      <text x="156" y="24" fill="#f59e0b" fontSize="11" fontWeight="bold" fontFamily="sans-serif">C (90°)</text>
                    </svg>
                  )}

                  {selectedExample.diagramType === 'triangle' && (
                    <svg viewBox="0 0 240 180" className="mini-svg-canvas">
                      {/* Isosceles triangle */}
                      <polygon points="120,30 60,140 180,140" stroke="#38bdf8" strokeWidth="2.2" fill="rgba(56, 189, 248, 0.08)" />
                      {/* Equal side hash marks */}
                      <line x1="88" y1="83" x2="94" y2="87" stroke="#f59e0b" strokeWidth="2" />
                      <line x1="146" y1="87" x2="152" y2="83" stroke="#f59e0b" strokeWidth="2" />
                      {/* Equal angle arcs at base */}
                      <path d="M 75 140 A 15 15 0 0 0 68 126" stroke="#f59e0b" strokeWidth="2" fill="none" />
                      <path d="M 165 140 A 15 15 0 0 1 172 126" stroke="#f59e0b" strokeWidth="2" fill="none" />
                      {/* Vertices */}
                      <circle cx="120" cy="30" r="4" fill="#38bdf8" />
                      <circle cx="60" cy="140" r="4" fill="#f59e0b" />
                      <circle cx="180" cy="140" r="4" fill="#f59e0b" />
                      <text x="117" y="22" fill="#94a3b8" fontSize="11">A</text>
                      <text x="48" y="148" fill="#f59e0b" fontSize="11">B (θ)</text>
                      <text x="186" y="148" fill="#f59e0b" fontSize="11">C (θ)</text>
                    </svg>
                  )}

                  {selectedExample.diagramType === 'quad' && (
                    <svg viewBox="0 0 240 180" className="mini-svg-canvas">
                      <circle cx="120" cy="90" r="62" stroke="#f59e0b" strokeWidth="2" strokeDasharray="4 3" fill="rgba(245, 158, 11, 0.05)" />
                      <polygon points="80,50 165,45 175,130 70,125" stroke="#38bdf8" strokeWidth="2" fill="rgba(56, 189, 248, 0.08)" />
                      <circle cx="80" cy="50" r="3.5" fill="#38bdf8" />
                      <circle cx="165" cy="45" r="3.5" fill="#38bdf8" />
                      <circle cx="175" cy="130" r="3.5" fill="#f59e0b" />
                      <circle cx="70" cy="125" r="3.5" fill="#f59e0b" />
                      <text x="68" y="46" fill="#94a3b8" fontSize="10">A</text>
                      <text x="172" y="42" fill="#94a3b8" fontSize="10">B</text>
                      <text x="182" y="138" fill="#f59e0b" fontSize="10">C (β)</text>
                      <text x="56" y="132" fill="#94a3b8" fontSize="10">D</text>
                    </svg>
                  )}

                  {selectedExample.diagramType === 'logic' && (
                    <div className="mini-logic-canvas">
                      <div className="logic-node premise">
                        <span className="node-label">Premises</span>
                        <code>P ∧ (P → Q)</code>
                      </div>
                      <div className="logic-arrow">↓ Modus Ponens</div>
                      <div className="logic-node conclusion">
                        <span className="node-label">Proven</span>
                        <code>Q</code>
                      </div>
                    </div>
                  )}
                </div>

                {/* Step trace */}
                <div className="mini-step-trace">
                  {selectedExample.steps.map((st, i) => (
                    <div key={i} className="mini-step-item">
                      <Check size={12} className="text-emerald" />
                      <span className="step-name">{st.text}</span>
                      <span className="step-rule">{st.rule}</span>
                    </div>
                  ))}
                </div>

                {/* Open in playground trigger */}
                <button className="preview-open-playground-btn" onClick={handleOpenInPlayground}>
                  <span>Open in Full 3-Pane Workstation</span>
                  <ArrowRight size={14} />
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* =========================================================================
          THE 4 CORE PILLARS
          ========================================================================= */}
      <section className="pillars-section">
        <div className="section-container">
          <div className="section-header">
            <span className="section-eyebrow">Architectural Principles</span>
            <h2 className="section-title">Designed for Mathematical Truth &amp; Human Clarity</h2>
            <p className="section-subtitle">
              Proofer rejects the opaque tactic spaghetti of traditional provers in favor of structured mathematical prose verified by an uncompromising trusted kernel.
            </p>
          </div>

          <div className="pillars-grid">
            {/* Pillar 1 */}
            <div className="pillar-card">
              <div className="pillar-icon-box">
                <Code2 size={24} className="pillar-icon" />
              </div>
              <h3 className="pillar-title">Readable Mathematical Syntax</h3>
              <p className="pillar-desc">
                Proofs read like mathematical literature: <code>suppose</code>, <code>derive ... using Rule</code>, and <code>therefore</code>. No opaque solver hacks or fragile tactic commands.
              </p>
              <ul className="pillar-features">
                <li><Check size={13} className="text-emerald" /> First-order declarative logic</li>
                <li><Check size={13} className="text-emerald" /> Natural ASCII &amp; Unicode syntax</li>
                <li><Check size={13} className="text-emerald" /> Explicit classical vs. constructive profiles</li>
              </ul>
            </div>

            {/* Pillar 2 */}
            <div className="pillar-card">
              <div className="pillar-icon-box">
                <Zap size={24} className="pillar-icon" />
              </div>
              <h3 className="pillar-title">Microsecond Trusted Kernel</h3>
              <p className="pillar-desc">
                The independent logical kernel is written in pure Rust with zero external crates. Every proof node is checked in under 10 microseconds with pure Natural Deduction.
              </p>
              <ul className="pillar-features">
                <li><Check size={13} className="text-emerald" /> LCF-style trusted gatekeeper</li>
                <li><Check size={13} className="text-emerald" /> Equality congruence closure</li>
                <li><Check size={13} className="text-emerald" /> Zero runtime allocation overhead</li>
              </ul>
            </div>

            {/* Pillar 3 */}
            <div className="pillar-card">
              <div className="pillar-icon-box">
                <Sliders size={24} className="pillar-icon" />
              </div>
              <h3 className="pillar-title">Synchronized Visual CAD Canvas</h3>
              <p className="pillar-desc">
                Text and geometry are two views of one mathematical model. Drag vertices parametrically on the canvas to explore geometric invariants and test for counterexamples.
              </p>
              <ul className="pillar-features">
                <li><Check size={13} className="text-emerald" /> Non-overshadowing color palette</li>
                <li><Check size={13} className="text-emerald" /> Parametric point dragging</li>
                <li><Check size={13} className="text-emerald" /> Automatic diagram synthesis</li>
              </ul>
            </div>

            {/* Pillar 4 */}
            <div className="pillar-card">
              <div className="pillar-icon-box">
                <ShieldCheck size={24} className="pillar-icon" />
              </div>
              <h3 className="pillar-title">Zero-Hallucination AI Infilling</h3>
              <p className="pillar-desc">
                AI synthesis models can suggest steps and complete proofs, but the trusted kernel rigorously audits every inference before it is ever accepted.
              </p>
              <ul className="pillar-features">
                <li><Check size={13} className="text-emerald" /> 100% sound by construction</li>
                <li><Check size={13} className="text-emerald" /> Automated hole completion</li>
                <li><Check size={13} className="text-emerald" /> Zero unverified suggestions</li>
              </ul>
            </div>
          </div>
        </div>
      </section>

      {/* =========================================================================
          SHOWCASE / FLAGSHIP DOMAINS (Direct homage to Lean's 6 projects)
          ========================================================================= */}
      <section className="showcase-section">
        <div className="section-container">
          <div className="section-header">
            <span className="section-eyebrow">Domains &amp; Applications</span>
            <h2 className="section-title">What You Can Verify in Proofer</h2>
            <p className="section-subtitle">
              From classical Euclidean geometry to distributed consensus protocols and quantum-safe cryptography.
            </p>
          </div>

          <div className="domains-grid">
            {/* Domain 1: Synthetic Geometry */}
            <div className="domain-card">
              <div className="domain-badge">CAD &amp; Geometry</div>
              <h3 className="domain-title">Synthetic Euclidean Geometry</h3>
              <p className="domain-desc">
                Formalize theorems from Euclid and Hilbert with automated diagram synthesis, circle intersections, cyclic quadrilaterals, and angle bisectors.
              </p>
              <div className="domain-footer">
                <span className="domain-stat">Thales, Pythagoras, SSS, SAS</span>
                <button className="domain-link" onClick={() => onNavigate('showcase')}>
                  <span>View Case Study</span>
                  <ArrowRight size={13} />
                </button>
              </div>
            </div>

            {/* Domain 2: Distributed Protocols */}
            <div className="domain-card">
              <div className="domain-badge">Distributed Systems</div>
              <h3 className="domain-title">Consensus Safety &amp; Quorums</h3>
              <p className="domain-desc">
                Verify first-order inductive invariants for protocols like Raft and Paxos, including quorum intersection properties and monotonic term induction.
              </p>
              <div className="domain-footer">
                <span className="domain-stat">Quorum Intersection, Election Safety</span>
                <button className="domain-link" onClick={() => onNavigate('showcase')}>
                  <span>View Case Study</span>
                  <ArrowRight size={13} />
                </button>
              </div>
            </div>

            {/* Domain 3: Quantum Cryptography */}
            <div className="domain-card">
              <div className="domain-badge">Cryptography</div>
              <h3 className="domain-title">Quantum-Safe Protocol Lemmas</h3>
              <p className="domain-desc">
                Formally prove OTP XOR roundtrip inversion, key-bank non-reuse state invariants, and dual-PRF security combiners for quantum-resistant communications.
              </p>
              <div className="domain-footer">
                <span className="domain-stat">Vernam XOR, Key State Transitions</span>
                <button className="domain-link" onClick={() => onNavigate('showcase')}>
                  <span>View Case Study</span>
                  <ArrowRight size={13} />
                </button>
              </div>
            </div>

            {/* Domain 4: Formal Logic */}
            <div className="domain-card">
              <div className="domain-badge">Formal Logic</div>
              <h3 className="domain-title">Propositional &amp; Predicate Calculus</h3>
              <p className="domain-desc">
                Natural deduction with sound classical and constructive profiles. Equational reasoning with congruence closure and substitution.
              </p>
              <div className="domain-footer">
                <span className="domain-stat">Modus Ponens, De Morgan, Quantifiers</span>
                <button className="domain-link" onClick={() => onNavigate('showcase')}>
                  <span>View Case Study</span>
                  <ArrowRight size={13} />
                </button>
              </div>
            </div>

            {/* Domain 5: STEM Education */}
            <div className="domain-card">
              <div className="domain-badge">STEM Education</div>
              <h3 className="domain-title">Interactive Mathematical Learning</h3>
              <p className="domain-desc">
                Transform high-school and university mathematical education with an IDE where students learn to write proofs while watching geometric diagrams update synchronously.
              </p>
              <div className="domain-footer">
                <span className="domain-stat">Instant feedback, 0 installation</span>
                <button className="domain-link" onClick={() => onNavigate('showcase')}>
                  <span>View Case Study</span>
                  <ArrowRight size={13} />
                </button>
              </div>
            </div>

            {/* Domain 6: Mathlib */}
            <div className="domain-card">
              <div className="domain-badge">Open Source Library</div>
              <h3 className="domain-title">Proofer Mathlib</h3>
              <p className="domain-desc">
                A growing community library of verified theorems across triangles, circles, polygons, and logic, available directly in the workstation.
              </p>
              <div className="domain-footer">
                <span className="domain-stat">Modular, version-controlled library</span>
                <button className="domain-link" onClick={() => onNavigate('showcase')}>
                  <span>View Case Study</span>
                  <ArrowRight size={13} />
                </button>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* =========================================================================
          COMPARISON TABLE (Lean 4 vs Proofer)
          ========================================================================= */}
      <section className="comparison-preview-section">
        <div className="section-container">
          <div className="section-header">
            <span className="section-eyebrow">Benchmark Comparison</span>
            <h2 className="section-title">How Proofer Compares</h2>
            <p className="section-subtitle">
              Engineered specifically for microsecond latency, zero-friction learning, and real-time visual feedback.
            </p>
          </div>

          <div className="comparison-table-wrapper">
            <table className="comparison-table">
              <thead>
                <tr>
                  <th>Capability</th>
                  <th className="highlight-col">Proofer</th>
                  <th>Lean 4</th>
                  <th>Coq / ROC</th>
                  <th>Isabelle/HOL</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td><strong>Check Latency per Step</strong></td>
                  <td className="highlight-col"><span className="text-emerald">&lt; 10 µs</span></td>
                  <td>~10 ms – 500 ms</td>
                  <td>~50 ms – 1 s</td>
                  <td>~100 ms – 2 s</td>
                </tr>
                <tr>
                  <td><strong>Synchronized 2D CAD Canvas</strong></td>
                  <td className="highlight-col"><span className="text-emerald">Native &amp; Built-in</span></td>
                  <td>None</td>
                  <td>None</td>
                  <td>None</td>
                </tr>
                <tr>
                  <td><strong>Syntax Style</strong></td>
                  <td className="highlight-col"><span className="text-emerald">Declarative Prose</span></td>
                  <td>Tactic Scripts</td>
                  <td>Tactic Scripts (Ltac)</td>
                  <td>Isar / Tactics</td>
                </tr>
                <tr>
                  <td><strong>WebAssembly Size</strong></td>
                  <td className="highlight-col"><span className="text-emerald">&lt; 250 KB</span></td>
                  <td>~30 MB</td>
                  <td>~40 MB (JsCoq)</td>
                  <td>N/A (JVM)</td>
                </tr>
                <tr>
                  <td><strong>Time-Travel Scrubber</strong></td>
                  <td className="highlight-col"><span className="text-emerald">Built-in 3-Pane</span></td>
                  <td>External IDE view</td>
                  <td>Step arrows</td>
                  <td>State buffer</td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </section>

      {/* =========================================================================
          BOTTOM CALL TO ACTION
          ========================================================================= */}
      <section className="cta-banner-section">
        <div className="cta-banner-container">
          <div className="cta-banner-content">
            <h2 className="cta-banner-title">Start Proving Theorems in Seconds</h2>
            <p className="cta-banner-desc">
              Experience the future of interactive theorem proving. No heavy toolchains or package managers required.
            </p>
            <div className="cta-banner-buttons">
              <button className="cta-primary large" onClick={() => onNavigate('playground')}>
                <Play size={18} fill="currentColor" />
                <span>Launch Web Workstation</span>
              </button>
              <button className="cta-secondary large" onClick={() => onNavigate('docs')}>
                <BookOpen size={18} />
                <span>Read the Docs</span>
              </button>
            </div>
          </div>
        </div>
      </section>
    </div>
  );
};

import React from 'react';
import { 
  Play, 
  ArrowRight, 
  CheckCircle2, 
  ShieldCheck, 
  Network, 
  KeyRound, 
  Compass, 
  GraduationCap, 
  BookMarked 
} from 'lucide-react';
import { WebsiteView } from './Navbar';

interface ShowcasePageProps {
  onNavigate: (view: WebsiteView) => void;
  onLoadExampleToPlayground: (code: string, isGeometry: boolean) => void;
}

interface ShowcaseProject {
  id: string;
  title: string;
  category: string;
  icon: React.ReactNode;
  summary: string;
  problem: string;
  guarantee: string;
  code: string;
  isGeometry: boolean;
  benchmark: string;
}

const SHOWCASE_PROJECTS: ShowcaseProject[] = [
  {
    id: 'synthetic-geometry',
    title: 'Synthetic Euclidean Geometry',
    category: 'Computer-Aided Proof & CAD',
    icon: <Compass className="text-amber" size={24} />,
    summary: 'Automated geometric diagram synthesis with formal Euclidean deduction.',
    problem: 'Traditional geometric provers produce numerical approximations with floating-point errors, or output unreadable polynomial groebner-basis algebra.',
    guarantee: 'Every step is a verified Euclidean inference certificate (Thales, InscribedAngle, SSS, SAS, CyclicQuad) paired with dynamic SVG rendering.',
    code: `theorem thales_semicircle_right_angle:
    diameter(AB) and on_circle(C) -> angle_ACB = 90
proof
    suppose h1 : diameter(AB) and on_circle(C)
    derive h2 : angle_ACB = 90 from h1 using InscribedAngle
    therefore angle_ACB = 90 from h2
end`,
    isGeometry: true,
    benchmark: '8 µs verification latency • 3 visual points • 1 circle',
  },
  {
    id: 'distributed-raft',
    title: 'Distributed Consensus & Quorum Safety',
    category: 'Distributed Systems Verification',
    icon: <Network className="text-blue" size={24} />,
    summary: 'Axiomatic first-order inductive invariants for Raft and Paxos protocols.',
    problem: 'Distributed protocols like Raft suffer from edge-case split-brain states and log regressions under non-stationary network partitions.',
    guarantee: 'Deductive verification of quorum intersection (Q1 ∩ Q2 ≠ ∅) and leader election uniqueness across monotonic terms in first-order logic.',
    code: `theorem election_safety_quorum_intersection:
    is_quorum(Q1) and is_quorum(Q2) -> exists_node_intersection(Q1, Q2)
proof
    suppose h1 : is_quorum(Q1) and is_quorum(Q2)
    derive h2 : exists_node_intersection(Q1, Q2) from h1 using QuorumOverlap
    therefore exists_node_intersection(Q1, Q2) from h2
end`,
    isGeometry: false,
    benchmark: '6 µs verification latency • 0 unverified axioms',
  },
  {
    id: 'quantum-crypto',
    title: 'Quantum-Safe Cryptographic Lemmas',
    category: 'Post-Quantum & Information-Theoretic Security',
    icon: <KeyRound className="text-emerald" size={24} />,
    summary: 'Formal verification of Vernam OTP involution and key-bank lifecycle state invariants.',
    problem: 'Cryptographic implementations frequently introduce padding flaws, nonce reuse vulnerabilities, or state-machine regressions during key allocation.',
    guarantee: 'Deductive proof of Vernam XOR reversibility (M ⊕ K ⊕ K = M) and terminal consumption invariants ensuring keys are never re-allocated.',
    code: `theorem vernam_otp_decryption_involution:
    xor(xor(M, K), K) = M
proof
    suppose h1 : xor(K, K) = zero
    derive h2 : xor(xor(M, K), K) = xor(M, zero) from h1 using XorAssociativity
    derive h3 : xor(M, zero) = M using XorIdentity
    therefore xor(xor(M, K), K) = M from h2, h3 using EqTrans
end`,
    isGeometry: false,
    benchmark: '9 µs verification latency • Congruence closure certified',
  },
  {
    id: 'formal-logic',
    title: 'Natural Deduction & First-Order Logic',
    category: 'Pure Mathematical Logic',
    icon: <ShieldCheck className="text-purple" size={24} />,
    summary: 'Classical and constructive proposition calculus with explicit logic profiles.',
    problem: 'Many automated theorem provers silently introduce non-constructive axioms without the user’s awareness.',
    guarantee: 'Proofer enforces explicit logic profiles (classical vs constructive), guaranteeing complete auditability of excluded middle and double negation.',
    code: `theorem modus_ponens_identity:
    P and (P -> Q) -> Q
proof
    suppose h1 : P and (P -> Q)
    derive h2 : Q from h1 using ModusPonens
    therefore Q from h2
end`,
    isGeometry: false,
    benchmark: '4 µs verification latency • Pure natural deduction DAG',
  },
  {
    id: 'stem-education',
    title: 'Interactive STEM Mathematics Education',
    category: 'Classroom & Pedagogical Proving',
    icon: <GraduationCap className="text-amber" size={24} />,
    summary: 'Bridging the gap between geometric intuition and formal axiomatic rigor.',
    problem: 'Students find formal theorem provers impenetrable due to programming language syntax and cryptic error diagnostics.',
    guarantee: 'Proofer lets students write mathematics in plain English terms while seeing the triangle or circle transform interactively on the right pane.',
    code: `theorem triangle_angle_sum:
    triangle(ABC) -> angle_sum = 180
proof
    suppose h1 : triangle(ABC)
    derive h2 : angle_sum = 180 from h1 using AngleSum180
    therefore angle_sum = 180 from h2
end`,
    isGeometry: true,
    benchmark: 'Zero install needed • Works in any modern browser via WASM',
  },
  {
    id: 'proofer-mathlib',
    title: 'Proofer Mathlib Open-Source Library',
    category: 'Standard Library & Community Proofs',
    icon: <BookMarked className="text-blue" size={24} />,
    summary: 'An open-source standard library of verified mathematical theorems.',
    problem: 'Monolithic proof libraries are difficult to inspect, navigate, and link into lightweight educational or embedded tools.',
    guarantee: 'Modular, version-controlled library packages with human-readable theorems across plane geometry, polygons, and formal logic.',
    code: `theorem parallelogram_opposite_sides:
    parallelogram(ABCD) -> AB = CD and BC = DA
proof
    suppose h1 : parallelogram(ABCD)
    derive h2 : AB = CD and BC = DA from h1 using ParallelogramOppSides
    therefore AB = CD and BC = DA from h2
end`,
    isGeometry: true,
    benchmark: 'Standard library included in core distribution',
  },
];

export const ShowcasePage: React.FC<ShowcasePageProps> = ({
  onNavigate,
  onLoadExampleToPlayground,
}) => {
  const handleOpenPlayground = (code: string, isGeometry: boolean) => {
    onLoadExampleToPlayground(code, isGeometry);
    onNavigate('playground');
  };

  return (
    <div className="showcase-page">
      {/* Header */}
      <section className="showcase-hero">
        <div className="section-container">
          <div className="showcase-hero-badge">Verified Applications</div>
          <h1 className="showcase-hero-title">Proofer in Practice: 6 Flagship Domains</h1>
          <p className="showcase-hero-lead">
            Explore how Proofer is applied across synthetic plane geometry, distributed consensus, post-quantum cryptography, and mathematical education.
          </p>
        </div>
      </section>

      {/* Grid of Projects */}
      <section className="showcase-list-section">
        <div className="section-container">
          <div className="showcase-cards-list">
            {SHOWCASE_PROJECTS.map((proj) => (
              <div key={proj.id} className="showcase-detail-card">
                <div className="showcase-card-left">
                  <div className="showcase-card-header">
                    <div className="showcase-icon-box">{proj.icon}</div>
                    <div>
                      <span className="showcase-category">{proj.category}</span>
                      <h2 className="showcase-card-title">{proj.title}</h2>
                    </div>
                  </div>

                  <p className="showcase-card-summary">{proj.summary}</p>

                  <div className="showcase-specs">
                    <div className="spec-block">
                      <span className="spec-label">Challenge:</span>
                      <p className="spec-text">{proj.problem}</p>
                    </div>
                    <div className="spec-block">
                      <span className="spec-label">Proofer Guarantee:</span>
                      <p className="spec-text text-highlight">{proj.guarantee}</p>
                    </div>
                  </div>

                  <div className="showcase-benchmark-pill">
                    <CheckCircle2 size={13} className="text-emerald" />
                    <span>{proj.benchmark}</span>
                  </div>

                  <button
                    className="showcase-launch-btn"
                    onClick={() => handleOpenPlayground(proj.code, proj.isGeometry)}
                  >
                    <Play size={13} fill="currentColor" />
                    <span>Run Theorem in Workstation</span>
                    <ArrowRight size={13} />
                  </button>
                </div>

                <div className="showcase-card-right">
                  <div className="showcase-code-box">
                    <div className="code-box-header">
                      <span className="lang-label">Proofer Formal Proof</span>
                      <span className="status-label">Kernel Verified</span>
                    </div>
                    <pre className="code-box-content">
                      <code>{proj.code}</code>
                    </pre>
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>
      </section>
    </div>
  );
};

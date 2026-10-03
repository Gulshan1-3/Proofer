export interface VisualPoint {
  id: string;
  name: string;
  x: number;
  y: number;
  fixed?: boolean;
}

export interface VisualSegment {
  id: string;
  p1: string;
  p2: string;
  name: string;
  color?: string;
  tickCount?: number;
}

export interface VisualAngle {
  id: string;
  vertex: string;
  p1: string;
  p2: string;
  name: string;
  deg: number;
  color?: string;
  arcCount?: number;
}

export type StepKind = 'suppose' | 'derive' | 'construct' | 'have' | 'therefore' | 'qed' | 'statement';

export interface ProofStep {
  id: number;
  text: string;
  status: 'Valid' | 'Rejected';
  rule?: string;
  conclusion?: string;
  kind?: StepKind;
  lineNumber?: number;
  hypothesis?: string; // e.g. "h1", "h2"
  dependencies?: string[]; // e.g. ["h1"]
  entities?: string[]; // e.g. ["AB", "C", "angle_ACB"]
}

export interface TheoremResult {
  name: string;
  status: 'Verified' | 'Rejected';
  proven: string;
  errors: string[];
  steps: ProofStep[];
}

export interface GeometryScene {
  name: string;
  points: VisualPoint[];
}

export interface VerificationResponse {
  status: 'Ok' | 'Error';
  verified: boolean;
  stage?: string;
  errors?: string[];
  theorems: TheoremResult[];
  geometryScene?: GeometryScene;
}

export interface SideConstraint {
  p1: string;
  p2: string;
  length: number;
}

export interface AngleConstraint {
  vertex: string;
  deg: number;
}

export interface FileCommit {
  id: string; // 7-char hash
  timestamp: number;
  message: string;
  content: string;
  verified: boolean;
  tag?: 'Verified' | 'Checkpoint' | 'Manual';
}

export interface ProjectFile {
  id: string;
  name: string; // e.g. "isosceles_theorem.proof"
  content: string;
  createdAt: number;
  updatedAt: number;
  commits: FileCommit[];
}

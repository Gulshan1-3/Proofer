/**
 * Proofer Geometric Constraint Solver & Vector Math Library (Stage 14)
 *
 * Implements high-precision 2D Euclidean geometry operations, constraint projections,
 * dependent entity derivations, and angle sweep metrics.
 */

export interface Point2D {
  x: number;
  y: number;
}

export interface AngleArc {
  vertex: Point2D;
  startAngleRad: number;
  endAngleRad: number;
  sweepAngleDeg: number;
  radius: number;
  pathD: string;
  labelPos: Point2D;
}

export interface SegmentMeasurement {
  p1: Point2D;
  p2: Point2D;
  length: number;
  midpoint: Point2D;
  normalUnit: Point2D;
  labelPos: Point2D;
}

export interface RightAngleSquare {
  p1: Point2D;
  p2: Point2D;
  p3: Point2D;
  pointsString: string;
}

// ==========================================
// 1. Vector Operations
// ==========================================

export function vecSub(a: Point2D, b: Point2D): Point2D {
  return { x: a.x - b.x, y: a.y - b.y };
}

export function vecAdd(a: Point2D, b: Point2D): Point2D {
  return { x: a.x + b.x, y: a.y + b.y };
}

export function vecScale(v: Point2D, s: number): Point2D {
  return { x: v.x * s, y: v.y * s };
}

export function vecDot(a: Point2D, b: Point2D): number {
  return a.x * b.x + a.y * b.y;
}

export function vecCross(a: Point2D, b: Point2D): number {
  return a.x * b.y - a.y * b.x;
}

export function vecLengthSq(v: Point2D): number {
  return v.x * v.x + v.y * v.y;
}

export function vecLength(v: Point2D): number {
  return Math.hypot(v.x, v.y);
}

export function vecNormalize(v: Point2D): Point2D {
  const len = vecLength(v);
  if (len < 1e-9) return { x: 1, y: 0 };
  return { x: v.x / len, y: v.y / len };
}

export function vecDist(a: Point2D, b: Point2D): number {
  return Math.hypot(a.x - b.x, a.y - b.y);
}

// ==========================================
// 2. Constraint Projection Functions
// ==========================================

/**
 * Projects a raw cursor point onto a circle of given center and radius.
 * Clamps to upper semicircle if upperOnly is true (standard Thales orientation).
 */
export function projectToCircle(
  raw: Point2D,
  center: Point2D,
  radius: number,
  upperOnly: boolean = false
): Point2D {
  const d = vecSub(raw, center);
  let angle = Math.atan2(d.y, d.x);

  if (upperOnly) {
    // In SVG coordinates, y is downward, so upper semicircle is negative y (angles between -PI and 0)
    if (angle > 0) {
      angle = angle > Math.PI / 2 ? -Math.PI : 0;
    }
  }

  return {
    x: center.x + radius * Math.cos(angle),
    y: center.y + radius * Math.sin(angle),
  };
}

/**
 * Projects apex vertex A onto the perpendicular bisector of base segment BC.
 * Guarantees AB = AC (Isosceles constraint).
 */
export function projectToPerpendicularBisector(
  rawA: Point2D,
  ptB: Point2D,
  ptC: Point2D
): Point2D {
  const midBC = { x: (ptB.x + ptC.x) / 2, y: (ptB.y + ptC.y) / 2 };
  const bc = vecSub(ptC, ptB);
  const bcLen = vecLength(bc);
  if (bcLen < 1e-6) return rawA;

  // Normal to BC
  const normal = { x: -bc.y / bcLen, y: bc.x / bcLen };

  // Project (rawA - midBC) onto normal
  const toRaw = vecSub(rawA, midBC);
  const distAlongNormal = vecDot(toRaw, normal);

  return {
    x: midBC.x + normal.x * distAlongNormal,
    y: midBC.y + normal.y * distAlongNormal,
  };
}

/**
 * Clamps coordinates within canvas drafting boundary.
 */
export function clampToViewport(
  pt: Point2D,
  minX: number = 30,
  minY: number = 30,
  maxX: number = 570,
  maxY: number = 420
): Point2D {
  return {
    x: Math.max(minX, Math.min(maxX, pt.x)),
    y: Math.max(minY, Math.min(maxY, pt.y)),
  };
}

// ==========================================
// 3. Dependent Entity Calculators
// ==========================================

/**
 * Computes midpoint M on segment BC.
 */
export function computeMidpoint(b: Point2D, c: Point2D): Point2D {
  return {
    x: (b.x + c.x) / 2,
    y: (b.y + c.y) / 2,
  };
}

/**
 * Computes altitude foot H: orthogonal projection of vertex A onto line BC.
 */
export function computeAltitudeFoot(a: Point2D, b: Point2D, c: Point2D): Point2D {
  const bc = vecSub(c, b);
  const bcLenSq = vecLengthSq(bc);
  if (bcLenSq < 1e-9) return { ...b };

  const ba = vecSub(a, b);
  const t = vecDot(ba, bc) / bcLenSq;
  return {
    x: b.x + t * bc.x,
    y: b.y + t * bc.y,
  };
}

/**
 * Computes angle bisector foot D on BC using the Angle Bisector Theorem:
 * BD / DC = AB / AC => t = AB / (AB + AC)
 */
export function computeAngleBisectorFoot(a: Point2D, b: Point2D, c: Point2D): Point2D {
  const lenAB = vecDist(a, b);
  const lenAC = vecDist(a, c);
  const sum = lenAB + lenAC;
  if (sum < 1e-9) return computeMidpoint(b, c);

  const t = lenAB / sum;
  const bc = vecSub(c, b);
  return {
    x: b.x + t * bc.x,
    y: b.y + t * bc.y,
  };
}

/**
 * Computes the 4th vertex D of a parallelogram ABCD (D = C + (A - B)).
 */
export function computeParallelogramD(a: Point2D, b: Point2D, c: Point2D): Point2D {
  return {
    x: c.x + (a.x - b.x),
    y: c.y + (a.y - b.y),
  };
}

// ==========================================
// 4. Metric & Measurement Generators
// ==========================================

/**
 * Computes Euclidean angle in degrees at vertex V between rays V->P1 and V->P2.
 */
export function computeAngleDegrees(v: Point2D, p1: Point2D, p2: Point2D): number {
  const v1 = vecNormalize(vecSub(p1, v));
  const v2 = vecNormalize(vecSub(p2, v));
  const dot = Math.max(-1, Math.min(1, vecDot(v1, v2)));
  return (Math.acos(dot) * 180) / Math.PI;
}

/**
 * Generates an SVG path for an angle arc sweep at vertex V between ray1 and ray2.
 */
export function createAngleArc(
  v: Point2D,
  p1: Point2D,
  p2: Point2D,
  arcRadius: number = 24
): AngleArc {
  const a1 = Math.atan2(p1.y - v.y, p1.x - v.x);
  const a2 = Math.atan2(p2.y - v.y, p2.x - v.x);

  // Normalize sweep direction counterclockwise vs clockwise
  let diff = a2 - a1;
  while (diff < -Math.PI) diff += 2 * Math.PI;
  while (diff > Math.PI) diff -= 2 * Math.PI;

  const sweepFlag = diff > 0 ? 1 : 0;
  const startX = v.x + arcRadius * Math.cos(a1);
  const startY = v.y + arcRadius * Math.sin(a1);
  const endX = v.x + arcRadius * Math.cos(a2);
  const endY = v.y + arcRadius * Math.sin(a2);

  const midAngle = a1 + diff / 2;
  const labelDist = arcRadius + 14;
  const labelPos = {
    x: v.x + labelDist * Math.cos(midAngle),
    y: v.y + labelDist * Math.sin(midAngle),
  };

  const pathD = `M ${startX} ${startY} A ${arcRadius} ${arcRadius} 0 0 ${sweepFlag} ${endX} ${endY}`;
  const deg = Math.round((Math.abs(diff) * 180) / Math.PI);

  return {
    vertex: v,
    startAngleRad: a1,
    endAngleRad: a2,
    sweepAngleDeg: deg,
    radius: arcRadius,
    pathD,
    labelPos,
  };
}

/**
 * Computes perpendicular right-angle square marker points at foot H between base ray and normal ray.
 */
export function createRightAngleSquare(
  h: Point2D,
  baseTarget: Point2D,
  normalTarget: Point2D,
  size: number = 11
): RightAngleSquare {
  const u = vecNormalize(vecSub(baseTarget, h));
  const v = vecNormalize(vecSub(normalTarget, h));

  const p1 = { x: h.x + size * u.x, y: h.y + size * u.y };
  const p2 = { x: h.x + size * u.x + size * v.x, y: h.y + size * u.y + size * v.y };
  const p3 = { x: h.x + size * v.x, y: h.y + size * v.y };

  return {
    p1,
    p2,
    p3,
    pointsString: `${p1.x},${p1.y} ${p2.x},${p2.y} ${p3.x},${p3.y}`,
  };
}

/**
 * Computes segment measurement metadata including length and label position.
 */
export function createSegmentMeasurement(p1: Point2D, p2: Point2D, offsetDistance: number = 14): SegmentMeasurement {
  const length = vecDist(p1, p2);
  const midpoint = { x: (p1.x + p2.x) / 2, y: (p1.y + p2.y) / 2 };
  const d = vecSub(p2, p1);
  const len = vecLength(d);

  const normalUnit = len > 1e-6 ? { x: -d.y / len, y: d.x / len } : { x: 0, y: -1 };
  const labelPos = {
    x: midpoint.x + normalUnit.x * offsetDistance,
    y: midpoint.y + normalUnit.y * offsetDistance,
  };

  return {
    p1,
    p2,
    length,
    midpoint,
    normalUnit,
    labelPos,
  };
}

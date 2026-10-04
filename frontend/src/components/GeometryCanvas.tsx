import React, { useState, useRef, useMemo } from 'react';
import { VisualPoint, ProofStep } from '../types';
import {
  projectToCircle,
  projectToPerpendicularBisector,
  computeMidpoint,
  computeAltitudeFoot,
  computeAngleBisectorFoot,
  computeParallelogramD,
  createAngleArc,
  createRightAngleSquare,
  createSegmentMeasurement,
  vecDist,
  clampToViewport,
  Point2D,
} from '../utils/geometrySolver';

interface GeometryCanvasProps {
  points: VisualPoint[];
  onUpdatePoint: (id: string, x: number, y: number) => void;
  selectedStepId: number | null;
  code: string;
  steps?: ProofStep[];
  onSelectStep?: (stepId: number | null) => void;
  hasMidpoint: boolean;
  hasAltitude: boolean;
  hasBisector: boolean;
  apexAngleDeg: number;
  baseAngleDeg: number;
}

export const GeometryCanvas: React.FC<GeometryCanvasProps> = ({
  points,
  onUpdatePoint,
  selectedStepId,
  code,
  steps = [],
  onSelectStep,
  hasMidpoint,
  hasAltitude,
  hasBisector,
}) => {
  const [draggingId, setDraggingId] = useState<string | null>(null);
  const [snapToGrid, setSnapToGrid] = useState<boolean>(false);
  const [hoveredPointId, setHoveredPointId] = useState<string | null>(null);
  const svgRef = useRef<SVGSVGElement | null>(null);

  const getPt = (name: string): VisualPoint => {
    return points.find((p) => p.name === name) || { id: '0', name, x: 300, y: 200 };
  };

  const ptA = getPt('A');
  const ptB = getPt('B');
  const ptC = getPt('C');
  const ptD = getPt('D');

  const lowerCode = code.toLowerCase();
  const isCircleFigure = lowerCode.includes('circle') || lowerCode.includes('diameter') || lowerCode.includes('thales');
  const isQuadFigure = lowerCode.includes('quadrilateral') || lowerCode.includes('parallelogram') || lowerCode.includes('cyclic') || lowerCode.includes('abcd');
  const hasSideEq_AB_AC = code.includes('AB = AC') || code.includes('AC = AB') || lowerCode.includes('isosceles');

  // Currently active step
  const activeStep = steps.find(s => s.id === selectedStepId);
  const activeEntities = activeStep?.entities || [];

  const isEntityActive = (name: string): boolean => {
    if (!selectedStepId || activeEntities.length === 0) return false;
    return activeEntities.includes(name);
  };

  // ==========================================
  // Dependent Geometric Calculations
  // ==========================================

  // Midpoint M on BC
  const ptM = useMemo(() => {
    const m = computeMidpoint(ptB, ptC);
    return { id: 'pt#m', name: 'M', x: m.x, y: m.y };
  }, [ptB, ptC]);

  // Altitude Foot H on BC
  const ptH = useMemo(() => {
    const h = computeAltitudeFoot(ptA, ptB, ptC);
    return { id: 'pt#h', name: 'H', x: h.x, y: h.y };
  }, [ptA, ptB, ptC]);

  // Right-angle square marker at H (altitude)
  const altitudeRightAngle = useMemo(() => {
    return createRightAngleSquare(ptH, ptC, ptA, 11);
  }, [ptH, ptC, ptA]);

  // Angle Bisector Foot D on BC
  const ptD_bis = useMemo(() => {
    const d = computeAngleBisectorFoot(ptA, ptB, ptC);
    return { id: 'pt#d_bis', name: 'D', x: d.x, y: d.y };
  }, [ptA, ptB, ptC]);

  // Circle / Thales Parameters
  const circleCenter = useMemo<Point2D>(() => ({
    x: (ptA.x + ptB.x) / 2,
    y: (ptA.y + ptB.y) / 2,
  }), [ptA, ptB]);

  const circleRadius = useMemo(() => {
    return vecDist(ptA, ptB) / 2;
  }, [ptA, ptB]);

  // Thales Right Angle Square at C
  const thalesRightAngle = useMemo(() => {
    return createRightAngleSquare(ptC, ptA, ptB, 13);
  }, [ptC, ptA, ptB]);

  // Dynamic Angle Arcs
  const angleArcA = useMemo(() => createAngleArc(ptA, ptB, ptC, 24), [ptA, ptB, ptC]);
  const angleArcB = useMemo(() => createAngleArc(ptB, ptA, ptC, 24), [ptB, ptA, ptC]);
  const angleArcC = useMemo(() => createAngleArc(ptC, ptA, ptB, 24), [ptC, ptA, ptB]);

  // Dynamic Segment Measurements
  const segAB = useMemo(() => createSegmentMeasurement(ptA, ptB, 14), [ptA, ptB]);
  const segBC = useMemo(() => createSegmentMeasurement(ptB, ptC, 14), [ptB, ptC]);
  const segAC = useMemo(() => createSegmentMeasurement(ptA, ptC, 14), [ptA, ptC]);

  // ==========================================
  // Pointer Events & Constraint Dragging
  // ==========================================

  const getSvgCoordinates = (e: React.PointerEvent): Point2D => {
    if (!svgRef.current) return { x: e.clientX, y: e.clientY };
    const ctm = svgRef.current.getScreenCTM();
    if (!ctm) return { x: e.clientX, y: e.clientY };
    const pt = svgRef.current.createSVGPoint();
    pt.x = e.clientX;
    pt.y = e.clientY;
    const transformed = pt.matrixTransform(ctm.inverse());
    return { x: transformed.x, y: transformed.y };
  };

  const handlePointerDown = (e: React.PointerEvent, id: string) => {
    e.stopPropagation();
    (e.target as Element).setPointerCapture(e.pointerId);
    setDraggingId(id);
  };

  const handlePointerMove = (e: React.PointerEvent) => {
    if (!draggingId || !svgRef.current) return;

    let target = getSvgCoordinates(e);

    // Grid snapping if enabled (snap to 10px / 20px increments)
    if (snapToGrid) {
      target = {
        x: Math.round(target.x / 10) * 10,
        y: Math.round(target.y / 10) * 10,
      };
    }

    const clamped = clampToViewport(target);

    // Apply Geometric Constraints
    if (isCircleFigure) {
      if (draggingId === ptC.id) {
        // Point C is constrained to circle circumference (Thales theorem)
        const proj = projectToCircle(clamped, circleCenter, circleRadius, true);
        onUpdatePoint(draggingId, proj.x, proj.y);
      } else if (draggingId === ptA.id || draggingId === ptB.id) {
        // Dragging diameter endpoint A or B: update endpoint, preserve C on new circumference
        onUpdatePoint(draggingId, clamped.x, clamped.y);
        const newA = draggingId === ptA.id ? clamped : ptA;
        const newB = draggingId === ptB.id ? clamped : ptB;
        const newCenter = { x: (newA.x + newB.x) / 2, y: (newA.y + newB.y) / 2 };
        const newRadius = vecDist(newA, newB) / 2;
        const updatedC = projectToCircle(ptC, newCenter, newRadius, true);
        onUpdatePoint(ptC.id, updatedC.x, updatedC.y);
      } else {
        onUpdatePoint(draggingId, clamped.x, clamped.y);
      }
    } else if (hasSideEq_AB_AC) {
      if (draggingId === ptA.id) {
        // Dragging apex A of isosceles triangle: project onto perpendicular bisector of BC
        const projA = projectToPerpendicularBisector(clamped, ptB, ptC);
        const safeA = clampToViewport(projA);
        onUpdatePoint(draggingId, safeA.x, safeA.y);
      } else {
        // Dragging base vertex B or C: allow movement, re-center apex A to preserve AB = AC
        onUpdatePoint(draggingId, clamped.x, clamped.y);
        const updatedB = draggingId === ptB.id ? clamped : ptB;
        const updatedC = draggingId === ptC.id ? clamped : ptC;
        const adjustedA = projectToPerpendicularBisector(ptA, updatedB, updatedC);
        onUpdatePoint(ptA.id, adjustedA.x, adjustedA.y);
      }
    } else if (isQuadFigure && draggingId === ptD.id && lowerCode.includes('parallelogram')) {
      // Constrain D for parallelogram
      const pD = computeParallelogramD(ptA, ptB, ptC);
      onUpdatePoint(draggingId, pD.x, pD.y);
    } else {
      // Free point movement
      onUpdatePoint(draggingId, clamped.x, clamped.y);
    }
  };

  const handlePointerUp = (e: React.PointerEvent) => {
    if (draggingId) {
      try {
        (e.target as Element).releasePointerCapture(e.pointerId);
      } catch (_) {}
      setDraggingId(null);
    }
  };

  const handleEntityClick = (entityName: string) => {
    if (!onSelectStep) return;
    const matchingStep = steps.find(s => s.entities && s.entities.includes(entityName));
    if (matchingStep) {
      onSelectStep(matchingStep.id);
    }
  };

  const activePoint = points.find(p => p.id === draggingId);

  return (
    <div className="panel canvas-panel" onPointerUp={handlePointerUp}>
      {/* Precision CAD Header */}
      <div className="panel-header canvas-header">
        <div className="canvas-header-left">
          <span className="canvas-title">GEOMETRIC WORKSPACE</span>
          <span className="canvas-mode-tag">
            {isCircleFigure ? 'Circle (Thales)' : isQuadFigure ? 'Quadrilateral' : 'Triangle Plane'}
          </span>
          <button
            className={`btn btn-xs ${snapToGrid ? 'btn-active' : 'btn-ghost'}`}
            style={{
              fontSize: '10px',
              padding: '2px 6px',
              background: snapToGrid ? '#1c2638' : 'transparent',
              border: '1px solid #24324a',
              borderRadius: '3px',
              color: snapToGrid ? '#38bdf8' : '#64748b',
              cursor: 'pointer',
              marginLeft: '8px',
            }}
            onClick={() => setSnapToGrid(prev => !prev)}
            title="Toggle CAD Grid Snapping (10px)"
          >
            {snapToGrid ? 'Snap ON' : 'Snap OFF'}
          </button>
        </div>

        <div className="canvas-telemetry">
          <span className={`coord-chip ${draggingId === ptA.id ? 'active' : ''}`}>
            A({Math.round(ptA.x)}, {Math.round(ptA.y)})
          </span>
          <span className={`coord-chip ${draggingId === ptB.id ? 'active' : ''}`}>
            B({Math.round(ptB.x)}, {Math.round(ptB.y)})
          </span>
          <span className={`coord-chip ${draggingId === ptC.id ? 'active' : ''}`}>
            C({Math.round(ptC.x)}, {Math.round(ptC.y)})
          </span>
          {isQuadFigure && (
            <span className={`coord-chip ${draggingId === ptD.id ? 'active' : ''}`}>
              D({Math.round(ptD.x)}, {Math.round(ptD.y)})
            </span>
          )}
        </div>
      </div>

      <div className="canvas-viewport" style={{ touchAction: 'none' }}>
        <svg
          ref={svgRef}
          className="geometry-svg"
          viewBox="0 0 600 450"
          onPointerMove={handlePointerMove}
          onPointerUp={handlePointerUp}
          onClick={() => onSelectStep && onSelectStep(null)}
          style={{ userSelect: 'none' }}
        >
          {/* Subtle drafting grid */}
          <defs>
            <pattern id="cad-grid-sm" width="20" height="20" patternUnits="userSpaceOnUse">
              <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#161f30" strokeWidth="0.5" />
            </pattern>
            <pattern id="cad-grid-lg" width="100" height="100" patternUnits="userSpaceOnUse">
              <rect width="100" height="100" fill="url(#cad-grid-sm)" />
              <path d="M 100 0 L 0 0 0 100" fill="none" stroke="#212e45" strokeWidth="1" />
            </pattern>
          </defs>

          <rect width="100%" height="100%" fill="#0a0e17" />
          <rect width="100%" height="100%" fill="url(#cad-grid-lg)" />

          {/* Coordinate Crosshairs */}
          <line x1="0" y1="225" x2="600" y2="225" stroke="#1c283d" strokeWidth="1" strokeDasharray="3 3" />
          <line x1="300" y1="0" x2="300" y2="450" stroke="#1c283d" strokeWidth="1" strokeDasharray="3 3" />

          {/* Live Drag Crosshairs following active vertex */}
          {activePoint && (
            <g className="live-drag-crosshair" opacity="0.4">
              <line x1="0" y1={activePoint.y} x2="600" y2={activePoint.y} stroke="#38bdf8" strokeWidth="1" strokeDasharray="2 2" />
              <line x1={activePoint.x} y1="0" x2={activePoint.x} y2="450" stroke="#38bdf8" strokeWidth="1" strokeDasharray="2 2" />
            </g>
          )}

          {/* ===================== CIRCLE / THALES SCENE ===================== */}
          {isCircleFigure ? (
            <g className="circle-scene">
              {/* Circumcircle - Luminous Warm Amber with balanced optical weight */}
              <circle
                cx={circleCenter.x}
                cy={circleCenter.y}
                r={circleRadius}
                fill={isEntityActive('circle') ? 'rgba(245, 158, 11, 0.08)' : 'rgba(245, 158, 11, 0.035)'}
                stroke={isEntityActive('circle') ? '#fde047' : '#f59e0b'}
                strokeWidth={isEntityActive('circle') ? '2.5' : '1.8'}
                onClick={(e) => { e.stopPropagation(); handleEntityClick('circle'); }}
                style={{ cursor: 'pointer' }}
              />

              {/* Inscribed Triangle ABC Fill */}
              <polygon
                points={`${ptA.x},${ptA.y} ${ptC.x},${ptC.y} ${ptB.x},${ptB.y}`}
                fill="rgba(56, 189, 248, 0.06)"
                stroke="none"
              />

              {/* Diameter AB - Slate Silver Baseline */}
              <line
                x1={ptA.x}
                y1={ptA.y}
                x2={ptB.x}
                y2={ptB.y}
                stroke={isEntityActive('AB') || isEntityActive('diameter') ? '#388bfd' : '#94a3b8'}
                strokeWidth={isEntityActive('AB') || isEntityActive('diameter') ? '2.8' : '1.8'}
                onClick={(e) => { e.stopPropagation(); handleEntityClick('AB'); }}
                style={{ cursor: 'pointer' }}
              />

              {/* Inscribed Chords AC, BC - Cool Sky Blue */}
              <line
                x1={ptA.x}
                y1={ptA.y}
                x2={ptC.x}
                y2={ptC.y}
                stroke={isEntityActive('AC') ? '#388bfd' : '#38bdf8'}
                strokeWidth={isEntityActive('AC') ? '3' : '2'}
                onClick={(e) => { e.stopPropagation(); handleEntityClick('AC'); }}
                style={{ cursor: 'pointer' }}
              />
              <line
                x1={ptB.x}
                y1={ptB.y}
                x2={ptC.x}
                y2={ptC.y}
                stroke={isEntityActive('BC') ? '#388bfd' : '#38bdf8'}
                strokeWidth={isEntityActive('BC') ? '3' : '2'}
                onClick={(e) => { e.stopPropagation(); handleEntityClick('BC'); }}
                style={{ cursor: 'pointer' }}
              />

              {/* Inscribed 90° Right Angle Marker at C */}
              <polyline
                points={thalesRightAngle.pointsString}
                fill="rgba(56, 189, 248, 0.18)"
                stroke="#38bdf8"
                strokeWidth="1.8"
                onClick={(e) => { e.stopPropagation(); handleEntityClick('angle_ACB'); }}
                style={{ cursor: 'pointer' }}
              />

              {/* Center point O */}
              <circle cx={circleCenter.x} cy={circleCenter.y} r="3.5" fill="#f59e0b" stroke="#0a0e17" strokeWidth="1" />
              <text x={circleCenter.x} y={circleCenter.y + 14} fill="#fbbf24" fontSize="10" fontFamily="monospace" textAnchor="middle" fontWeight="bold">O</text>

              {/* Live 90° Badge at C */}
              <g transform={`translate(${ptC.x}, ${ptC.y - 18})`}>
                <rect x="-18" y="-12" width="36" height="15" rx="3" fill="#0d1117" stroke="#38bdf8" strokeWidth="1" />
                <text x="0" y="-1" fill="#38bdf8" fontSize="10" fontFamily="Inter, sans-serif" fontWeight="700" textAnchor="middle">
                  90°
                </text>
              </g>

              {/* Dynamic Angle Arcs at A and B */}
              <path d={angleArcA.pathD} fill="none" stroke="#64748b" strokeWidth="1.2" />
              <text x={angleArcA.labelPos.x} y={angleArcA.labelPos.y} fill="#94a3b8" fontSize="9" fontFamily="monospace" textAnchor="middle">
                {angleArcA.sweepAngleDeg}°
              </text>
              <path d={angleArcB.pathD} fill="none" stroke="#64748b" strokeWidth="1.2" />
              <text x={angleArcB.labelPos.x} y={angleArcB.labelPos.y} fill="#94a3b8" fontSize="9" fontFamily="monospace" textAnchor="middle">
                {angleArcB.sweepAngleDeg}°
              </text>
            </g>
          ) : isQuadFigure ? (
            /* ===================== QUADRILATERAL SCENE ===================== */
            <g className="quad-scene">
              <polygon
                points={`${ptA.x},${ptA.y} ${ptB.x},${ptB.y} ${ptC.x},${ptC.y} ${ptD.x},${ptD.y}`}
                fill="rgba(56, 189, 248, 0.04)"
                stroke="#58a6ff"
                strokeWidth="2"
              />
              <line x1={ptA.x} y1={ptA.y} x2={ptC.x} y2={ptC.y} stroke="#30363d" strokeWidth="1" strokeDasharray="4 4" />
              <line x1={ptB.x} y1={ptB.y} x2={ptD.x} y2={ptD.y} stroke="#30363d" strokeWidth="1" strokeDasharray="4 4" />
            </g>
          ) : (
            /* ===================== TRIANGLE PLANE SCENE ===================== */
            <g className="triangle-scene">
              {/* Triangle Polygon ABC */}
              <polygon
                points={`${ptA.x},${ptA.y} ${ptB.x},${ptB.y} ${ptC.x},${ptC.y}`}
                fill="rgba(56, 189, 248, 0.04)"
                stroke={isEntityActive('triangle') ? '#58a6ff' : '#388bfd'}
                strokeWidth="2"
              />

              {/* Segment AB */}
              <line
                x1={ptA.x}
                y1={ptA.y}
                x2={ptB.x}
                y2={ptB.y}
                stroke={isEntityActive('AB') ? '#388bfd' : '#79c0ff'}
                strokeWidth={isEntityActive('AB') ? '3.5' : '2'}
                onClick={(e) => { e.stopPropagation(); handleEntityClick('AB'); }}
              />
              {/* Segment AC */}
              <line
                x1={ptA.x}
                y1={ptA.y}
                x2={ptC.x}
                y2={ptC.y}
                stroke={isEntityActive('AC') ? '#388bfd' : '#79c0ff'}
                strokeWidth={isEntityActive('AC') ? '3.5' : '2'}
                onClick={(e) => { e.stopPropagation(); handleEntityClick('AC'); }}
              />
              {/* Base BC */}
              <line
                x1={ptB.x}
                y1={ptB.y}
                x2={ptC.x}
                y2={ptC.y}
                stroke={isEntityActive('BC') ? '#388bfd' : '#e6edf3'}
                strokeWidth={isEntityActive('BC') ? '3.5' : '2'}
                onClick={(e) => { e.stopPropagation(); handleEntityClick('BC'); }}
              />

              {/* Dynamic Segment Length Chips */}
              <g className="segment-measurements" opacity="0.8">
                <g transform={`translate(${segAB.labelPos.x}, ${segAB.labelPos.y})`}>
                  <rect x="-12" y="-7" width="24" height="13" rx="2" fill="#0d1117" stroke="#24324a" strokeWidth="0.6" />
                  <text x="0" y="2.5" fill="#79c0ff" fontSize="8" fontFamily="monospace" textAnchor="middle">
                    {Math.round(segAB.length)}
                  </text>
                </g>
                <g transform={`translate(${segAC.labelPos.x}, ${segAC.labelPos.y})`}>
                  <rect x="-12" y="-7" width="24" height="13" rx="2" fill="#0d1117" stroke="#24324a" strokeWidth="0.6" />
                  <text x="0" y="2.5" fill="#79c0ff" fontSize="8" fontFamily="monospace" textAnchor="middle">
                    {Math.round(segAC.length)}
                  </text>
                </g>
                <g transform={`translate(${segBC.labelPos.x}, ${segBC.labelPos.y})`}>
                  <rect x="-12" y="-7" width="24" height="13" rx="2" fill="#0d1117" stroke="#24324a" strokeWidth="0.6" />
                  <text x="0" y="2.5" fill="#e6edf3" fontSize="8" fontFamily="monospace" textAnchor="middle">
                    {Math.round(segBC.length)}
                  </text>
                </g>
              </g>

              {/* Equilateral/Isosceles Side Tick Marks on AB and AC */}
              {hasSideEq_AB_AC && (
                <>
                  <line
                    x1={segAB.midpoint.x - 4 * segAB.normalUnit.x}
                    y1={segAB.midpoint.y - 4 * segAB.normalUnit.y}
                    x2={segAB.midpoint.x + 4 * segAB.normalUnit.x}
                    y2={segAB.midpoint.y + 4 * segAB.normalUnit.y}
                    stroke="#58a6ff"
                    strokeWidth="2"
                  />
                  <line
                    x1={segAC.midpoint.x - 4 * segAC.normalUnit.x}
                    y1={segAC.midpoint.y - 4 * segAC.normalUnit.y}
                    x2={segAC.midpoint.x + 4 * segAC.normalUnit.x}
                    y2={segAC.midpoint.y + 4 * segAC.normalUnit.y}
                    stroke="#58a6ff"
                    strokeWidth="2"
                  />
                </>
              )}

              {/* Dynamic Angle Arcs & Degree Chips at B and C */}
              <path d={angleArcB.pathD} fill="none" stroke="#58a6ff" strokeWidth="1.5" />
              <g transform={`translate(${angleArcB.labelPos.x}, ${angleArcB.labelPos.y})`}>
                <rect x="-14" y="-8" width="28" height="15" rx="3" fill="#0d1117" stroke="#24324a" strokeWidth="0.8" />
                <text x="0" y="3" fill="#58a6ff" fontSize="9" fontFamily="monospace" fontWeight="bold" textAnchor="middle">
                  {angleArcB.sweepAngleDeg}°
                </text>
              </g>

              <path d={angleArcC.pathD} fill="none" stroke="#58a6ff" strokeWidth="1.5" />
              <g transform={`translate(${angleArcC.labelPos.x}, ${angleArcC.labelPos.y})`}>
                <rect x="-14" y="-8" width="28" height="15" rx="3" fill="#0d1117" stroke="#24324a" strokeWidth="0.8" />
                <text x="0" y="3" fill="#58a6ff" fontSize="9" fontFamily="monospace" fontWeight="bold" textAnchor="middle">
                  {angleArcC.sweepAngleDeg}°
                </text>
              </g>

              {/* Apex Angle Arc at A */}
              <path d={angleArcA.pathD} fill="none" stroke="#94a3b8" strokeWidth="1.2" />
              <g transform={`translate(${angleArcA.labelPos.x}, ${angleArcA.labelPos.y})`}>
                <rect x="-14" y="-8" width="28" height="15" rx="3" fill="#0d1117" stroke="#24324a" strokeWidth="0.8" />
                <text x="0" y="3" fill="#94a3b8" fontSize="9" fontFamily="monospace" textAnchor="middle">
                  {angleArcA.sweepAngleDeg}°
                </text>
              </g>

              {/* Midpoint M Construction */}
              {hasMidpoint && (
                <g className="construction-midpoint">
                  <line
                    x1={ptA.x}
                    y1={ptA.y}
                    x2={ptM.x}
                    y2={ptM.y}
                    stroke={isEntityActive('M') ? '#388bfd' : '#58a6ff'}
                    strokeWidth="2"
                    strokeDasharray="4 3"
                  />
                  <circle cx={ptM.x} cy={ptM.y} r="5" fill="#0d1117" stroke="#58a6ff" strokeWidth="2" />
                  <text x={ptM.x + 8} y={ptM.y + 14} fill="#58a6ff" fontSize="11" fontWeight="bold">M</text>
                </g>
              )}

              {/* Altitude Foot H Construction */}
              {hasAltitude && (
                <g className="construction-altitude">
                  <line
                    x1={ptA.x}
                    y1={ptA.y}
                    x2={ptH.x}
                    y2={ptH.y}
                    stroke={isEntityActive('H') ? '#388bfd' : '#3fb950'}
                    strokeWidth="2"
                    strokeDasharray="4 3"
                  />
                  <polyline
                    points={altitudeRightAngle.pointsString}
                    fill="none"
                    stroke="#3fb950"
                    strokeWidth="1.5"
                  />
                  <circle cx={ptH.x} cy={ptH.y} r="5" fill="#0d1117" stroke="#3fb950" strokeWidth="2" />
                  <text x={ptH.x + 8} y={ptH.y + 14} fill="#3fb950" fontSize="11" fontWeight="bold">H</text>
                </g>
              )}

              {/* Angle Bisector D Construction */}
              {hasBisector && (
                <g className="construction-bisector">
                  <line
                    x1={ptA.x}
                    y1={ptA.y}
                    x2={ptD_bis.x}
                    y2={ptD_bis.y}
                    stroke={isEntityActive('D') ? '#388bfd' : '#d29922'}
                    strokeWidth="2"
                    strokeDasharray="4 3"
                  />
                  <circle cx={ptD_bis.x} cy={ptD_bis.y} r="5" fill="#0d1117" stroke="#d29922" strokeWidth="2" />
                  <text x={ptD_bis.x + 8} y={ptD_bis.y + 14} fill="#d29922" fontSize="11" fontWeight="bold">D</text>
                </g>
              )}
            </g>
          )}

          {/* ===================== DRAGGABLE VERTEX HANDLES A, B, C, D ===================== */}
          {(isQuadFigure ? [ptA, ptB, ptC, ptD] : [ptA, ptB, ptC]).map((pt) => {
            const isPtActive = isEntityActive(pt.name) || draggingId === pt.id;
            const isHovered = hoveredPointId === pt.id;

            return (
              <g
                key={pt.id}
                transform={`translate(${pt.x}, ${pt.y})`}
                onPointerDown={(e) => handlePointerDown(e, pt.id)}
                onPointerEnter={() => setHoveredPointId(pt.id)}
                onPointerLeave={() => setHoveredPointId(null)}
                onClick={(e) => { e.stopPropagation(); handleEntityClick(pt.name); }}
                style={{ cursor: draggingId === pt.id ? 'grabbing' : 'grab' }}
              >
                {/* Touch/Mouse Target Halo */}
                <circle r={18} fill="transparent" />

                {/* Outer Ring on Hover or Drag */}
                {(isHovered || isPtActive) && (
                  <circle
                    r={12}
                    fill="none"
                    stroke="#38bdf8"
                    strokeWidth="1.5"
                    strokeDasharray="3 2"
                    opacity={isPtActive ? 1 : 0.6}
                  />
                )}

                {/* Clean technical node */}
                <circle
                  r={isPtActive ? 8 : 6}
                  fill="#0d1117"
                  stroke={isPtActive ? '#38bdf8' : '#e6edf3'}
                  strokeWidth={isPtActive ? 3 : 2}
                />
                <circle r={isPtActive ? 4 : 2.5} fill={isPtActive ? '#38bdf8' : '#e6edf3'} />

                {/* Mathematical Label */}
                <g transform="translate(10, -8)">
                  <rect x="-3" y="-12" width="16" height="15" rx="3" fill="#161b22" stroke="#30363d" strokeWidth="0.8" />
                  <text
                    x="0"
                    y="0"
                    fill="#e6edf3"
                    fontSize="11"
                    fontFamily="Inter, sans-serif"
                    fontWeight="600"
                    fontStyle="italic"
                  >
                    {pt.name}
                  </text>
                </g>
              </g>
            );
          })}
        </svg>
      </div>
    </div>
  );
};

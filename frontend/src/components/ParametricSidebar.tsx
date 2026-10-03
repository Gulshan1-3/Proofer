import React from 'react';
import { Sliders, Check, AlertCircle, Compass } from 'lucide-react';

interface ParametricSidebarProps {
  apexAngle: number;
  onApexAngleChange: (val: number) => void;
  sideLength: number;
  onSideLengthChange: (val: number) => void;
  baseAngle: number;
}

export const ParametricSidebar: React.FC<ParametricSidebarProps> = ({
  apexAngle,
  onApexAngleChange,
  sideLength,
  onSideLengthChange,
  baseAngle,
}) => {
  // Law of Cosines: a^2 = b^2 + c^2 - 2bc*cos(A)
  const radA = (apexAngle * Math.PI) / 180;
  const baseLength = Math.sqrt(
    sideLength ** 2 + sideLength ** 2 - 2 * sideLength * sideLength * Math.cos(radA)
  );

  const satisfiesTriangleInequality = sideLength + sideLength > baseLength;
  const satisfiesAngleSum = Math.abs(apexAngle + 2 * baseAngle - 180) < 0.1;

  return (
    <div className="parametric-pane">
      <div className="param-header">
        <Sliders size={12} className="param-header-icon" />
        <span className="param-header-title">CONTINUOUS SOLVER & DEFORMATION</span>
      </div>

      {/* Apex Angle Slider */}
      <div className="param-control-group">
        <div className="param-label-row">
          <span className="param-label">Apex Angle ∠A</span>
          <span className="param-val-pill">{apexAngle}°</span>
        </div>
        <input
          type="range"
          min="30"
          max="120"
          value={apexAngle}
          onChange={(e) => onApexAngleChange(Number(e.target.value))}
          className="param-range-slider"
        />
        <div className="param-subtext">Drag to continuously deform triangle vertex A</div>
      </div>

      {/* Base Angle Readout */}
      <div className="param-control-group">
        <div className="param-label-row">
          <span className="param-label">Base Angles ∠B = ∠C</span>
          <span className="param-val-pill">{baseAngle.toFixed(1)}°</span>
        </div>
        <div className="param-subtext formula-font">
          Enforced by symmetry: (180° - {apexAngle}°) / 2
        </div>
      </div>

      {/* Side Length Slider */}
      <div className="param-control-group">
        <div className="param-label-row">
          <span className="param-label">Leg Length AB = AC</span>
          <span className="param-val-pill">{sideLength.toFixed(1)} cm</span>
        </div>
        <input
          type="range"
          min="4"
          max="16"
          step="0.5"
          value={sideLength}
          onChange={(e) => onSideLengthChange(Number(e.target.value))}
          className="param-range-slider"
        />
      </div>

      {/* Derived Base Length BC (Law of Cosines) */}
      <div className="param-control-group">
        <div className="param-label-row">
          <span className="param-label">Base Length BC</span>
          <span className="param-val-pill">{baseLength.toFixed(2)} cm</span>
        </div>
        <div className="param-subtext formula-font">
          BC = √(AB² + AC² - 2·AB·AC·cos(A))
        </div>
      </div>

      {/* Constraint Solver Verification */}
      <div className="euclidean-constraints-box">
        <div className="constraints-title">
          <Compass size={11} />
          <span>EUCLIDEAN CONSTRAINTS STATUS</span>
        </div>

        <div className={`constraint-row ${satisfiesAngleSum ? 'satisfied' : 'violated'}`}>
          {satisfiesAngleSum ? <Check size={11} strokeWidth={3} /> : <AlertCircle size={11} />}
          <span>Angle Sum: ∠A + 2∠B = 180°</span>
        </div>

        <div className={`constraint-row ${satisfiesTriangleInequality ? 'satisfied' : 'violated'}`}>
          {satisfiesTriangleInequality ? <Check size={11} strokeWidth={3} /> : <AlertCircle size={11} />}
          <span>Triangle Inequality: 2·AB &gt; BC</span>
        </div>
      </div>
    </div>
  );
};

import React from 'react';
import { Divide, Compass, CornerDownRight, RotateCcw, Binary, Sparkles } from 'lucide-react';

interface ConstructionToolbarProps {
  onApplyPatch: (patchText: string) => void;
  onRemovePatch: (substrings: string[]) => void;
  onResetPoints: () => void;
  onSynthesizeStep?: () => void;
  hasMidpoint: boolean;
  hasAltitude: boolean;
  hasBisector: boolean;
  isGeometry: boolean;
  isSynthesizing?: boolean;
}

export const ConstructionToolbar: React.FC<ConstructionToolbarProps> = ({
  onApplyPatch,
  onRemovePatch,
  onResetPoints,
  onSynthesizeStep,
  hasMidpoint,
  hasAltitude,
  hasBisector,
  isGeometry,
  isSynthesizing = false,
}) => {
  const handleMidpoint = () => {
    if (hasMidpoint) {
      onRemovePatch(['construct M as midpoint', 'triangle ABM congruent triangle ACM', 'midpoint of BC']);
    } else {
      const patch = `    construct M as midpoint of BC\n    have h3 : triangle ABM congruent triangle ACM from h1 using SSS\n`;
      onApplyPatch(patch);
    }
  };

  const handleAltitude = () => {
    if (hasAltitude) {
      onRemovePatch(['construct H as altitude', 'perpendicular(line_AH, line_BC)', 'line_AH perpendicular line_BC', 'altitude of BC']);
    } else {
      const patch = `    construct H as altitude of BC\n    have h_alt : perpendicular(line_AH, line_BC)\n`;
      onApplyPatch(patch);
    }
  };

  const handleBisector = () => {
    if (hasBisector) {
      onRemovePatch(['construct D as bisector', 'angle_BAD = angle_CAD', 'bisector of angle_BAC']);
    } else {
      const patch = `    construct D as bisector of angle_BAC\n    have h_bis : angle_BAD = angle_CAD\n`;
      onApplyPatch(patch);
    }
  };

  if (!isGeometry) {
    return (
      <div className="workstation-subbar logic-mode">
        <div className="subbar-left">
          <Binary size={13} className="subbar-icon" />
          <span className="subbar-tag">PROPOSITIONAL LOGIC MODE</span>
          <span className="subbar-desc">Spatial canvas collapsed for non-geometric deductive proof</span>
        </div>
        <div className="subbar-right">
          {onSynthesizeStep && (
            <button
              className="construct-btn synth-btn"
              onClick={onSynthesizeStep}
              disabled={isSynthesizing}
              title="Deterministic kernel-verified proof synthesizer (AI Co-Prover)"
              style={{ background: 'rgba(56, 189, 248, 0.15)', borderColor: 'rgba(56, 189, 248, 0.4)' }}
            >
              <Sparkles size={12} style={{ color: '#38bdf8' }} />
              <span style={{ color: '#38bdf8', fontWeight: 600 }}>{isSynthesizing ? 'Synthesizing...' : 'Infill Step (Co-Prover)'}</span>
            </button>
          )}
        </div>
      </div>
    );
  }

  return (
    <div className="workstation-subbar">
      <div className="subbar-left">
        <span className="subbar-section-title">CONSTRUCT:</span>

        <button
          className={`construct-btn ${hasMidpoint ? 'active' : ''}`}
          onClick={handleMidpoint}
          title="Construct midpoint M on BC with median AM"
        >
          <Divide size={12} />
          <span>Midpoint (M)</span>
        </button>

        <button
          className={`construct-btn ${hasAltitude ? 'active' : ''}`}
          onClick={handleAltitude}
          title="Drop perpendicular altitude AH from apex A to base BC"
        >
          <CornerDownRight size={12} />
          <span>Altitude (AH)</span>
        </button>

        <button
          className={`construct-btn ${hasBisector ? 'active' : ''}`}
          onClick={handleBisector}
          title="Construct ray AD bisecting angle BAC"
        >
          <Compass size={12} />
          <span>Angle Bisector (AD)</span>
        </button>
      </div>

      <div className="subbar-right">
        {onSynthesizeStep && (
          <button
            className="construct-btn synth-btn"
            onClick={onSynthesizeStep}
            disabled={isSynthesizing}
            title="Deterministic kernel-verified proof synthesizer (AI Co-Prover)"
            style={{ background: 'rgba(56, 189, 248, 0.15)', borderColor: 'rgba(56, 189, 248, 0.4)' }}
          >
            <Sparkles size={12} style={{ color: '#38bdf8' }} />
            <span style={{ color: '#38bdf8', fontWeight: 600 }}>{isSynthesizing ? 'Synthesizing...' : 'Infill Step (Co-Prover)'}</span>
          </button>
        )}
        <button
          className="construct-btn reset-btn"
          onClick={onResetPoints}
          title="Reset vertex coordinates to canonical positions"
        >
          <RotateCcw size={12} />
          <span>Reset Geometry</span>
        </button>
      </div>
    </div>
  );
};

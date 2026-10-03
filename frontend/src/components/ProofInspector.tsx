import React from 'react';
import { ShieldCheck, ShieldAlert, Check, X, CornerDownRight, Cpu, Layers } from 'lucide-react';
import { ProofStep } from '../types';
import { formatMathFormula } from '../services/stepMapper';

interface ProofInspectorProps {
  steps: ProofStep[];
  selectedStepId: number | null;
  onSelectStep: (stepId: number | null) => void;
  theoremProven: string;
  theoremName?: string;
  isVerified?: boolean;
}

export const ProofInspector: React.FC<ProofInspectorProps> = ({
  steps,
  selectedStepId,
  onSelectStep,
  theoremProven,
  theoremName = 'theorem',
  isVerified = false,
}) => {
  const formattedGoal = formatMathFormula(theoremProven);

  const getVerb = (step: ProofStep): { label: string; kindClass: string } => {
    if (step.kind === 'suppose' || step.text.startsWith('suppose')) {
      return { label: 'SUPPOSE', kindClass: 'verb-suppose' };
    }
    if (step.kind === 'derive' || step.text.startsWith('derive')) {
      return { label: 'DERIVE', kindClass: 'verb-derive' };
    }
    if (step.kind === 'construct' || step.text.startsWith('construct')) {
      return { label: 'CONSTRUCT', kindClass: 'verb-construct' };
    }
    if (step.kind === 'have' || step.text.startsWith('have')) {
      return { label: 'HAVE', kindClass: 'verb-have' };
    }
    if (step.kind === 'therefore' || step.text.startsWith('therefore')) {
      return { label: 'THEREFORE', kindClass: 'verb-therefore' };
    }
    return { label: 'STEP', kindClass: 'verb-default' };
  };

  return (
    <div className="proof-debugger-pane">
      {/* Target Theorem Specification */}
      <div className="debugger-target-section">
        <div className="target-header">
          <div className="target-title-wrap">
            <span className="target-kicker">TARGET THEOREM</span>
            <span className="target-name">{theoremName}</span>
          </div>
          <span className={`kernel-badge ${isVerified ? 'verified' : 'pending'}`}>
            {isVerified ? (
              <>
                <ShieldCheck size={12} />
                <span>Q.E.D. VERIFIED</span>
              </>
            ) : (
              <>
                <ShieldAlert size={12} />
                <span>INCOMPLETE / REJECTED</span>
              </>
            )}
          </span>
        </div>

        {formattedGoal && (
          <div className="target-formula-box">
            <span className="formula-prefix">⊢</span>
            <span className="formula-content">{formattedGoal}</span>
          </div>
        )}
      </div>

      {/* Execution Trace Timeline */}
      <div className="debugger-trace-header">
        <div className="trace-title">
          <Layers size={12} />
          <span>INFERENCE EXECUTION TRACE</span>
        </div>
        <span className="trace-counter">{steps.length} STEPS</span>
      </div>

      <div className="debugger-trace-timeline">
        {steps.length === 0 ? (
          <div className="debugger-empty-state">
            <Cpu size={20} className="empty-icon" />
            <div className="empty-title">No proof steps parsed</div>
            <div className="empty-subtitle">Write formal proof steps in the editor to inspect execution trace.</div>
          </div>
        ) : (
          steps.map((step, idx) => {
            const isSelected = selectedStepId === step.id;
            const isValid = step.status === 'Valid';
            const { label: verbLabel, kindClass } = getVerb(step);
            const stepNum = String(idx + 1).padStart(2, '0');

            // Format proposition and hypothesis
            let formulaDisplay = step.conclusion || step.text;
            if (step.hypothesis) {
              formulaDisplay = `${step.hypothesis} : ${step.conclusion || ''}`;
            }
            const formattedFormula = formatMathFormula(formulaDisplay);

            return (
              <div
                key={step.id}
                className={`trace-step-item ${isSelected ? 'active-step' : ''} ${!isValid ? 'rejected-step' : ''}`}
                onClick={() => onSelectStep(isSelected ? null : step.id)}
              >
                {/* Active Trace Pointer */}
                <div className="step-left-rail">
                  <span className={`step-dot ${isValid ? 'valid' : 'rejected'}`}>
                    {isValid ? <Check size={9} strokeWidth={3} /> : <X size={9} strokeWidth={3} />}
                  </span>
                  {idx < steps.length - 1 && <span className="trace-vertical-line" />}
                </div>

                <div className="step-content-body">
                  <div className="step-meta-row">
                    <span className="step-index">{stepNum}</span>
                    <span className={`step-verb-badge ${kindClass}`}>{verbLabel}</span>

                    {step.lineNumber && (
                      <span className="step-line-tag">ln {step.lineNumber}</span>
                    )}

                    {isSelected && (
                      <span className="step-selected-indicator">▶ INSPECTING</span>
                    )}
                  </div>

                  {/* Mathematical Proposition */}
                  <div className="step-formula">
                    {formattedFormula}
                  </div>

                  {/* Inference Rule Attribution & Premises */}
                  {(step.rule || (step.dependencies && step.dependencies.length > 0)) && (
                    <div className="step-attribution">
                      {step.rule && (
                        <div className="attribution-rule">
                          <span className="attr-label">rule:</span>
                          <span className="attr-value rule-name">{step.rule}</span>
                        </div>
                      )}
                      {step.dependencies && step.dependencies.length > 0 && (
                        <div className="attribution-premise">
                          <CornerDownRight size={11} className="premise-icon" />
                          <span className="attr-label">premise:</span>
                          {step.dependencies.map((dep, dIdx) => (
                            <span key={dIdx} className="attr-value premise-chip">{dep}</span>
                          ))}
                        </div>
                      )}
                    </div>
                  )}

                  {/* Referenced Geometric Objects */}
                  {step.entities && step.entities.length > 0 && (
                    <div className="step-entities-row">
                      <span className="entity-label">geometry:</span>
                      {step.entities.slice(0, 4).map((ent, eIdx) => (
                        <span key={eIdx} className="entity-chip">{ent}</span>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            );
          })
        )}

        {/* Q.E.D. Milestone */}
        {isVerified && steps.length > 0 && (
          <div className="qed-milestone">
            <div className="qed-badge">
              <Check size={12} strokeWidth={3} />
              <span>Q.E.D. — THEOREM FULLY DISCHARGED</span>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

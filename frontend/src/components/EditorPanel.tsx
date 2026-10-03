import React, { useRef, useEffect } from 'react';
import { Code2, AlertTriangle, CheckCircle2, Terminal } from 'lucide-react';
import { VerificationResponse, ProofStep } from '../types';

interface EditorPanelProps {
  code: string;
  onChange: (newCode: string) => void;
  verification: VerificationResponse | null;
  activeLine?: number | null;
  selectedStepId?: number | null;
  steps?: ProofStep[];
  onSelectStep?: (stepId: number | null) => void;
}

export const EditorPanel: React.FC<EditorPanelProps> = ({
  code,
  onChange,
  verification,
  activeLine,
  selectedStepId,
  steps = [],
  onSelectStep,
}) => {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const highlightRef = useRef<HTMLPreElement>(null);
  const lines = code.split('\n');
  const lineCount = lines.length;
  const isVerified = verification?.verified ?? false;
  const errors = verification?.theorems?.[0]?.errors ?? [];

  // Determine which line is currently active
  const activeStep = steps.find(s => s.id === selectedStepId);
  const effectiveActiveLine = activeLine || activeStep?.lineNumber || null;

  // Sync scrolling between textarea and syntax highlight backdrop
  const handleScroll = (e: React.UIEvent<HTMLTextAreaElement>) => {
    if (highlightRef.current) {
      highlightRef.current.scrollTop = e.currentTarget.scrollTop;
      highlightRef.current.scrollLeft = e.currentTarget.scrollLeft;
    }
  };

  // When active line changes from external step selection, scroll it into view if needed
  useEffect(() => {
    if (effectiveActiveLine && textareaRef.current) {
      const lineHeight = 20; // px
      const targetScroll = (effectiveActiveLine - 4) * lineHeight;
      if (Math.abs(textareaRef.current.scrollTop - targetScroll) > 100) {
        textareaRef.current.scrollTo({ top: Math.max(0, targetScroll), behavior: 'smooth' });
      }
    }
  }, [effectiveActiveLine]);

  // Handle cursor movement to auto-sync active step
  const handleCursorActivity = () => {
    if (!textareaRef.current || !onSelectStep) return;
    const pos = textareaRef.current.selectionStart;
    const textBefore = code.slice(0, pos);
    const currentLine = textBefore.split('\n').length;

    const matchedStep = steps.find(s => s.lineNumber === currentLine);
    if (matchedStep && matchedStep.id !== selectedStepId) {
      onSelectStep(matchedStep.id);
    }
  };

  // Syntax highlighting tokenizer
  const renderHighlightedCode = () => {
    return lines.map((line, idx) => {
      const lineNum = idx + 1;
      const isLineActive = effectiveActiveLine === lineNum;

      // Tokenize line with disciplined technical styling
      const tokens = tokenizeLine(line);

      return (
        <div key={idx} className={`code-line-layer ${isLineActive ? 'active-code-line' : ''}`}>
          {tokens.length === 0 ? '\u00A0' : tokens}
        </div>
      );
    });
  };

  return (
    <div className="panel editor-panel">
      {/* Editor Title Bar */}
      <div className="panel-header editor-header">
        <div className="editor-title-left">
          <Code2 size={13} className="editor-icon" />
          <span className="editor-title">PROOF SCRIPT</span>
          <span className="editor-lang-tag">proofer</span>
        </div>

        <div className="editor-metrics">
          <span className="metric-item">{lineCount} lines</span>
          <span className="metric-sep">•</span>
          <span className="metric-item">UTF-8</span>
          <span className="metric-sep">•</span>
          {isVerified ? (
            <span className="editor-status-text verified">
              <CheckCircle2 size={11} /> Verified
            </span>
          ) : (
            <span className="editor-status-text pending">
              <Terminal size={11} /> Checking
            </span>
          )}
        </div>
      </div>

      {/* Editor Body with Line Numbers, Gutters, and Syntax Backdrop */}
      <div className="code-editor-layout">
        {/* Gutter with line numbers and verification/step indicators */}
        <div className="editor-gutter">
          {lines.map((_, i) => {
            const lineNum = i + 1;
            const matchingStep = steps.find(s => s.lineNumber === lineNum);
            const isLineActive = effectiveActiveLine === lineNum;
            const hasError = !isVerified && errors.length > 0 && i === lines.length - 2;

            return (
              <div
                key={i}
                className={`gutter-cell ${isLineActive ? 'active-gutter' : ''}`}
                onClick={() => matchingStep && onSelectStep && onSelectStep(matchingStep.id)}
              >
                <span className="gutter-line-num">{lineNum}</span>
                <span className="gutter-marker">
                  {isLineActive ? (
                    <span className="marker-active-arrow">▶</span>
                  ) : hasError ? (
                    <span className="marker-error">●</span>
                  ) : matchingStep?.status === 'Valid' ? (
                    <span className="marker-verified">●</span>
                  ) : null}
                </span>
              </div>
            );
          })}
        </div>

        {/* Highlight Layer & Textarea Stack */}
        <div className="code-stack-container">
          <pre ref={highlightRef} className="code-highlight-backdrop" aria-hidden="true">
            {renderHighlightedCode()}
          </pre>

          <textarea
            ref={textareaRef}
            className="code-textarea"
            value={code}
            onChange={(e) => onChange(e.target.value)}
            onScroll={handleScroll}
            onClick={handleCursorActivity}
            onKeyUp={handleCursorActivity}
            spellCheck={false}
            autoComplete="off"
            autoCorrect="off"
            autoCapitalize="off"
          />
        </div>
      </div>

      {/* Inline Diagnostic Drawer */}
      {!isVerified && errors.length > 0 && (
        <div className="editor-diagnostic-bar rejected">
          <AlertTriangle size={13} className="diag-icon" />
          <span className="diag-label">KERNEL DIAGNOSTIC:</span>
          <span className="diag-msg">{errors[0]}</span>
        </div>
      )}

      {isVerified && (
        <div className="editor-diagnostic-bar verified">
          <CheckCircle2 size={13} className="diag-icon" />
          <span className="diag-label">KERNEL:</span>
          <span className="diag-msg">Formal specification and inferences sound. Q.E.D.</span>
        </div>
      )}
    </div>
  );
};

// Tokenizer for Proofer formal language
function tokenizeLine(line: string): React.ReactNode[] {
  if (!line) return [];
  if (line.trim().startsWith('#')) {
    return [<span key="comment" className="tok-comment">{line}</span>];
  }

  // Regex patterns for language constructs
  const pattern = /(\b(?:theorem|proof|suppose|derive|construct|have|therefore|end|from|using)\b|\b(?:IsoscelesBaseAngles|AngleSum180|InscribedAngle|Thales|CyclicQuad|CyclicOppositeAngles|ParallelogramOppSides|OppositeSidesEqual|MidpointBisects|SSS|SAS|VerticalAngles)\b|\b(?:diameter|on_circle|midpoint|altitude|bisector|angle_[a-zA-Z0-9_]+|triangle_[a-zA-Z0-9_]+)\b|\b(?:h\d+|[A-D][A-D]|[A-D][MHD]|[MHD][A-D]|[A-D]|M|H)\b|->|:=|=|!=|<=|>=|\band\b|\bor\b|\bnot\b)/g;

  const nodes: React.ReactNode[] = [];
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = pattern.exec(line)) !== null) {
    if (match.index > lastIndex) {
      nodes.push(line.slice(lastIndex, match.index));
    }
    const token = match[0];

    if (/^(theorem|proof|suppose|derive|construct|have|therefore|end|from|using)$/.test(token)) {
      nodes.push(<span key={match.index} className="tok-keyword">{token}</span>);
    } else if (/^(IsoscelesBaseAngles|AngleSum180|InscribedAngle|Thales|CyclicQuad|CyclicOppositeAngles|ParallelogramOppSides|OppositeSidesEqual|MidpointBisects|SSS|SAS|VerticalAngles)$/.test(token)) {
      nodes.push(<span key={match.index} className="tok-rule">{token}</span>);
    } else if (/^(diameter|on_circle|midpoint|altitude|bisector|angle_|triangle_)/.test(token)) {
      nodes.push(<span key={match.index} className="tok-predicate">{token}</span>);
    } else if (/^h\d+$/.test(token)) {
      nodes.push(<span key={match.index} className="tok-hypothesis">{token}</span>);
    } else if (/^([A-D][A-D]|[A-D][MHD]|[MHD][A-D]|[A-D]|M|H)$/.test(token)) {
      nodes.push(<span key={match.index} className="tok-entity">{token}</span>);
    } else {
      nodes.push(<span key={match.index} className="tok-operator">{token}</span>);
    }

    lastIndex = pattern.lastIndex;
  }

  if (lastIndex < line.length) {
    nodes.push(line.slice(lastIndex));
  }

  return nodes;
}

import React, { useState } from 'react';
import { GitCommit, GitBranch, History, RotateCcw, ShieldCheck, ShieldAlert, X, Plus, Diff, ArrowRight } from 'lucide-react';
import { ProjectFile, FileCommit } from '../types';
import { parseProofCodeToSteps, formatMathFormula } from '../services/stepMapper';

interface VersionControlPanelProps {
  file: ProjectFile;
  onCommit: (message: string) => void;
  onCheckout: (commit: FileCommit) => void;
  onClose: () => void;
}

export const VersionControlPanel: React.FC<VersionControlPanelProps> = ({
  file,
  onCommit,
  onCheckout,
  onClose,
}) => {
  const [commitMessage, setCommitMessage] = useState('');
  const [selectedCommitId, setSelectedCommitId] = useState<string | null>(null);

  const handleCommitSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!commitMessage.trim()) return;
    onCommit(commitMessage.trim());
    setCommitMessage('');
  };

  const commits = file.commits || [];
  const currentSteps = parseProofCodeToSteps(file.content);

  const selectedCommit = commits.find(c => c.id === selectedCommitId) || commits[commits.length - 1];
  const commitSteps = selectedCommit ? parseProofCodeToSteps(selectedCommit.content) : [];

  // Compute Proof State Diff
  const currentRules = new Set(currentSteps.map(s => s.rule).filter(Boolean));
  const commitRules = new Set(commitSteps.map(s => s.rule).filter(Boolean));
  const addedRules = [...currentRules].filter(r => !commitRules.has(r));
  const removedRules = [...commitRules].filter(r => !currentRules.has(r));

  const currentHyps = currentSteps.filter(s => s.hypothesis);
  const commitHyps = commitSteps.filter(s => s.hypothesis);
  const newHypotheses = currentHyps.filter(ch => !commitHyps.some(sh => sh.hypothesis === ch.hypothesis));

  return (
    <div className="vcs-drawer">
      <div className="drawer-header">
        <div className="drawer-title-group">
          <GitBranch size={14} className="drawer-title-icon" />
          <span className="drawer-title">VERSION CONTROL</span>
          <span className="drawer-filename">{file.name}</span>
        </div>
        <button className="drawer-close-btn" onClick={onClose} title="Close (Esc)">
          <X size={14} />
        </button>
      </div>

      {/* Commit Snapshot Action */}
      <div className="vcs-commit-action-box">
        <form onSubmit={handleCommitSubmit}>
          <div className="vcs-section-label">RECORD PROOF MILESTONE SNAPSHOT</div>
          <div className="commit-input-row">
            <input
              type="text"
              placeholder="e.g. Added altitude AH and orthogonal projection step"
              value={commitMessage}
              onChange={(e) => setCommitMessage(e.target.value)}
              className="vcs-input"
            />
            <button
              type="submit"
              className="vcs-submit-btn"
              disabled={!commitMessage.trim()}
            >
              <Plus size={13} />
              <span>Snapshot</span>
            </button>
          </div>
        </form>
      </div>

      {/* Semantic Proof State Diff */}
      {selectedCommit && (
        <div className="vcs-diff-section">
          <div className="vcs-section-label diff-header">
            <div className="diff-header-left">
              <Diff size={12} />
              <span>PROOF STATE DIFF (Working Tree vs #{selectedCommit.id})</span>
            </div>
            <button
              className="vcs-checkout-btn"
              onClick={() => onCheckout(selectedCommit)}
              title="Revert working tree to this snapshot"
            >
              <RotateCcw size={11} />
              <span>Checkout #{selectedCommit.id}</span>
            </button>
          </div>

          <div className="proof-diff-card">
            {/* Inference Rule Diffs */}
            <div className="diff-row">
              <span className="diff-label">Inference Rules:</span>
              <div className="diff-values">
                {addedRules.length === 0 && removedRules.length === 0 ? (
                  <span className="diff-unchanged">Identical rules</span>
                ) : (
                  <>
                    {addedRules.map(r => (
                      <span key={r} className="diff-pill added">+ {r}</span>
                    ))}
                    {removedRules.map(r => (
                      <span key={r} className="diff-pill removed">- {r}</span>
                    ))}
                  </>
                )}
              </div>
            </div>

            {/* New Hypotheses */}
            {newHypotheses.length > 0 && (
              <div className="diff-row">
                <span className="diff-label">New Hypotheses:</span>
                <div className="diff-values">
                  {newHypotheses.map(h => (
                    <span key={h.hypothesis} className="diff-pill added">
                      + {h.hypothesis}: {formatMathFormula(h.conclusion || '')}
                    </span>
                  ))}
                </div>
              </div>
            )}

            {/* Kernel Status Transition */}
            <div className="diff-row">
              <span className="diff-label">Kernel State:</span>
              <div className="diff-values kernel-diff">
                <span className={`status-badge-sm ${selectedCommit.verified ? 'verified' : 'rejected'}`}>
                  {selectedCommit.verified ? 'VERIFIED' : 'PENDING'}
                </span>
                <ArrowRight size={11} />
                <span className="status-badge-sm current">WORKING STATE</span>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Commit History Timeline */}
      <div className="vcs-history-container">
        <div className="vcs-section-label">
          <History size={12} />
          <span>REVISION TIMELINE ({commits.length} SNAPSHOTS)</span>
        </div>

        {commits.length === 0 ? (
          <div className="vcs-empty-state">
            No snapshots recorded yet. Create your first proof snapshot above.
          </div>
        ) : (
          <div className="vcs-timeline-list">
            {[...commits].reverse().map((commit) => {
              const isSelected = (selectedCommitId || commits[commits.length - 1]?.id) === commit.id;
              const dateStr = new Date(commit.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });

              return (
                <div
                  key={commit.id}
                  className={`vcs-commit-item ${isSelected ? 'active' : ''}`}
                  onClick={() => setSelectedCommitId(commit.id)}
                >
                  <div className="commit-item-top">
                    <div className="commit-hash-group">
                      <GitCommit size={13} className="commit-icon" />
                      <span className="commit-hash">#{commit.id}</span>
                      {commit.verified ? (
                        <span className="commit-verified-tag">
                          <ShieldCheck size={10} /> Verified
                        </span>
                      ) : (
                        <span className="commit-pending-tag">
                          <ShieldAlert size={10} /> Unverified
                        </span>
                      )}
                    </div>
                    <span className="commit-timestamp">{dateStr}</span>
                  </div>

                  <div className="commit-message-text">{commit.message}</div>
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};

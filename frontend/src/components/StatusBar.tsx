import React from 'react';
import { Cpu, ShieldCheck, ShieldAlert, CheckCircle2, Layers, Zap } from 'lucide-react';
import { VerificationResponse } from '../types';
import { isWasmReady } from '../services/wasmEngine';

interface StatusBarProps {
  verification: VerificationResponse | null;
  daemonOnline: boolean;
  activeFileName: string;
  isGeometry: boolean;
  stepCount: number;
  validStepCount: number;
}

export const StatusBar: React.FC<StatusBarProps> = ({
  verification,
  daemonOnline,
  activeFileName,
  isGeometry,
  stepCount,
  validStepCount,
}) => {
  const isVerified = verification?.verified ?? false;

  return (
    <footer className="workstation-statusbar">
      <div className="status-section left">
        <span className="status-item file-indicator">
          <span className="status-text">{activeFileName}</span>
        </span>
        <span className="status-divider">|</span>
        <span className="status-item">
          {isVerified ? (
            <span className="status-badge verified">
              <ShieldCheck size={12} />
              <span>KERNEL VERIFIED</span>
            </span>
          ) : (
            <span className="status-badge rejected">
              <ShieldAlert size={12} />
              <span>KERNEL PENDING / REJECTED</span>
            </span>
          )}
        </span>
        <span className="status-divider">|</span>
        <span className="status-item">
          <CheckCircle2 size={12} color={isVerified ? '#3fb950' : '#8b949e'} />
          <span>
            {stepCount > 0 ? `${validStepCount}/${stepCount} Steps Valid` : '0 Steps'}
          </span>
        </span>
      </div>

      <div className="status-section right">
        <span className="status-item">
          <Layers size={12} />
          <span>{isGeometry ? 'Interactive Geometry' : 'Pure Logic Mode'}</span>
        </span>
        <span className="status-divider">|</span>
        <span className="status-item daemon-status">
          {isWasmReady() ? (
            <>
              <Zap size={12} color="#10b981" />
              <span style={{ color: '#10b981', fontWeight: 600 }}>WASM Kernel (0ms)</span>
            </>
          ) : (
            <>
              <Cpu size={12} color={daemonOnline ? '#58a6ff' : '#8b949e'} />
              <span>{daemonOnline ? 'Kernel Daemon (Online)' : 'Local Engine'}</span>
            </>
          )}
        </span>
        <span className="status-divider">|</span>
        <span className="status-item encoding">UTF-8</span>
        <span className="status-divider">|</span>
        <span className="status-item language-id">proofer-lang</span>
      </div>
    </footer>
  );
};

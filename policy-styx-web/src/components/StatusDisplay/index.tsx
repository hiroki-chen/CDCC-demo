import React from 'react';
import { CircleNotchIcon, ShieldCheckIcon } from '@phosphor-icons/react';
import { AttestationReport } from "@/interface/computation";

interface StatusDisplayProps {
  status: 'idle' | 'pending' | 'succeeded';
  report: AttestationReport | null;
  onViewReport: () => void;
}

const StatusDisplay = ({ status, report, onViewReport }: StatusDisplayProps) => {
  // --- Renders when API call is in flight ---
  if (status === 'pending') {
    return (
      <div className="status-badge pending">
        <CircleNotchIcon size={20} className="spinner" /> Verifying Environment...
      </div>
    );
  }

  // --- Renders on successful attestation and verification ---
  if (status === 'succeeded' && report) {
    return (
      <div className="attestation-success">
        <div className="status-badge success">
          <ShieldCheckIcon weight="fill" size={20} /> Environment Verified
        </div>
        <div className="report-details">
          <h4>Attestation Report Summary</h4>
          <pre>
            <code>
              {/* Using substring for cleaner UI. Real hash might come from report. */}
              Session ID: {report.sessionId?.substring(0, 13) || 'N/A'}...
              <br />
              PCR0 Measurement: {'114514'}, // placeholder.
            </code>
          </pre>
          <a href="#" onClick={(e) => { e.preventDefault(); onViewReport(); }} className="view-report-link">
            View Full Report
            <span className="icon">🔍</span>
          </a>
        </div>
      </div>
    );
  }

  // --- Renders nothing if idle or in any other state ---
  return null;
};

export default StatusDisplay;
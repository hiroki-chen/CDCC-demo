import React from 'react';
import { CheckCircleIcon, FileTextIcon } from '@phosphor-icons/react';
import './index.less';
import { AttestationReport } from '@/interface/computation';


interface AttestationResultProps {
  report: AttestationReport;
  onViewReport: () => void;
}

const AttestationResult: React.FC<AttestationResultProps> = ({ report, onViewReport }) => {
  return (
    <div className="attestation-result">
      {/* 1. Primary Status */}
      <div className="result-header">
        <CheckCircleIcon className="success-icon" />
        <h4>Environment Verified</h4>
      </div>

      {/* 2. Data Summary Card */}
      <div className="report-summary">
        <div className="summary-title">Attestation Report Summary</div>
        <div className="summary-item">
          <span className="item-label">Session ID</span>
          <span className="item-value"><code>{report.sessionId}</code></span>
        </div>
        <div className="summary-item">
          <span className="item-label">Quote</span>
          <span className="item-value"><code>{report.quote.substring(0, 16)}...</code></span>
        </div>
      </div>

      {/* 3. Primary Action */}
      <a onClick={onViewReport} className="view-report-link">
        <FileTextIcon /> View Full Report
      </a>
    </div>
  );
};

export default AttestationResult;

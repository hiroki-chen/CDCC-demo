import { observer } from "mobx-react-lite"
import './index.less';
import { useEffect, useState } from "react";
import { SecureClient } from "@/api/cc/types/client";
import { AttestationReport } from "@/interface/computation";
import { startAttestation } from "@/api/cc";
import React from "react";
import Card from "@/components/Card";
import Button from "@/components/Button";
import StatusDisplay from "@/components/StatusDisplay";
import FileUploader from "@/components/FileUploader";
import Modal from "@/components/Modal";
// import { RocketLaunchIcon } from "@phosphor-icons/react";

const ComputationWizard = () => {
  const [client] = useState(() => new SecureClient());
  const [isClientReady, setIsClientReady] = useState(false);

  const [attestationStatus, setAttestationStatus] = useState<'idle' | 'pending' | 'succeeded'>('idle');
  const [attestationReport, setAttestationReport] = useState<AttestationReport | null>(null);
  const [dataFile, setDataFile] = useState<File | null>(null);
  const [programFile, setProgramFile] = useState<File | null>(null);

  const [isReportModalOpen, setIsReportModalOpen] = useState(false);

  // On component mount, generate the client's key pair.
  useEffect(() => {
    client.generateClientKeyPair().then(() => setIsClientReady(true))
      .catch(err => {
        console.error("🔥 Failed to generate client key pair:", err)
        alert("Failed to generate client key pair. Please check the console for details.");
      })
  }, [client]);

  const handleStartAttestation = async () => {
    setAttestationStatus('pending');

    try {
      const report = await startAttestation(client);
      setAttestationReport(report);
      setAttestationStatus('succeeded');
      alert("Attestation started successfully! You can now upload your data and program files.");
    } catch (error) {
      console.error("🔥 Error during attestation:", error);
      alert("Failed to start attestation. Please check the console for details.");
      setAttestationStatus('idle');
      alert(`Attestation failed: ${error}`);
    }
  };

  const handleFileUpload = async () => {
    if (!dataFile || !programFile || !attestationReport) {
      return; // do nothing.
    }
  };

  const getAttestationButtonText = () => {
    if (!isClientReady) {
      return 'Initializing client...';
    }
    if (attestationStatus === 'pending') {
      return 'Attestation in progress...';
    }

    return 'Please start attestation';
  };

  return (
    <div className='computation-wizard-root'>
      <Card>
        <h2>Create a New Secure Computation Job</h2>

        {/* --- STEP 1: ATTESTATION --- */}
        <div className={`step-card ${attestationStatus === 'succeeded' ? 'completed' : ''}`}>
          <div className='step-header'>
            <div className='step-indicator'>1</div>
            <h3> Establish Trust in the Secure Environment</h3>
          </div>
          {attestationStatus !== 'succeeded' && (
            <>
              <p>Before uploading any assets, you must verify the integrity of the remote virtual machine.</p>
              <Button
                onClick={handleStartAttestation}
                disabled={!isClientReady || attestationStatus === 'pending'}
              >
                {getAttestationButtonText()}
              </Button>
            </>
          )}
          <StatusDisplay status={attestationStatus} report={attestationReport} onViewReport={() => setIsReportModalOpen(true)} />
        </div>

        {/* STEP 2 and 3 remain exactly the same as the previous answer, as they just manage file state */}
        {/* --- STEP 2: UPLOAD ASSETS --- */}
        {attestationStatus === 'succeeded' && (
          <div className="step-card">
            {/* ... (step header is the same) ... */}
            <div className="step-header">
              <div className="step-indicator">2</div>
              <h3>Upload Your Assets</h3>
            </div>
            <div className="upload-columns">
              <FileUploader title="Data Owner: Upload Data" description="Select your dataset." selectedFile={dataFile} onFileSelect={setDataFile} />
              <FileUploader title="Developer: Upload Program" description="Select your compiled .wasm program." selectedFile={programFile} onFileSelect={setProgramFile} />
            </div>
          </div>
        )}
        {/* --- STEP 3: EXECUTE --- */}
        {dataFile && programFile && (
          <div className="step-card">
            {/* ... (step header is the same) ... */}
            <div className="step-header">
              <div className="step-indicator">3</div>
              <h3>Execute Computation</h3>
            </div>
            <p>The secure environment has been verified, and both assets are ready.</p>
            <Button onClick={handleFileUpload}  >
              Start Secure Computation
            </Button>
          </div>
        )}
      </Card>
      <Modal
        isOpen={isReportModalOpen}
        onClose={() => setIsReportModalOpen(false)}
        title="Full Attestation Report"
      >
        {attestationReport ? (
          <pre>
            <code>
              {JSON.stringify(attestationReport, null, 2)}
            </code>
          </pre>
        ) : (
          <p>No report data available.</p>
        )}
      </Modal>
    </div>
  )
}

export default observer(ComputationWizard);
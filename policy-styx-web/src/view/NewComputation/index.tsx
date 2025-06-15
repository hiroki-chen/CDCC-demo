import { observer } from "mobx-react-lite"
import './index.less';
import { useEffect, useState } from "react";
import { SecureClient } from "@/api/cc/types/client";
import { AttestationReport } from "@/interface/computation";
import { startAttestation, parseQuote, uploadFiles } from "@/api/cc";
import React from "react";
import Button from "@/components/Button";
// import StatusDisplay from "@/components/StatusDisplay";
import FileUploader from "@/components/FileUploader";
import Modal from "@/components/Modal";
// import { RocketLaunchIcon } from "@phosphor-icons/react";
import './index.less';
import AttestationResult from "@/components/AttestationResult";

const ComputationWizard = () => {
  const [client] = useState(() => new SecureClient());
  const [, setIsClientReady] = useState(false);

  const [attestationStatus, setAttestationStatus] = useState<'idle' | 'pending' | 'succeeded'>('idle');
  const [attestationReport, setAttestationReport] = useState<AttestationReport | null>(null);
  const [parsedReport, setParsedReport] = useState<string | null>(null);
  const [dataFile, setDataFile] = useState<File | null>(null);
  const [programFile, setProgramFile] = useState<File | null>(null);

  const [isReportModalOpen, setIsReportModalOpen] = useState(false);
  const [isFileUploadModalOpen, setIsFileUploadModalOpen] = useState(false);
  const [fileUploadModalContent, setFileUploadModalContent] = useState<string | null>(null);

  const [computationStatus, setComputationStatus] = useState<'idle' | 'pending' | 'succeeded'>('idle');
  const [computationResult, setComputationResult] = useState<Blob | null>(null);

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

      const parsedQuote = await parseQuote(report.quote);
      setParsedReport(JSON.stringify(parsedQuote, null, 2));
      console.log("✅ Attestation succeeded:", report);
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

    // Pack the files and session info into a payload.
    const payload = {
      dataFile: dataFile,
      programFile: programFile,
      sessionId: client.sessionId,
    }

    const response = await uploadFiles(client, payload);

    if (!response.ok) {
      const errorText = await response.text();
      console.error(`File upload failed with status ${response.status}:`, errorText);

      setIsFileUploadModalOpen(true);
      setFileUploadModalContent(`File upload failed with status ${response.status}: ${errorText}`);

      return;
    }
  };

  const handleStartComputation = async () => {
    if (!dataFile || !programFile || !attestationReport) {
      alert("Please ensure both data and program files are uploaded, and the environment is verified.");
      return; // do nothing.
    }

    setComputationStatus('pending');

    try {
      console.log("🚀 Starting computation with ", { dataFile, programFile });
      await new Promise(resolve => setTimeout(resolve, 20000)); // Simulate computation delay

      // TODO: Replace with actual computation logic
      const result = 123;
      setComputationResult(new Blob([result.toString()], { type: 'text/plain' }));
      setComputationStatus('succeeded');
      console.log("✅ Computation completed successfully:", result);
    } catch (error) {
      console.error("🔥 Error during computation:", error);
      alert("Failed to start computation. Please check the console for details.");
      setComputationStatus('idle');
      alert(`Computation failed: ${error}`);
    }
  };

  return (
    <div className="wizard-page-wrapper">
      <div className="computation-wizard-container">
        <div className="wizard-layout">
          {/* === Column 1: Main Title === */}
          <div className="wizard-column title-column">
            <h1 className="wizard-main-title">
              Create a New Secure Computation Job
            </h1>
          </div>

          {/* === Column 2: Step 1 - Attestation === */}
          <div className="wizard-column step-column">
            <div className="step-header">
              <span className="step-number">1</span>
              <h2 className="step-title">Establish Trust in the Secure Environment</h2>
            </div>
            <div className="step-content">
              {attestationStatus !== 'succeeded' && (
                <>
                  <p>Before proceeding, you must cryptographically verify the integrity of the remote environment.</p>
                  <Button
                    onClick={handleStartAttestation}
                    disabled={attestationStatus === 'pending'}
                    type="text"
                  >
                    {attestationStatus === 'pending' ? 'Verifying...' : 'Verify Environment'}
                  </Button>
                </>
              )}
              {attestationStatus === 'succeeded' && attestationReport && (<AttestationResult
                // status={attestationStatus}
                report={attestationReport!}
                onViewReport={() => setIsReportModalOpen(true)}
              />)}
            </div>
          </div>

          {/* === Column 3: Step 2 & 3 - Upload & Execute === */}
          <div className={`wizard-column step-column ${attestationStatus !== 'succeeded' ? 'is-disabled' : ''}`}>
            <div className="step-header">
              <span className="step-number">2</span>
              <h2 className="step-title">Upload & Execute</h2>
            </div>
            <div className="step-content">
              <p>Once the environment is trusted, upload your assets to begin the computation.</p>
              <div className="upload-columns">
                <FileUploader
                  title="Data Owner: Upload Data"
                  description="Upload the data file you want to process."
                  selectedFile={dataFile}
                  onFileSelect={setDataFile}
                />
                <FileUploader
                  title="Developer: Upload Program"
                  description="Upload the program file that will process the data."
                  selectedFile={programFile}
                  onFileSelect={setProgramFile}
                />
              </div>
              {dataFile && programFile && (
                <Button onClick={handleFileUpload} className="execute-button">
                  Upload
                </Button>
              )}
            </div>
          </div>
          <div className={`wizard-column step-column ${!dataFile || !programFile ? 'is-disabled' : ''}`}>
            <div className="step-header">
              <span className="step-number">3</span>
              <h2 className="step-title">Execute & View Result</h2>
            </div>
            <div className="step-content">

              {/* State 1: Idle - Ready to start */}
              {computationStatus === 'idle' && (
                <>
                  <p>All assets are ready. You can now start the secure computation.</p>
                  <Button onClick={handleStartComputation} className="execute-button">
                    Start Secure Computation
                  </Button>
                </>
              )}

              {/* State 2: Pending - Waiting for result */}
              {computationStatus === 'pending' && (
                <div className="computation-pending-indicator">
                  <div className="spinner"></div>
                  <h4>Computation in progress...</h4>
                  <p>Please wait while the secure environment processes your job. This may take a moment.</p>
                </div>
              )}

              {/* State 3: Succeeded - Display result */}
              {computationStatus === 'succeeded' && (
                <div className="computation-result-display">
                  <h4>Computation Complete</h4>
                  <pre><code>{computationResult?.size}</code></pre>
                </div>
              )}
            </div>
          </div>
        </div>

        <Modal
          isOpen={isReportModalOpen}
          onClose={() => setIsReportModalOpen(false)}
          title="Full Attestation Report"
        >
          {attestationReport ? (
            <pre><code>{JSON.stringify(parsedReport, null, 2)}</code></pre>
          ) : (
            <p>No report data available.</p>
          )}
        </Modal>

        <Modal
          isOpen={isFileUploadModalOpen}
          onClose={() => setIsFileUploadModalOpen(false)}
          title="File Upload Status"
        >
          <pre><code>{fileUploadModalContent!}</code></pre>
        </Modal>
      </div>
    </div >
  );
}

export default observer(ComputationWizard);
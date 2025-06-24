/* eslint-disable prettier/prettier */
import { arrayBufferToBase64, encryptWithAESGCM256 } from "@/utils/crypto";
import { SecureClient } from "./types/client";
import {
  AttestationReport,
  AttestationResponse,
  ExecutionPayload,
  ParsedQuote,
} from "@/interface/computation";
// import { keysToCamel } from "@/utils/convert";

// This is the URL for the compute backend (TDX VM)
export const compute_backend_url = "http://localhost:10086/api/v1";
// This is the URL for verifying quotes
export const verify_backend_url = "http://localhost:10087/api/v1";

export async function verifyQuote(quote: ArrayBuffer) {
  try {
    const response = await fetch(`${verify_backend_url}/verify`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({ quote: arrayBufferToBase64(quote) }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error(
        `Quote verification failed with status ${response.status}:`,
        errorText,
      );
      throw new Error(`Server responded with ${response.status}`);
    }

    const result = await response.json();
    return result.valid;
  } catch (error) {
    console.error("🔥 Failed to verify quote:", error);
    throw error; // Re-throw the error to be caught by the caller
  }
}

/**
 * Orchestrates the entire attestation and verification flow.
 * @param client - The stateful SecureClient instance.
 * @returns A promise that resolves with the processed attestation report.
 */
export async function startAttestation(
  client: SecureClient,
): Promise<AttestationReport> {
  // 1. Call the initial attestation endpoint.
  const response =
    (await client.policyStyxAttestationRequst()) as AttestationResponse;

  console.debug("Attestation response received:", response);

  // 2. Set the session ID on the cloent.
  client.setSessionId(response.sessionId);

  // 3. Deode server's public key and derive shared secret.
  const serverPublicKeyBytes = Uint8Array.from(atob(response.gy), (c) =>
    c.charCodeAt(0),
  ).buffer;
  await client.deriveSharedSecret(serverPublicKeyBytes);

  console.log("✅ Shared secret derived successfully: ", client.sessionKey);

  // 4. Verify the quote.
  const valid = await verifyQuote(
    Uint8Array.from(atob(response.quote), (c) => c.charCodeAt(0)).buffer,
  );
  if (!valid) {
    throw new Error("Quote verification failed.");
  }

  return {
    sessionId: response.sessionId, // UUID
    quote: response.quote, // Base64 encoded quote
  };
}

/**
 * Encrypts data, uploads files, and starts the computation job.
 * @param client - The SecureClient instance holding the session key.
 * @param payload - The files and session info.
 */
export async function uploadFiles(
  client: SecureClient,
  payload: ExecutionPayload,
): Promise<Response> {
  // 1. Read the user's data file into an ArrayBuffer.
  const dataBuffer = await payload.dataFile.arrayBuffer();

  // 2. Encrypt the data using the derived session key.
  const encryptedDataBuffer = await encryptWithAESGCM256(
    client.sessionKey,
    dataBuffer,
  );
  const encryptedDataFile = new Blob(
    [encryptedDataBuffer.encryptedData, encryptedDataBuffer.iv],
    { type: "application/octet-stream" },
  );

  // 3. Use FormData to send the encrypted data and program file.
  const formData = new FormData();
  formData.append("sessionId", payload.sessionId);
  formData.append("encryptedDataFile", encryptedDataFile, "encrypted_data.enc");
  formData.append("programFile", payload.programFile);

  // 4. Send the request to the compute backend.
  const response = await fetch(`${compute_backend_url}/upload`, {
    method: "POST",
    body: formData,
  });

  return response;
}

export async function parseQuote(quote: string): Promise<ParsedQuote> {
  const response = await fetch(`${verify_backend_url}/parse`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ quote }),
  });
  if (!response.ok) {
    const errorText = await response.text();
    console.error(
      `Quote parsing failed with status ${response.status}:`,
      errorText,
    );
    throw new Error(`Server responded with ${response.status}`);
  }
  const result = await response.json();
  return {
    quote: result.quote, // Assuming the response contains a 'quote' field
  };
}

/**
 * Prepares the computation by letting the server initialize all the needed computational
 * contexts and resources.
 */
export async function prepareComputation(
  client: SecureClient,
  dataFile: string,
  programFile: string,
): Promise<Response> {
  // 1. Call the prepare endpoint on the compute backend.
  const response = await fetch(`${compute_backend_url}/prepare`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      // We only need the session id for this request.
      sessionId: client.sessionId,
      dataFile: dataFile,
      programFile: programFile,
    }),
  });

  if (!response.ok) {
    const errorText = await response.text();
    console.error(
      `Preparation failed with status ${response.status}:`,
      errorText,
    );
    throw new Error(`Server responded with ${response.status}`);
  }

  return response;
}

export async function doCompute(
  client: SecureClient,
  entry: string,
  args: Record<string, Uint8Array>,
): Promise<Response> {
  const response = await fetch(`${compute_backend_url}/compute`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      sessionId: client.sessionId,
      entry: entry,
      args: args,
    }),
  });

  if (!response.ok) {
    const errorText = await response.text();
    console.error(
      `Computation failed with status ${response.status}:`,
      errorText,
    );
    throw new Error(`Server responded with ${response.status}`);
  }

  return response;
}

/* eslint-disable prettier/prettier */
import { arrayBufferToBase64 } from "@/utils/crypto";
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
  payload: ExecutionPayload
): Promise<Response> {
  // 1. First, send the plaintext data to data owner backend for encryption
  const encryptFormData = new FormData();
  encryptFormData.append("sessionKey", arrayBufferToBase64(client.sessionKey));
  encryptFormData.append("data", payload.dataFile);

  console.log("🔐 Encrypting data with data owner backend...");
  const encryptResponse = await fetch(`${verify_backend_url}/encrypt_data`, {
    method: "POST",
    body: encryptFormData,
  });

  if (!encryptResponse.ok) {
    const errorText = await encryptResponse.text();
    console.error(`Data encryption failed: ${encryptResponse.status}`, errorText);
    throw new Error(`Data encryption failed: ${encryptResponse.status}`);
  }

  const encryptedDataBlob = await encryptResponse.blob();
  console.log(`✅ Data encrypted (${encryptedDataBlob.size} bytes)`);

  // 2. Upload encrypted data to compute server
  const formData = new FormData();
  formData.append("sessionId", payload.sessionId);
  formData.append("encryptedDataFile", encryptedDataBlob, "encrypted_data.bin");
  formData.append("programFile", payload.programFile);
  formData.append("policyEngine", payload.policyEngine);

  // 3. Send the request to the compute backend
  console.log("📤 Uploading encrypted files to compute server...");
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
  // Convert Uint8Array values to regular arrays for JSON serialization
  const serializedArgs: Record<string, number[]> = {};
  for (const [key, value] of Object.entries(args)) {
    serializedArgs[key] = Array.from(value);
  }

  const response = await fetch(`${compute_backend_url}/compute`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({
      sessionId: client.sessionId,
      entry: entry,
      args: serializedArgs,
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

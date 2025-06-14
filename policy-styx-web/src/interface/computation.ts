/**
 * The raw response from the initial/attestation endpoint.
 */
export interface AttestationResponse {
  gy: string, //  Base64 encoded ArrayBuffer
  sessionId: string, // UUID
  quote: string, // The full attestation quote, base64 encoded.
  quote_type: string, // The type of quote.
}

export interface AttestationReport {
  sessionId: string, // UUID
  quote: string, // Base64 encoded quote
}

/**
 * The payload for the execution request.
 * By default we should not put the program and ecnrypted data together but
 * this is just a demo, so we will do this for convenience.
 */
export interface ExecutionPayload {
  encryptedData: Blob,
  programFile: File,
  sessionId: string, // UUID
}
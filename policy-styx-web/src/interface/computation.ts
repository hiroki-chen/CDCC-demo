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
  dataFile: File,
  programFile: File,
  sessionId: string, // UUID
}

export interface ParsedQuote {
  quote: string,
}

export interface ComputeRequest {
  sessionId: string, // UUID
  programName: string, // Name of the program to run
  dataName: string, // Name of the data file
  entry: string, // Entry point of the program
  args: string[], // Arguments to the program
}

export interface ComputationResult {
  sessionId: string, // UUID
  result: Blob, // The result of the computation
  status: 'success' | 'error', // Status of the computation
  errorMessage?: string, // Optional error message if status is 'error'
}

import { arrayBufferToBase64, deriveSharedSecret, generateECDHKeypair } from "@/utils/crypto";

// Define a type for clarity
export interface KeyPair {
  publicKey: ArrayBuffer | null;
  privateKey: CryptoKey | null;
}

export class SecureClient {
  private keyPair!: KeyPair; // Use '!' to tell TypeScript it will be initialized

  sessionKey!: ArrayBuffer;

  sessionId!: string;

  constructor() {
    // DO NOT call async functions in the constructor without handling the promise.
    // We will call it from React instead.
  }

  // This method MUST be async
  async generateClientKeyPair() {
    try {
      this.keyPair = await generateECDHKeypair();
      console.log("✅ Key pair generated successfully.");
    } catch (error) {
      console.error("🔥 Failed to generate key pair:", error);
      throw error; // Re-throw the error to be caught by the caller
    }
  }


  // This is the function called by the button click
  async policyStyxAttestationRequst() {
    // Export the public key from the generated key pair
    const publicKeyBuffer = this.keyPair.publicKey!;

    console.log("Public key buffer size (should be 65):", publicKeyBuffer.byteLength);

    // Convert to Base64
    const gx_base64 = arrayBufferToBase64(publicKeyBuffer);
    if (!gx_base64) {
      throw new Error("Failed to create Base64 public key.");
    }

    // ... rest of your fetch logic
    const response = await fetch('http://localhost:10086/api/v1/attestation', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ gx: gx_base64 }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.error(`Request failed with status ${response.status}:`, errorText);
      throw new Error(`Server responded with ${response.status}`);
    }

    console.log("✅ Attestation request sent successfully.");
    // ... handle successful response

    return response.json(); // Return the response JSON for further processing
  }


  setSessionId(sessionId: string) {
    this.sessionId = sessionId;
    console.log("✅ Session ID set successfully.");
  }

  async deriveSharedSecret(publicKey: ArrayBuffer) {
    try {
      this.sessionKey = await deriveSharedSecret(this.keyPair.privateKey!, publicKey);
      console.log("✅ Shared secret derived successfully.");
    } catch (error) {
      console.error("🔥 Failed to derive shared secret:", error);
      throw error; // Re-throw the error to be caught by the caller
    }
  }
}
import { deriveSharedSecret, generateECDHKeypair } from "@/utils/crypto";

export const backend_url = 'http://localhost:10086/api/v1';

// Define a type for clarity
export interface KeyPair {
  publicKey: ArrayBuffer | null;
  privateKey: CryptoKey | null;
}

export class SecureClient {
  private key_pair!: KeyPair; // Use '!' to tell TypeScript it will be initialized

  session_key!: ArrayBuffer;

  constructor() {
    // DO NOT call async functions in the constructor without handling the promise.
    // We will call it from React instead.
  }

  // This method MUST be async
  async generateClientKeyPair() {
    try {
      this.key_pair = await generateECDHKeypair();
      console.log("✅ Key pair generated successfully.");
    } catch (error) {
      console.error("🔥 Failed to generate key pair:", error);
      throw error; // Re-throw the error to be caught by the caller
    }
  }

  // This is the function called by the button click
  async policyStyxAttestationRequst() {
    // Export the public key from the generated key pair
    const publicKeyBuffer = this.key_pair.publicKey!;

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

  async deriveSharedSecret(publicKey: ArrayBuffer) {
    try {
      this.session_key = await deriveSharedSecret(this.key_pair.privateKey!, publicKey);
      console.log("✅ Shared secret derived successfully.");
    } catch (error) {
      console.error("🔥 Failed to derive shared secret:", error);
      throw error; // Re-throw the error to be caught by the caller
    }
  }

}

// Helper function (must be available)
function arrayBufferToBase64(buffer: ArrayBuffer): string {
  let binary = '';
  const bytes = new Uint8Array(buffer);
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return window.btoa(binary);
}

// /**
//  * This function will handle the attestation request to the backend.

//  */
// export function policyStyxAttestationRequest() {
//   // Create a new key pair.
//   const keyPair = window.crypto.subtle.generateKey(
//     {
//       name: 'ECDSA',
//       namedCurve: 'P-256',
//     },
//     true,
//     ['sign', 'verify']
//   );

//   // Extract the public key part in sec1 format.
//   const gx = keyPair.then((key) => {
//     return window.crypto.subtle.exportKey('raw', key.publicKey);
//   });

//   // Construct the attestation request.
//   const attestationRequest = gx.then((gx) => {
//     return {
//       gx: Array.from(new Uint8Array(gx)),
//       // Add other necessary fields for the attestation request.
//     };
//   });

//   // Send the attestation request to the backend.
//   return attestationRequest.then((request) => {
//     return fetch(`${backend_url}/attestation`, {
//       method: 'POST',
//       headers: {
//         'Content-Type': 'application/json',
//       },
//       body: JSON.stringify(request),
//     })
//       .then((response) => {
//         if (!response.ok) {
//           throw new Error('Network response was not ok');
//         }
//         return response.json();
//       })
//       .then((data) => {
//         // Handle the response from the backend.
//         console.log('Attestation successful:', data);
//         return data;
//       })
//       .catch((error) => {
//         console.error('Error during attestation:', error);
//         throw error;
//       });
//   });
// }
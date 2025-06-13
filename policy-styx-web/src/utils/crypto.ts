export async function generateECDHKeypair() {
  const { publicKey, privateKey } = await crypto.subtle.generateKey(
    {
      name: 'ECDH',
      namedCurve: 'P-256'
    },
    true,
    ['deriveKey', 'deriveBits']
  );

  return {
    publicKey: await crypto.subtle.exportKey('raw', publicKey),
    privateKey: privateKey
  };
}

export async function deriveSharedSecret(privateKey: CryptoKey, publicKey: ArrayBuffer) {

  const publicKeyObj = await crypto.subtle.importKey(
    'raw',
    publicKey,
    { name: 'ECDH', namedCurve: 'P-256' },
    false,
    []
  );

  return await crypto.subtle.deriveBits(
    {
      name: 'ECDH',
      public: publicKeyObj
    },
    privateKey,
    256
  );
}

export async function encryptWithAESGCM256(key: ArrayBuffer, data: ArrayBuffer) {
  const iv = crypto.getRandomValues(new Uint8Array(12)); // 96-bit IV for AES-GCM
  const encryptedData = await crypto.subtle.encrypt(
    {
      name: 'AES-GCM',
      iv: iv
    },
    await crypto.subtle.importKey('raw', key, 'AES-GCM', false, ['encrypt']),
    data
  );

  return {
    iv: iv,
    encryptedData: new Uint8Array(encryptedData)
  };
}

export async function decryptWithAESGCM256(key: ArrayBuffer, iv: Uint8Array, encryptedData: Uint8Array) {
  const decryptedData = await crypto.subtle.decrypt(
    {
      name: 'AES-GCM',
      iv: iv as BufferSource
    },
    await crypto.subtle.importKey('raw', key, 'AES-GCM', false, ['decrypt']),
    encryptedData as BufferSource
  );

  return new Uint8Array(decryptedData);
}
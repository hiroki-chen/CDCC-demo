#ifndef _pcd_crypto_aes_gcm_H_
#define _pcd_crypto_aes_gcm_H_

#ifdef PCD_CONFIG_CRYPTO_AES_GCM

#include <stdint.h>

#define PCD_CRYPTO_AES_IV_LEN	12

typedef uint8_t pcd_crypto_aes_gcm_key_t[16];
typedef uint8_t pcd_crypto_aes_gcm_tag_t[12];

typedef struct {
	uint8_t iv[PCD_CRYPTO_AES_IV_LEN];
	pcd_crypto_aes_gcm_tag_t mac;
	size_t aad_size;
	uint8_t aad[];
} pcd_crypto_aes_gcm_metadata_t;


int pcd_crypto_aes_gcm_init(void);
int pcd_crypto_aes_gcm_exit(void);

int pcd_crypto_backend_aes_gcm_decrypt(void *ciphertext, size_t cipher_size,
                                       void **plaintext, size_t *plain_size,
                                       pcd_crypto_aes_gcm_metadata_t *metadata,
                                       pcd_crypto_aes_gcm_key_t *key);
int pcd_crypto_backend_aes_gcm_encrypt(void *plaintext, size_t plain_size,
                                       void **ciphertext, size_t *cipher_size,
                                       pcd_crypto_aes_gcm_metadata_t *metadata,
                                       pcd_crypto_aes_gcm_key_t *key);
static int
pcd_crypto_backend_aes_gcm_verify(void *ciphertext, size_t cipher_size,
                                  pcd_crypto_aes_gcm_metadata_t *metadata,
                                  void *key) {
  return 0;
}

#endif // PCD_CONFIG_CRYPTO_AES_GCM

#endif // _pcd_crypto_aes_gcm_H_

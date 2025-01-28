#ifndef _pcd_crypto_aes_gcm_H_
#define _pcd_crypto_aes_gcm_H_

#ifdef PCD_CONFIG_CRYPTO_AES_GCM

#include <stdint.h>

#include <crypto/aes_backend_types.h>

#define PCD_CRYPTO_AES_IV_LEN	12

typedef struct {
	uint8_t iv[PCD_CRYPTO_AES_IV_LEN];
	pcd_crypto_aes_gcm_tag_t mac;
	size_t aad_size;
	uint8_t aad[];
} pcd_crypto_aes_gcm_metadata_t;

int pcd_crypto_aes_gcm_init(void);
int pcd_crypto_aes_gcm_exit(void);

#include <crypto/aes_backend.h>

#endif // PCD_CONFIG_CRYPTO_AES_GCM

#endif // _pcd_crypto_aes_gcm_H_

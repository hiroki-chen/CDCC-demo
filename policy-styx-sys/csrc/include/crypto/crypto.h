#ifndef _PCD_CRYPTO_H_
#define _PCD_CRYPTO_H_

#include <stdint.h>

typedef uint32_t pcd_crypto_algo_t;
#define PCD_CRYPTO_PLAIN		0x0000
#define PCD_CRYPTO_AES_GCM		0x0001

#define PCD_CRYPTO_ALGO_MAX_INDEX	0x0001

typedef uint64_t pcd_crypto_nonce_t;

typedef struct {
	int (*decrypt)(void *ciphertext, size_t cipher_size, void **plaintext, size_t *plain_size, void *metadata, void *key);
	int (*encrypt)(void *plaintext, size_t plain_size, void **ciphertext, size_t *cipher_size, void *metadata, void *key);
	int (*verify)(void *ciphertext, size_t cipher_size, void *metadata, void *key);
	int (*generate_metadata)(uint8_t *iv, uint8_t *aad, size_t aad_size, void *mac, void **output_metadata);
	int (*get_mac_from_metadata)(void *input_metadata, void **output_mac, size_t *mac_size);
	size_t mac_size;
	size_t key_size;
	size_t iv_size;
} pcd_crypto_algo_struct_t;

pcd_crypto_algo_struct_t *pcd_crypto_get_algo(pcd_crypto_algo_t algo_id);
int pcd_crypto_register_algo(pcd_crypto_algo_struct_t *algo, pcd_crypto_algo_t algo_id);
int pcd_crypto_init(void);

#endif

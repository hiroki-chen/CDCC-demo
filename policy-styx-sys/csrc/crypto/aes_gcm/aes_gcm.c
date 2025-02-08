#include <string.h>
#include <stdlib.h>

#include "error_codes.h"
#include <crypto/crypto.h>
#include <crypto/aes_gcm.h>

static int pcd_crypto_aes_gcm_get_mac_from_metadata(void *input_metadata, void **output_mac, size_t *mac_size) {
	void *ret_mac;
	pcd_crypto_aes_gcm_metadata_t *metadata = (pcd_crypto_aes_gcm_metadata_t *)input_metadata;
	
	if (metadata == NULL || output_mac == NULL || mac_size == NULL) {
		return PCD_NULL_ARG;
	}

	ret_mac = malloc(sizeof(pcd_crypto_aes_gcm_tag_t));
	if (ret_mac == NULL) {
		return PCD_MEMERR;
	}

	memcpy(ret_mac, &metadata->mac, sizeof(pcd_crypto_aes_gcm_tag_t));
	*output_mac = ret_mac;
	*mac_size = sizeof(pcd_crypto_aes_gcm_tag_t);

	return PCD_OK;
}

static int pcd_crypto_aes_gcm_generate_metadata(uint8_t *iv, uint8_t *aad, size_t aad_size, void *mac, void **output_metadata) {
	pcd_crypto_aes_gcm_metadata_t *ret_metadata;

	if (iv == NULL || aad == NULL || output_metadata == NULL) {
		return PCD_NULL_ARG;
	}

	ret_metadata = (pcd_crypto_aes_gcm_metadata_t *)malloc(sizeof(pcd_crypto_aes_gcm_metadata_t) + aad_size);
	if (ret_metadata == NULL) {
		return PCD_MEMERR;
	}

	memcpy(ret_metadata->iv, iv, PCD_CRYPTO_AES_IV_LEN);
	ret_metadata->aad_size = aad_size;
	memcpy(ret_metadata->aad, aad, aad_size);

	if (mac != NULL) {
		memcpy(ret_metadata->mac, mac, sizeof(pcd_crypto_aes_gcm_tag_t));
	}
	else {
		memset(ret_metadata->mac, 0, sizeof(pcd_crypto_aes_gcm_tag_t));
	}

	*output_metadata = (void *)ret_metadata;

	return PCD_OK;
}

pcd_crypto_algo_struct_t pcd_crypto_algo_plain = {
	.encrypt = (int (*)(void *, size_t,  void **, size_t *, void *, void *))&pcd_crypto_backend_aes_gcm_encrypt,
	.decrypt = (int (*)(void *, size_t,  void **, size_t *, void *, void *))&pcd_crypto_backend_aes_gcm_decrypt,
	.verify  = (int (*)(void *, size_t,  void *, void *))&pcd_crypto_backend_aes_gcm_verify,
	.generate_metadata = &pcd_crypto_aes_gcm_generate_metadata,
	.get_mac_from_metadata = &pcd_crypto_aes_gcm_get_mac_from_metadata,
	.mac_size = sizeof(pcd_crypto_aes_gcm_tag_t),
	.key_size = sizeof(pcd_crypto_aes_gcm_key_t),
	.iv_size = PCD_CRYPTO_AES_IV_LEN
};

int pcd_crypto_aes_gcm_init(void) {
	return pcd_crypto_register_algo(&pcd_crypto_algo_plain, PCD_CRYPTO_AES_GCM);
}

int pcd_crypto_aes_gcm_exit(void) {
	return PCD_OK;
}

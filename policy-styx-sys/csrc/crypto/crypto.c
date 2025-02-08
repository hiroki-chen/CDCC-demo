#include <string.h>

#include "error_codes.h"
#include "crypto/crypto.h"

static pcd_crypto_algo_struct_t *pcd_crypto_algos[PCD_CRYPTO_ALGO_MAX_INDEX + 1];

pcd_crypto_algo_struct_t *pcd_crypto_get_algo(pcd_crypto_algo_t algo_id) {
	if (algo_id < 0 || algo_id > PCD_CRYPTO_ALGO_MAX_INDEX) {
		return NULL;
	}

	return pcd_crypto_algos[algo_id];
}

int pcd_crypto_register_algo(pcd_crypto_algo_struct_t *algo, pcd_crypto_algo_t algo_id) {
	if (algo == NULL) {
		return PCD_NULL_ARG;
	}

	if (algo_id < 0 || algo_id > PCD_CRYPTO_ALGO_MAX_INDEX) {
		return PCD_OUT_OF_RANGE;
	}

	if (pcd_crypto_algos[algo_id] != NULL) {
		return PCD_REINIT;
	}

	pcd_crypto_algos[algo_id] = algo;

	return PCD_OK;
}

#include <crypto/plain.h>
#include <crypto/aes_gcm.h>

int pcd_crypto_init() {
	memset(&pcd_crypto_algos, 0, sizeof(pcd_crypto_algos));

#ifdef PCD_CONFIG_CRYPTO_PLAIN
	pcd_crypto_plain_init();
#endif

#ifdef PCD_CONFIG_CRYPTO_AES_GCM
	pcd_crypto_aes_gcm_init();
#endif


	return PCD_OK;
}

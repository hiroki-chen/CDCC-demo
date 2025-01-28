#include <stdint.h>
#include <string.h>
#include <stdlib.h>

#include "identity.h"
#include "data.h"
#include "log.h"
#include "error_codes.h"
#include "tag.h"

#include "crypto/crypto.h"
#include "policy/policy_def.h"

static uint8_t pcd_data_unpacker_generic_iv[16] = { [0 ... 15] = 0 };

int pcd_data_unpacker_decrypt_payload(void *enc_payload_with_mac, size_t enc_size_with_mac,
				pcd_identity_t *owner_id,
				pcd_crypto_algo_t crypto_algo, void *key,
				pcd_payload_t **payload) {
	pcd_crypto_algo_struct_t *algo;
	int status;
	size_t mac_size;
	void *mac = enc_payload_with_mac;
	size_t enc_size;
	void *enc_payload;
	void *metadata;
	size_t payload_size_dummy;

	algo = pcd_crypto_get_algo(crypto_algo);
	if (algo == NULL) {
		return PCD_CRYPTO_NSUPPORT;
	}

	// Generate metadata
	status = (*algo->generate_metadata)(pcd_data_unpacker_generic_iv, (uint8_t *)owner_id, sizeof(pcd_identity_t), mac, &metadata);
	if (status != PCD_OK) {
		return status;
	}
	
	// Get MAC size and set enc_payload
	mac_size = algo->mac_size;
	enc_payload = enc_payload_with_mac + mac_size;
	enc_size = enc_size_with_mac - mac_size;
	
	// Decrypt
	status = (*algo->decrypt)(enc_payload, enc_size, (void **)payload, &payload_size_dummy, metadata, key);
	if (status != PCD_OK) {
		free(metadata);
		return status;
	}

	// Free metadata
	free(metadata);

	return PCD_OK;
}

static void print_buffer(char *buffer, size_t size) {
	int i; int j;
	for (j = 0; j < size; ) {
		for (i = 0; i < 8; i++) {
			pcd_log("%02X ", (unsigned char)buffer[j]);
			j++;
		}
		pcd_log("\n");
	}
}
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

static uint8_t pcd_data_packer_generic_iv[16] = { [0 ... 15] = 0 };

int pcd_data_packer_encrypt_payload(pcd_payload_t *payload, size_t payload_size,
				pcd_identity_t *owner_id,
				pcd_crypto_algo_t crypto_algo, void *key,
				void **enc_payload, size_t *enc_size, 
				void **mac, size_t *mac_size) {
	pcd_crypto_algo_struct_t *algo;
	int status;
	void *metadata;

	algo = pcd_crypto_get_algo(crypto_algo);
	if (algo == NULL) {
		return PCD_CRYPTO_NSUPPORT;
	}

	// Generate metadata
	status = (*algo->generate_metadata)(pcd_data_packer_generic_iv, (uint8_t *)owner_id, sizeof(pcd_identity_t), NULL, &metadata);
	if (status != PCD_OK) {
		return status;
	}
	
	// Encrypt
	status = (*algo->encrypt)((void *)payload, payload_size, enc_payload, enc_size, metadata, key);
	if (status != PCD_OK) {
		free(metadata);
		return status;
	}

	// Get MAC
	status = (*algo->get_mac_from_metadata)(metadata, mac, mac_size);
	if (status != PCD_OK) {
		free(*enc_payload);
		free(metadata);
		return status;
	}

	// Free metadata
	free(metadata);

	return PCD_OK;
}


int pcd_data_packer_generate_data(void *data, size_t data_size,
				pcd_identity_t *owner_id,
				pcd_delegator_addr_t *delegator_addr,
				pcd_policy_t *policy,
				pcd_tag_t *tags, uint32_t tag_count,
				void *attributes, size_t attribute_size,
				pcd_crypto_algo_t crypto_algo, void *key,
				pcd_enc_data_t **out_data) {
	size_t total_size;
	size_t policy_size;
	size_t tag_size;
	pcd_payload_t *plain_payload;
	void *enc_payload;
	size_t enc_size;
	void *mac;
	size_t mac_size;
	pcd_enc_data_t *ret_data;
	int status;

	if ((data == NULL && data_size != 0) || owner_id == NULL || delegator_addr == NULL
	    || (policy == NULL) || (tags == NULL && tag_count != 0)
	    || (attributes == NULL && attribute_size != 0) ||key == NULL) {
		pcd_log_error("ERROR: pcd_data_pcaker_generate_data: Parameter is NULL\n");
		return PCD_NULL_ARG;
	}
	*out_data = NULL;

	// Verify address fits in 64 bytes
	if (strnlen((char *)delegator_addr, 64) > PCD_DELEGATOR_ADDR_MAX_LEN - 1 
		&& ((char *)delegator_addr)[PCD_DELEGATOR_ADDR_MAX_LEN - 1] != (char)0) {
		// Invalid addr
		return PCD_ADDR_TOO_LONG;
	}

	// Generate payload
	policy_size = sizeof(pcd_policy_t) + policy->policy_size;
	tag_size = sizeof(pcd_tag_t) * tag_count;
	total_size = sizeof(pcd_payload_t) + data_size + policy_size + tag_size + attribute_size;

	plain_payload = (pcd_payload_t *)malloc(total_size);
	if (plain_payload == NULL) {
		pcd_log_error("ERROR: pcd_data_pcaker_generate_data: Failed to allocate plain payload\n");
		return PCD_MEMERR;
	}

	plain_payload->data_size = data_size;
	plain_payload->policy_size = policy_size;
	plain_payload->tag_size = tag_size;
	plain_payload->attribute_size = attribute_size;

	if (data_size > 0)
		memcpy(plain_payload->payload, data, data_size);
	if (policy_size > 0)
		memcpy(plain_payload->payload + data_size, policy, policy_size);
	if (tag_size > 0)
		memcpy(plain_payload->payload + data_size + policy_size, tags, tag_size);
	if (attribute_size > 0)
		memcpy(plain_payload->payload + data_size + policy_size + tag_size, attributes, attribute_size);

	// Encrypt payload
	status = pcd_data_packer_encrypt_payload(plain_payload, total_size,
				owner_id, crypto_algo, key,
				&enc_payload, &enc_size, 
				&mac, &mac_size);
	if (status != PCD_OK) {
		free(plain_payload);
		return status;
	}
	free(plain_payload);

	// Stich data
	ret_data = (pcd_enc_data_t *)malloc(sizeof(pcd_enc_data_t) + mac_size + enc_size);
	if (ret_data == NULL) {
		free(enc_payload);
		if (mac_size != 0) {
			free(mac);
		}
		return PCD_MEMERR;
	}

	ret_data->protocol_version = PCD_PROTOCOL_VERSION;
	memcpy(&ret_data->owner_id, owner_id, sizeof(pcd_identity_t));
	ret_data->enc_algo = crypto_algo;
	ret_data->enc_size = mac_size + enc_size;
	memcpy(&ret_data->delegator_addr, delegator_addr, PCD_DELEGATOR_ADDR_MAX_LEN);

	if (mac_size > 0) {
		memcpy(ret_data->encrypted_payload, mac, mac_size);
	}
	memcpy(ret_data->encrypted_payload + mac_size, enc_payload, enc_size);

	free(enc_payload);
	if (mac_size != 0) {
		free(mac);
	}

	*out_data = ret_data;

	return PCD_OK;
}

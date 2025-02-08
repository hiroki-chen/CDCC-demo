#include <stdint.h>
#include <string.h>
#include <stdlib.h>

#include "data.h"
#include "data_packer.h"
#include "secret.h"
#include "log.h"
#include "error_codes.h"

int pcd_generate_data(void *data, size_t data_size,
				pcd_identity_t *data_owner_id,
				pcd_delegator_addr_t *delegator_addr,
				pcd_policy_t *policy,
				pcd_tag_t *tags, uint32_t tag_count,
				void *attributes, uint64_t attribute_size,
				pcd_crypto_algo_t crypto_algo,
				pcd_enc_data_t **output_data) {
	int status;
	pcd_secret_t *secret;
	int j, k;
	uint8_t *payload_test = (uint8_t *)data;
	
	if ((data == NULL && data_size != 0) || data_owner_id == NULL || delegator_addr == NULL
	    || (policy == NULL) || (tags == NULL && tag_count != 0)
	    || (attributes == NULL && attribute_size != 0)) {
		pcd_log_error("ERROR: pcd_generate_data: Parameter is NULL\n");
		return PCD_NULL_ARG;
	}

	// Get key from secret storage
	status = pcd_secret_retrieve(data_owner_id, &secret);
	if(status != PCD_OK) {
		return status;
	}

	status = pcd_data_packer_generate_data(data, data_size, data_owner_id,
				delegator_addr, policy,
				tags, tag_count, 
				attributes, attribute_size,
				crypto_algo, (void *)(&secret->secret),
				output_data);
	if (status != PCD_OK) {
		pcd_secret_release(data_owner_id);
		pcd_log_error("ERROR: pcd_producer_generate_data: Packer failed to generate data with %d\n", status);
		return status;
	}
	pcd_secret_release(data_owner_id);

	return status;
}
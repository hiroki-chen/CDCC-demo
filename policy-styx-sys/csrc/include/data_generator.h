#ifndef _PCD_DATA_GENERATOR_H_
#define _PCD_DATA_GENERATOR_H_

#include "data.h"
#include "tag.h"
#include "identity.h"

#include "policy/policy_def.h"
#include "crypto/crypto.h"

int pcd_generate_data(void *data, size_t data_size,
				pcd_identity_t *data_owner_id,
				pcd_delegator_addr_t *delegator_addr,
				pcd_policy_t *policy,
				pcd_tag_t *tags, uint32_t tag_count,
				void *attributes, uint64_t attribute_size,
				pcd_crypto_algo_t crypto_algo,
				pcd_enc_data_t **output_data);

#endif

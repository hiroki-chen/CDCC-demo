#ifndef _PCD_DATA_PACKER_H_
#define _PCD_DATA_PACKER_H_

#include <stdint.h>

#include "data.h"
#include "tag.h"

#include "policy/policy_def.h"
#include "crypto/crypto.h"

int pcd_data_packer_generate_data(void *data, size_t data_size,
				pcd_identity_t *owner_id,
				pcd_delegator_addr_t *delegator_addr,
				pcd_policy_t *policy,
				pcd_tag_t *tags, uint32_t tag_count,
				void *attributes, size_t attribute_size,
				pcd_crypto_algo_t crypto_algo, void *key,
				pcd_enc_data_t **out_data);

#endif
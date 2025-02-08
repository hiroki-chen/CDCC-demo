#ifndef _PCD_DATA_UNPACKER_H_
#define _PCD_DATA_UNPACKER_H_

#include <stdint.h>

#include "data.h"
#include "tag.h"

#include "policy/policy.h"
#include "crypto/crypto.h"

int pcd_data_unpacker_decrypt_payload(void *enc_payload_with_mac, size_t enc_size_with_mac,
				pcd_identity_t *owner_id,
				pcd_crypto_algo_t crypto_algo, void *key,
				pcd_payload_t **payload);

#endif

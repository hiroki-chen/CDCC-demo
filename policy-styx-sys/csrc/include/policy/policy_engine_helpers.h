#ifndef _PCD_POLICY_ENGINE_HELPER_H_
#define _PCD_POLICY_ENGINE_HELPER_H_

#include <stdint.h>

#include "crypto/sha256.h"

uint32_t pcd_get_program_hash(pcd_sha256_t *output_hash);
uint32_t pcd_get_output_custodian(pcd_identity_t *output_id);

#endif
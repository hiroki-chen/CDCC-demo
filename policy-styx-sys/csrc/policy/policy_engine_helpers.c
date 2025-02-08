#include <stdint.h>
#include <string.h>

#include "app.h"
#include "log.h"
#include "identity.h"

#include "policy/policy_def.h"
#include "crypto/sha256.h"

uint32_t pcd_get_program_hash(pcd_sha256_t *output_hash) {
	pcd_sha256_t *app_hash = NULL;
	
	app_hash = pcd_app_get_hash();
	if (app_hash == NULL) {
		pcd_log_error("ERROR: pcd_get_program_hash: pcd_app_get_hash failed\n");
		return -1;
	}

	memcpy(output_hash, app_hash, sizeof(pcd_sha256_t));

	return 0;
}


uint32_t pcd_get_output_custodian(pcd_identity_t *output_id) {
	pcd_identity_t *output_identity = NULL;

	output_identity = pcd_app_get_custodian_id();
	if (output_identity == NULL) {
		pcd_log_error("ERROR: pcd_get_output_custodian: pcd_app_get_custodian_id failed\n");
		return -1;
	}

	memcpy(output_id, output_identity, sizeof(pcd_identity_t));

	return 0;
}
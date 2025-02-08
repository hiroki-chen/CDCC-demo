#include <stdint.h>

#include "log.h"
#include "identity.h"
#include "error_codes.h"
#include "runtime.h"
#include "crypto/sha256.h"

static pcd_module_t *pcd_app_module = NULL;
static pcd_instance_t *pcd_app_instance = NULL;

static pcd_sha256_t pcd_app_hash;
static pcd_identity_t pcd_app_custodian_id;

// SHA256
pcd_sha256_t *pcd_app_get_hash() {
	if (pcd_app_module == NULL) {
		pcd_log_error("ERROR: pcd_app_get_hash: No app loaded\n");
		return NULL;
	}
	return &pcd_app_hash;
}

pcd_identity_t *pcd_app_get_custodian_id() {
	if (pcd_app_module == NULL) {
		pcd_log_error("ERROR: pcd_app_get_hash: No app loaded\n");
		return NULL;
	}
	return &pcd_app_custodian_id;
}

pcd_instance_t *pcd_app_get_instance() {
	return pcd_app_instance;
}

int pcd_app_load(uint8_t *app_module_buffer, uint32_t app_module_size, pcd_identity_t *custodian_id) {
	int ret;
	if (pcd_app_module != NULL) {
		pcd_log_error("ERROR: pcd_app_load: App already loaded\n");
		return PCD_REINIT;
	}

	ret = pcd_crypto_sha256_hash_buffer(app_module_buffer, app_module_size, &pcd_app_hash);
	if (ret != 0) {
		pcd_log_error("ERROR: pcd_app_load: pcd_crypto_sha256_hash_buffer failed\n");
		return PCD_UNKNOWN;
	}

	memcpy(&pcd_app_custodian_id, custodian_id, sizeof(pcd_identity_t));

	pcd_app_module = pcd_runtime_load_module(app_module_buffer, app_module_size);
	if (pcd_app_module == NULL) {
		pcd_log_error("ERROR: pcd_app_load: Failed to load app module\n");
		return PCD_RUNTIME_ERR;
	}

	return PCD_OK;
}

int pcd_app_run(const char **dir_allow_list, const uint32_t dir_allow_count, 
			uint32_t stack_size, uint32_t heap_size, uint32_t *argv, uint32_t argc) {
	int status;

	if (pcd_app_module == NULL) {
		pcd_log_error("ERROR: pcd_app_run: No app loaded\n");
		return PCD_RUNTIME_ERR;
	}

	if (pcd_app_instance != NULL) {
		pcd_log_error("ERROR: pcd_app_run: App already launched\n");
		return PCD_REINIT;
	}

	pcd_app_instance = pcd_runtime_instantiate_module(pcd_app_module, dir_allow_list, dir_allow_count, stack_size, heap_size);
	if (pcd_app_instance == NULL) {
		pcd_log_error("ERROR: pcd_app_run: Failed to instantiate app module\n");
		return PCD_RUNTIME_ERR;
	}

	status = pcd_runtime_execute_function(pcd_app_instance, "main", argv, argc);
	if (status != 0) {
		// Failed to execute
		pcd_log_error("ERROR: pcd_app_run: Failed to execute main funciton\n");
		pcd_runtime_deinstantiate_module(pcd_app_instance);
		pcd_app_instance = NULL;
		return PCD_RUNTIME_ERR;
	}
	else {
		pcd_log("INFO: pcd_app_run: main returned %d\n", argv[0]);
	}

	pcd_runtime_deinstantiate_module(pcd_app_instance);
	pcd_app_instance = NULL;

	return PCD_OK;
}

int pcd_app_unload() {
	if (pcd_app_module == NULL) {
		pcd_log_error("ERROR: pcd_app_run: No app loaded\n");
		return PCD_RUNTIME_ERR;
	}

	pcd_runtime_unload_module(pcd_app_module);
	pcd_app_module = NULL;
	return 0;
}
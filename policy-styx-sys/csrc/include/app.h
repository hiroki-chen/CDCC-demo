#ifndef _PCD_APP_H_
#define _PCD_APP_H_

#include <stdint.h>

#include "identity.h"
#include "runtime.h"
#include "crypto/sha256.h"

pcd_sha256_t *pcd_app_get_hash(void);
pcd_identity_t *pcd_app_get_custodian_id(void);

pcd_instance_t *pcd_app_get_instance(void);
int pcd_app_load(uint8_t *app_module_buffer, uint32_t app_module_size, pcd_identity_t *custodian_id);
int pcd_app_run(const char **dir_allow_list, const uint32_t dir_allow_count, 
			uint32_t stack_size, uint32_t heap_size, uint32_t *argv, uint32_t argc);
int pcd_app_unload(void);

#endif
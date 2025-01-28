#ifndef _PCD_RUNTIME_H_
#define _PCD_RUNTIME_H_

#include <stdint.h>

// #ifdef PCD_CONFIG_RUNTIME_WAMR
#include "runtime/wamr.h"
// #endif

uint32_t pcd_runtime_setup_environment(void);
pcd_module_t *pcd_runtime_load_module(uint8_t *module_buffer, uint32_t module_size);
pcd_instance_t *pcd_runtime_instantiate_module(pcd_module_t *module, const char **dir_allow_list, const uint32_t dir_allow_count, 
					uint32_t stack_size, uint32_t heap_size);

uint32_t pcd_runtime_execute_function(pcd_instance_t *instance, char *func_name,
				uint32_t *argv, uint32_t argc);
void pcd_runtime_deinstantiate_module(pcd_instance_t *instance);
void pcd_runtime_unload_module(pcd_module_t *module);
void pcd_runtime_destroy_environment(void);

pcd_runtime_pointer_t pcd_runtime_malloc(pcd_instance_t *instance, const uint32_t data_size, void **native_ptr);
pcd_runtime_pointer_t pcd_runtime_copy_data_into_runtime(pcd_instance_t *instance, const void *data, const uint32_t data_size);
void *pcd_runtime_app_to_native(pcd_instance_t *instance, pcd_runtime_pointer_t app_addr);
pcd_runtime_pointer_t pcd_runtime_native_to_app(pcd_instance_t *instance, void *native_addr);
void pcd_runtime_free(pcd_instance_t *instance, pcd_runtime_pointer_t app_addr);

#endif
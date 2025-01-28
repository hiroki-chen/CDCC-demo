#include <stdint.h>
#include <stdlib.h>

#include "runtime/wamr.h"

#include "error_codes.h"
#include "log.h"
#include "wasm_export.h"
#include "bh_platform.h"

#ifdef PCD_CONFIG_DATASET

#include "dataset.h"

// Native API Wrappers
uint32_t pcd_dataset_new_wrapper(wasm_exec_env_t exec_env, uint32_t dataset_max_size) {
	return pcd_dataset_new(dataset_max_size);
}

uint32_t pcd_dataset_add_data_wrapper(wasm_exec_env_t exec_env, uint32_t dataset_index, pcd_enc_data_t *input_data) {
	return pcd_dataset_add_data(dataset_index, input_data);
}

uint32_t pcd_dataset_check_policy_wrapper(wasm_exec_env_t exec_env, uint32_t dataset_index, pcd_identity_t *program_owner_id) {
	return pcd_dataset_check_policy(dataset_index, program_owner_id);
}

pcd_runtime_pointer_t pcd_dataset_access_wrapper(wasm_exec_env_t exec_env, uint32_t dataset_index, uint32_t data_index) {
	return pcd_dataset_access(dataset_index, data_index);
}

uint32_t pcd_dataset_release_wrapper(wasm_exec_env_t exec_env, uint32_t dataset_index) {
	return pcd_dataset_release(dataset_index);
}

static NativeSymbol pcd_dataset_native_symbols[] = 
{
    { "pcd_dataset_new", 		pcd_dataset_new_wrapper, 		"(i)i" },
    { "pcd_dataset_add_data", 		pcd_dataset_add_data_wrapper, 		"(i*)i" },
    { "pcd_dataset_check_policy", 	pcd_dataset_check_policy_wrapper, 	"(i*)i" },
    { "pcd_dataset_access", 		pcd_dataset_access_wrapper, 		"(ii)i" },
    { "pcd_dataset_release", 		pcd_dataset_release_wrapper, 		"(i)i" }
};

#endif

#ifdef PCD_CONFIG_DATA_GENERATOR

#include "data_generator.h"

int pcd_generate_data_wrapper(wasm_exec_env_t exec_env, void *data, size_t data_size,
			pcd_identity_t *data_owner_id,
			pcd_delegator_addr_t *delegator_addr,
			pcd_policy_t *policy,
			pcd_tag_t *tags, uint32_t tag_count,
			void *attributes, uint64_t attribute_size,
			pcd_crypto_algo_t crypto_algo,
			uint32_t *output_data_app_ptr) {
	wasm_module_inst_t instance = NULL;
	pcd_enc_data_t *encrypted_data = NULL;
	int ret_val;
	uint32_t output_data_ptr;

	ret_val = pcd_generate_data(data, data_size, data_owner_id, delegator_addr, policy, 
					tags, tag_count, attributes, attribute_size, crypto_algo,
					&encrypted_data);
	if (ret_val != PCD_OK) {
		return ret_val;
	}

	// Copy buffer into the runtime
	instance = wasm_runtime_get_module_inst(exec_env);
	output_data_ptr = wasm_runtime_module_dup_data(instance, (void *)encrypted_data, sizeof(pcd_enc_data_t) + encrypted_data->enc_size);
	*output_data_app_ptr = output_data_ptr;
	free(encrypted_data);

	return ret_val;
}

static NativeSymbol pcd_data_generator_native_symbols[] = 
{
    { "pcd_generate_data", 		pcd_generate_data_wrapper, 		"(*~****i*~i*)i" }
};

#endif

extern sgx_status_t ocall_open(int* retval, const char* pathname, int flags, bool has_mode, unsigned int mode);
extern sgx_status_t ocall_fstat(int* retval, int fd, void* buf, unsigned int buf_len);
extern sgx_status_t ocall_close(int* retval, int fd);
extern sgx_status_t ocall_read_file_to_outside_buffer(char** ret_buffer, int file, size_t file_size, size_t* read_size);
extern sgx_status_t ocall_write_file_to_outside(int file, char *buffer, size_t write_size);

#ifdef PCD_CONFIG_CONSUMER_DATA_GENERATOR

#include "data_generator_consumer_internal.h"

#define	S_IRUSR	0000400
#define	S_IWUSR	0000200
#define	S_IRGRP	0000040
#define	S_IWGRP	0000020
#define S_IROTH 0000004

int pcd_consumer_generate_data_wrapper(wasm_exec_env_t exec_env, void *data, size_t data_size,
			pcd_identity_t *data_owner_id,
			pcd_delegator_addr_t *delegator_addr,
			pcd_policy_t *policy,
			pcd_tag_t *tags, uint32_t tag_count,
			void *attributes, uint32_t attribute_size,
			pcd_crypto_algo_t crypto_algo,
			const char *output_path) {
	wasm_module_inst_t instance = NULL;
	pcd_enc_data_t *encrypted_data = NULL;
	int ret_val;
	uint32_t output_data_ptr;
	int file = -1;

	ret_val = pcd_consumer_generate_data(data, data_size, data_owner_id, delegator_addr, policy, 
					tags, tag_count, attributes, attribute_size, crypto_algo,
					&encrypted_data);
	if (ret_val != PCD_OK) {
		pcd_log_error("ERROR: Failed to generate data\n");
		return ret_val;
	}

	// output_path
	ocall_open(&file, output_path, O_RDWR | O_CREAT, true, S_IRUSR | S_IWUSR | S_IRGRP | S_IWGRP | S_IROTH);
    	if (file < 0) {
    	    pcd_log_error("ERROR: Write file failed: open file %s failed.\n", output_path);
	    free(encrypted_data);
    	    return -1;
    	}
	
	ocall_write_file_to_outside(file, (char *)encrypted_data, sizeof(pcd_enc_data_t) + encrypted_data->enc_size);

    	ocall_close(&ret_val, file);

	free(encrypted_data);

	return ret_val;
}

static NativeSymbol pcd_consumer_data_generator_native_symbols[] = 
{
    { "pcd_consumer_generate_data", 	pcd_consumer_generate_data_wrapper, 	"(*~****i*~i$)i" }
};

#endif

#ifdef PCD_CONFIG_POLICY_ENGINE

#include "policy/policy_engine_helpers.h"

uint32_t pcd_get_program_hash_wrapper(wasm_exec_env_t exec_env, pcd_sha256_t *output_hash) {
	return pcd_get_program_hash(output_hash);
}

uint32_t pcd_get_output_custodian_wrapper(wasm_exec_env_t exec_env, pcd_identity_t *output_id) {
	return pcd_get_output_custodian(output_id);
}

static NativeSymbol pcd_policy_engine_native_symbols[] = 
{
    { "pcd_get_program_hash", 		pcd_get_program_hash_wrapper, 		"(*)i" },
    { "pcd_get_output_custodian", 	pcd_get_output_custodian_wrapper, 	"(*)i" }
};

#endif

#include <stddef.h>

extern pcd_instance_t *pcd_app_get_instance(void);

extern int ocall_read_file_to_outside_buf(char **ret_buffer, int file, size_t file_size, size_t *read_size);
extern int ocall_free_outside_buffer(char *outside_buffer);

uint32_t pcd_read_file_to_buffer(wasm_exec_env_t exec_env, const char *filename, size_t *ret_size) {
    char *buffer;
    char *outside_buffer;
    int file;
    size_t file_size, buf_size, read_size;
    struct stat stat_buf;
    uint32_t ret_ptr;
    int ret;

    if (!filename || !ret_size) {
        pcd_log_error("ERROR: Read file to buffer failed: invalid filename or ret size.\n");
        return 0;
    }

    ocall_open(&file, filename, O_RDONLY, false, 0);
    if (file < 0) {
        pcd_log_error("ERROR: Read file to buffer failed: open file %s failed.\n", filename);
        return 0;
    }
    
    ocall_fstat(&ret, file, &stat_buf, sizeof(struct stat));
    if (ret != 0) {
        pcd_log_error("ERROR: Read file to buffer failed: fstat file %s failed.\n", filename);
        ocall_close(&ret, file);
        return 0;
    }
    file_size = stat_buf.st_size;

    buf_size = file_size > 0 ? file_size : 1;

    ret_ptr = wasm_runtime_module_malloc(get_module_inst(exec_env), buf_size, (void **)&buffer);
    if (ret_ptr == 0) {
        pcd_log_error("ERROR: Read file to buffer failed: alloc memory failed.\n");
        ocall_close(&ret, file);
        return 0;
    }
    
     //ocall_read(&read_size, file, buffer, file_size);
    ocall_read_file_to_outside_buffer(&outside_buffer, file, file_size, &read_size);
    if (outside_buffer == NULL) {
	pcd_log_error("ERROR: Failed to read to outside buffer\n");
	ocall_close(&ret, file);
	return 0;
    }

    memcpy(buffer, outside_buffer, read_size);
    
    ocall_free_outside_buffer(outside_buffer);

    ocall_close(&ret, file);

    if (read_size < file_size) {
        pcd_log_error("ERROR: Read file to buffer failed: read file content failed.\n");
        free(buffer);
        return 0;
    }

    *ret_size = file_size;
    return ret_ptr;
}

static NativeSymbol pcd_debug_native_symbols[] = 
{
    { "read_file_to_buffer", 		pcd_read_file_to_buffer, 		"(**)i" }
};

int
ocall_print(const char *str);

uint32_t pcd_runtime_setup_environment() {
#ifdef PCD_CONFIG_DATASET
	int pcd_dataset_n_native_symbols;
#endif

#ifdef PCD_CONFIG_DATA_GENERATOR
	int pcd_data_generator_n_native_symbols;
#endif

#ifdef PCD_CONFIG_CONSUMER_DATA_GENERATOR
	int pcd_consumer_data_generator_n_native_symbols;
#endif

#ifdef PCD_CONFIG_POLICY_ENGINE
	int pcd_policy_engine_n_native_symbols;
#endif

	int pcd_debug_n_native_symbols;

	os_set_print_function(ocall_print);
	wasm_runtime_init();

	// Add native APIs to the environment
	// TODO: Add symbols to corresponding environments instead of env
#ifdef PCD_CONFIG_DATASET
	pcd_dataset_n_native_symbols = sizeof(pcd_dataset_native_symbols) / sizeof(NativeSymbol);
	if (!wasm_runtime_register_natives("env", pcd_dataset_native_symbols, pcd_dataset_n_native_symbols)) {
		pcd_log_error("ERROR: pcd_runtime_setup_environment: Failed to register native symbols for datasets\n");
		goto fail;
	}
#endif

#ifdef PCD_CONFIG_DATA_GENERATOR
	pcd_data_generator_n_native_symbols = sizeof(pcd_data_generator_native_symbols) / sizeof(NativeSymbol);
	if (!wasm_runtime_register_natives("env", pcd_data_generator_native_symbols, pcd_data_generator_n_native_symbols)) {
		pcd_log_error("ERROR: pcd_runtime_setup_environment: Failed to register native symbols for data generator\n");
		goto fail;
	}
#endif

#ifdef PCD_CONFIG_CONSUMER_DATA_GENERATOR
	pcd_consumer_data_generator_n_native_symbols = sizeof(pcd_consumer_data_generator_native_symbols) / sizeof(NativeSymbol);
	if (!wasm_runtime_register_natives("env", pcd_consumer_data_generator_native_symbols, pcd_consumer_data_generator_n_native_symbols)) {
		pcd_log_error("ERROR: pcd_runtime_setup_environment: Failed to register native symbols for consumer data generator\n");
		goto fail;
	}
#endif


#ifdef PCD_CONFIG_POLICY_ENGINE
	pcd_policy_engine_n_native_symbols = sizeof(pcd_policy_engine_native_symbols) / sizeof(NativeSymbol);
	if (!wasm_runtime_register_natives("env", pcd_policy_engine_native_symbols, pcd_policy_engine_n_native_symbols)) {
		pcd_log_error("ERROR: pcd_runtime_setup_environment: Failed to register native symbols for data generator\n");
		goto fail;
	}
#endif

	pcd_debug_n_native_symbols = sizeof(pcd_debug_native_symbols) / sizeof(NativeSymbol);
	if (!wasm_runtime_register_natives("env", pcd_debug_native_symbols, pcd_debug_n_native_symbols)) {
		pcd_log_error("ERROR: pcd_runtime_setup_environment: Failed to register native symbols for debug\n");
		goto fail;
	}
	return PCD_OK;

fail:
	wasm_runtime_destroy();
	return PCD_RUNTIME_ERR;
}

uint32_t pcd_wamr_register_native_symbol(const char *name, void *ptr, const char *signature) {
	NativeSymbol *symbol = malloc(sizeof(NativeSymbol));

	symbol->symbol = name;
	symbol->func_ptr = ptr;
	symbol->signature = signature;
	if (!wasm_runtime_register_natives("env", symbol, 1)) {
		pcd_log_error("ERROR: pcd_wamr_register_native_symbol: Failed to register native symbol %s\n", name);
		return PCD_RUNTIME_ERR;
	}
	return PCD_OK;
}

pcd_module_t *pcd_runtime_load_module(uint8_t *module_buffer, uint32_t module_size) {
	pcd_module_t *module = NULL;
	char error_buf[128];

	if (!(module = (pcd_module_t *)wasm_runtime_load(module_buffer, module_size,
						error_buf, sizeof(error_buf)))) {
		pcd_log_error("ERROR: %s\n", error_buf);
		return NULL;
	}

	return module;
}

static uint32_t pcd_runtime_stack_size = 0;

pcd_instance_t *pcd_runtime_instantiate_module(pcd_module_t *module, const char **dir_allow_list, const uint32_t dir_allow_count, 
					uint32_t stack_size, uint32_t heap_size) {
	pcd_instance_t *instance = NULL;
	char error_buf[128];
	
	wasm_runtime_set_wasi_args((wasm_module_t)module,
				dir_allow_list,
				dir_allow_count,
				NULL, 0, // map_dir
				NULL, 0, // env
				NULL, 0 // argv, argc. Not set here
				);
	//wasm_runtime_set_wasi_args_ex((wasm_module_t)module, dir_allow_list, dir_allow_count, NULL, 0, NULL, 0, NULL,
        //                          0, 0, 1, 2);

	if (!(instance = (pcd_instance_t *)wasm_runtime_instantiate((wasm_module_t)module,
						stack_size,
						heap_size,
						error_buf,
						sizeof(error_buf)))) {
		pcd_log_error("ERROR: %s\n", error_buf);
		return NULL;
	}

	pcd_runtime_stack_size = stack_size;

	return instance;
}

uint32_t pcd_runtime_execute_function(pcd_instance_t *instance, char *func_name,
				uint32_t *argv, uint32_t argc) {
	wasm_function_inst_t func = NULL;
	wasm_exec_env_t exec_env = NULL;

	if (!(func = wasm_runtime_lookup_function((wasm_module_inst_t)instance, func_name, NULL))) {
		pcd_log_error("ERROR: Failed to lookup function %s\n", func_name);
		return -1;
	}

	if (!(exec_env = wasm_runtime_create_exec_env((wasm_module_inst_t)instance, pcd_runtime_stack_size))) {
		pcd_log_error("ERROR: Failed to create exec_env for function %s\n", func_name);
		return -1;
	}

	if (wasm_runtime_call_wasm(exec_env, func, argc, argv) ) {
		//pcd_log("INFO: %s function returned %d\n", func_name, argv[0]);
	}
	else {
		pcd_log_error("ERROR: Failed to call %s. %s\n", func_name, wasm_runtime_get_exception((wasm_module_inst_t)instance));
		wasm_runtime_destroy_exec_env(exec_env);
		return -1;
	}

	wasm_runtime_destroy_exec_env(exec_env);

	return 0;

}

void pcd_runtime_deinstantiate_module(pcd_instance_t *instance) {
	wasm_runtime_deinstantiate((wasm_module_inst_t)instance);
}

void pcd_runtime_unload_module(pcd_module_t *module) {
	wasm_runtime_unload((wasm_module_t)module);
}

void pcd_runtime_destroy_environment() {
	wasm_runtime_destroy();
}

pcd_runtime_pointer_t pcd_runtime_malloc(pcd_instance_t *instance, const uint32_t data_size, void **native_ptr) {
	return (pcd_runtime_pointer_t)wasm_runtime_module_malloc((wasm_module_inst_t)instance, data_size, native_ptr);
}

pcd_runtime_pointer_t pcd_runtime_copy_data_into_runtime(pcd_instance_t *instance, const void *data, const uint32_t data_size) {
	return (pcd_runtime_pointer_t)wasm_runtime_module_dup_data((wasm_module_inst_t)instance, data, data_size);
}

void *pcd_runtime_app_to_native(pcd_instance_t *instance, pcd_runtime_pointer_t app_addr) {
	return wasm_runtime_addr_app_to_native((wasm_module_inst_t)instance, (uint32_t)app_addr);
}

pcd_runtime_pointer_t pcd_runtime_native_to_app(pcd_instance_t *instance, void *native_addr) {
	return (pcd_runtime_pointer_t)wasm_runtime_addr_native_to_app((wasm_module_inst_t)instance, native_addr);
}

void pcd_runtime_free(pcd_instance_t *instance, pcd_runtime_pointer_t app_addr) {
	wasm_runtime_module_free((wasm_module_inst_t)instance, (uint32_t)app_addr);
}
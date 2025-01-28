#include <string.h>

#include "runtime.h"

#include "log.h"
#include "error_codes.h"
#include "policy/policy.h"
#include "identity.h"
#include "data.h"
#include "dataset.h"

#include "stopwatch.h"

static int pcd_policy_loaded = 0;
static pcd_policy_disc_t pcd_policy_types[PCD_POLICY_TYPE_MAX_INDEX + 1] = { [0 ... PCD_POLICY_TYPE_MAX_INDEX] = { {0}, NULL} };

int pcd_policy_load_disc(char *module_buffer, size_t module_size, pcd_policy_type_t *type) {
	if (pcd_policy_loaded > PCD_POLICY_TYPE_MAX_INDEX) {
		pcd_log_error("ERROR: Too many DISC registered\n");
		return PCD_OUT_OF_RANGE;
	}

	pcd_policy_types[pcd_policy_loaded].disc_module = pcd_runtime_load_module(module_buffer, module_size);
	if (pcd_policy_types[pcd_policy_loaded].disc_module == NULL) {
		pcd_log_error("ERROR: Failed to load DISC module\n");
		return PCD_RUNTIME_ERR;
	}

	memcpy(&pcd_policy_types[pcd_policy_loaded].disc_id, type, sizeof(pcd_policy_type_t));

	pcd_policy_loaded += 1;

	return PCD_OK;
}

int pcd_policy_eval_over_dataset(pcd_dataset_t *dataset, pcd_identity_t *program_owner_id) {
	int i, j;
	pcd_instance_t *disc_instance = NULL;
	pcd_runtime_pointer_t *data_app_addr_array = NULL;
	pcd_runtime_pointer_t data_array_app_addr = 0;
	uint32_t argv[2];
	int ret = PCD_OK;

	PCD_EVAL_DEFINE_WATCH(watch);

	pcd_eval_stopwatch_start(&watch);

	// Instantiate policy module
	for (i = 0; i < pcd_policy_loaded; i++) {
		if (!pcd_compare_policy_type(&pcd_policy_types[i].disc_id, &dataset->policy_type)) {
			disc_instance = pcd_runtime_instantiate_module(pcd_policy_types[i].disc_module, NULL, 0, PCD_POLICY_STACK_SIZE, PCD_POLICY_HEAP_SIZE);
			if (disc_instance == NULL) {
				return PCD_RUNTIME_ERR;
			}
			break;
		}
	}
	if (i == pcd_policy_loaded) {
		pcd_log("INFO: No policy found\n");
		return PCD_NOT_FOUND;
	}

	pcd_eval_stopwatch_lap("pcd_policy_eval_over_dataset: Instantiate policy module", &watch, 1);

	// Copy all data in the dataset into the policy module
	data_array_app_addr = pcd_runtime_malloc(disc_instance, (dataset->data_count + 1) * sizeof(pcd_runtime_pointer_t), (void *)&data_app_addr_array);
	if (data_array_app_addr == 0) {
		pcd_log_error("ERROR: Failed to allocate data_app_addr_array\n");
		ret = PCD_MEMERR;
		goto fail;
	}

	for (i = 0; i < dataset->data_count; i++) {
		// Copy each payload
		data_app_addr_array[i] = pcd_runtime_copy_data_into_runtime(disc_instance, dataset->payload_pointers[i], 
						sizeof(pcd_payload_t) + dataset->payload_pointers[i]->data_size
						 + dataset->payload_pointers[i]->policy_size + dataset->payload_pointers[i]->tag_size + dataset->payload_pointers[i]->attribute_size);
		if (data_app_addr_array[i] == 0) {
			pcd_log_error("ERROR: Failed to copy data [%d]\n", i);
			ret = PCD_RUNTIME_ERR;
			goto fail;
		}
	}

	pcd_eval_stopwatch_lap("pcd_policy_eval_over_dataset: Copy all data into runtime", &watch, 1);

	argv[0] = data_array_app_addr;
	argv[1] = dataset->data_count;

	// Call the policy module
	if ((ret = pcd_runtime_execute_function((wasm_module_inst_t)disc_instance, "eval", argv, 2))) {
		pcd_log("INFO: Failed to execute eval\n");
	}
	else {
		ret = argv[0];
		if (ret == PCD_OK) {
			//pcd_log("INFO: Policy check passed\n");
			dataset->dataset_policy_passed = 1;
			ret = PCD_OK;
		}
		else if (ret == PCD_DENINED) {
			pcd_log("INFO: Policy doesn't met\n");
			ret = PCD_DENINED;
		}
		else {
			pcd_log("INFO: pcd_runtime_execute_function returned %d\n", ret);
		}
	}

	pcd_eval_stopwatch_lap("pcd_policy_eval_over_dataset: Evaluate policy", &watch, 1);
fail:
	pcd_runtime_deinstantiate_module(disc_instance);

	pcd_eval_stopwatch_lap("pcd_policy_eval_over_dataset: Done", &watch, 1);
	return ret;
	
}

int pcd_policy_eval_output_over_dataset(pcd_dataset_t *dataset,
					void *data, size_t data_size,
					pcd_identity_t *data_owner_id,
					pcd_policy_t *policy,
					void *attributes, uint64_t attribute_size) {
	int i, j;
	pcd_instance_t *disc_instance = NULL;
	pcd_runtime_pointer_t *data_app_addr_array = NULL;
	pcd_runtime_pointer_t data_array_app_addr = 0;
	pcd_runtime_pointer_t output_data_app_addr = 0;
	pcd_runtime_pointer_t output_custodian_id_app_addr = 0;
	pcd_runtime_pointer_t output_policy_app_addr = 0;
	pcd_runtime_pointer_t output_attributes_app_addr = 0;
	uint32_t argv[8];
	int ret = PCD_OK;

	PCD_EVAL_DEFINE_WATCH(watch);

	pcd_eval_stopwatch_start(&watch);

	// Instantiate policy module
	for (i = 0; i < pcd_policy_loaded; i++) {
		if (!pcd_compare_policy_type(&pcd_policy_types[i].disc_id, &dataset->policy_type)) {
			disc_instance = pcd_runtime_instantiate_module(pcd_policy_types[i].disc_module, NULL, 0, PCD_POLICY_STACK_SIZE, PCD_POLICY_HEAP_SIZE);
			if (disc_instance == NULL) {
				return PCD_RUNTIME_ERR;
			}
			break;
		}
	}
	if (i == pcd_policy_loaded) {
		pcd_log("INFO: No policy found\n");
		return PCD_NOT_FOUND;
	}

	pcd_eval_stopwatch_lap("pcd_policy_eval_output_over_dataset: Instantiate policy module", &watch, 1);

	// Copy all data in the dataset into the policy module
	data_array_app_addr = pcd_runtime_malloc(disc_instance, (dataset->data_count + 1) * sizeof(pcd_runtime_pointer_t), (void *)&data_app_addr_array);
	if (data_array_app_addr == 0) {
		pcd_log_error("ERROR: Failed to allocate data_app_addr_array\n");
		ret = PCD_MEMERR;
		goto fail;
	}

	for (i = 0; i < dataset->data_count; i++) {
		// Copy each payload
		data_app_addr_array[i] = pcd_runtime_copy_data_into_runtime(disc_instance, dataset->payload_pointers[i], 
						sizeof(pcd_payload_t) + dataset->payload_pointers[i]->data_size
						 + dataset->payload_pointers[i]->policy_size + dataset->payload_pointers[i]->tag_size + dataset->payload_pointers[i]->attribute_size);
		if (data_app_addr_array[i] == 0) {
			pcd_log_error("ERROR: Failed to copy data [%d]\n", i);
			ret = PCD_RUNTIME_ERR;
			goto fail;
		}
	}

	output_data_app_addr = pcd_runtime_copy_data_into_runtime(disc_instance, data, data_size);
	if (output_data_app_addr == 0) {
		pcd_log_error("ERROR: Failed to copy output data\n");
		ret = PCD_RUNTIME_ERR;
		goto fail;
	}

	output_custodian_id_app_addr = pcd_runtime_copy_data_into_runtime(disc_instance, data_owner_id, sizeof(pcd_identity_t));
	if (output_custodian_id_app_addr == 0) {
		pcd_log_error("ERROR: Failed to copy output custodian ID\n");
		ret = PCD_RUNTIME_ERR;
		goto fail;
	}

	output_policy_app_addr = pcd_runtime_copy_data_into_runtime(disc_instance, policy, sizeof(pcd_policy_t) + policy->policy_size);
	if (output_policy_app_addr == 0) {
		pcd_log_error("ERROR: Failed to copy output policy\n");
		ret = PCD_RUNTIME_ERR;
		goto fail;
	}

	output_attributes_app_addr = pcd_runtime_copy_data_into_runtime(disc_instance, attributes, attribute_size);
	if (output_attributes_app_addr == 0) {
		pcd_log_error("ERROR: Failed to copy output policy\n");
		ret = PCD_RUNTIME_ERR;
		goto fail;
	}

	pcd_eval_stopwatch_lap("pcd_policy_eval_output_over_dataset: Copy all data into runtime", &watch, 1);

	argv[0] = data_array_app_addr;
	argv[1] = dataset->data_count;
	argv[2] = output_data_app_addr;
	argv[3] = data_size;
	argv[4] = output_custodian_id_app_addr;
	argv[5] = output_policy_app_addr;
	argv[6] = output_attributes_app_addr;
	argv[7] = attribute_size;


	// Call the policy module
	if ((ret = pcd_runtime_execute_function((wasm_module_inst_t)disc_instance, "eval_output", argv, 8))) {
		pcd_log("INFO: Failed to execute eval\n");
	}
	else {
		ret = argv[0];
		if (ret == PCD_OK) {
			//pcd_log("INFO: Policy check passed\n");
			dataset->dataset_policy_passed = 1;
			ret = PCD_OK;
		}
		else if (ret == PCD_DENINED) {
			pcd_log("INFO: Policy doesn't met\n");
			ret = PCD_DENINED;
		}
		else {
			pcd_log("INFO: pcd_runtime_execute_function returned %d\n", ret);
		}
	}

	pcd_eval_stopwatch_lap("pcd_policy_eval_output_over_dataset: Evaluate policy", &watch, 1);
fail:
	pcd_runtime_deinstantiate_module(disc_instance);

	pcd_eval_stopwatch_lap("pcd_policy_eval_output_over_dataset: Done", &watch, 1);
	return ret;
	
}

#include <stdint.h>
#include <stdlib.h>

#include "log.h"
#include "error_codes.h"
#include "data.h"
#include "dataset.h"
#include "data_unpacker.h"
#include "runtime.h"
#include "identity.h"
#include "secret.h"
#include "app.h"

#include "policy/policy.h"

#include "stopwatch.h"

static char pcd_dataset_occupied[PCD_MAX_AMOUNT_DATASET] = { 0 };
static pcd_dataset_t *pcd_datasets[PCD_MAX_AMOUNT_DATASET] = { NULL };

static int pcd_dataset_current = -1;

pcd_dataset_t *pcd_get_current_active_dataset() {
	if (pcd_dataset_current < 0) {
		return NULL;
	}
	else {
		return pcd_datasets[pcd_dataset_current];
	}
}

uint32_t pcd_dataset_new(uint32_t dataset_max_size) {
	int i;

	PCD_EVAL_DEFINE_WATCH(watch);

	pcd_eval_stopwatch_start(&watch);

	for (i = 0; i < PCD_MAX_AMOUNT_DATASET; i++) {
		if (pcd_dataset_occupied[i] == 0) {
			pcd_dataset_occupied[i] = 1;
			pcd_datasets[i] = (pcd_dataset_t *)malloc(sizeof(pcd_dataset_t) + dataset_max_size * sizeof(pcd_payload_t *));
			if (pcd_datasets[i] == NULL) {
				pcd_log_error("ERROR: Failed to allocate pcd_dataset\n");
				pcd_dataset_occupied[i] = 0;
				return -PCD_MEMERR;
			}
			pcd_datasets[i]->data_max_count = dataset_max_size;
			pcd_datasets[i]->dataset_policy_passed = 0;
			pcd_datasets[i]->data_count = 0;
			memset(&pcd_datasets[i]->policy_type, 0, sizeof(pcd_policy_type_t));
			pcd_eval_stopwatch_lap("pcd_dataset_new: New dataset", &watch, 1);
			return i;
		}
	}

	pcd_eval_stopwatch_lap("pcd_dataset_new: New dataset", &watch, 1);

	return -PCD_OUT_OF_RANGE;
}

uint32_t pcd_dataset_add_data(uint32_t dataset_index, pcd_enc_data_t *input_data) {
	// Decrypt and add to dataset
	// Note that the app cannot access the dataset here. It's decrypted in the middleware
	pcd_payload_t *plain_payload = NULL;
	pcd_secret_t *secret = NULL;
	int status;

	PCD_EVAL_DEFINE_WATCH(watch);

	// Check if dataset has an empty slot
	if (pcd_datasets[dataset_index]->data_max_count == pcd_datasets[dataset_index]->data_count) {
		pcd_log_error("ERROR: The dataset is full. Currently used %d/%d\n", pcd_datasets[dataset_index]->data_count, pcd_datasets[dataset_index]->data_max_count);
		return PCD_OUT_OF_RANGE;
	}


	pcd_eval_stopwatch_start(&watch);

	// Fetch the key
	status = pcd_secret_fetch(&input_data->owner_id, &input_data->delegator_addr, &secret);
	if (status != PCD_OK) {
		pcd_log_error("ERROR: pcd_dataset_add_data: Failed to fetch secret with %d\n", status);
		return status;
	}
	//pcd_log("INFO: pcd_dataset_add_data: fetched secret\n");

	pcd_eval_stopwatch_lap("pcd_dataset_add_data: Fecth secret", &watch, 1);

	// Decrypt
	status = pcd_data_unpacker_decrypt_payload(input_data->encrypted_payload, input_data->enc_size,
				&input_data->owner_id,
				input_data->enc_algo, (void *)(&secret->secret),
				&plain_payload);
	if (status != PCD_OK) {
		pcd_log_error("ERROR: pcd_dataset_add_data: Failed to decrypt with %d\n", status);
		pcd_secret_release(&input_data->owner_id);
		return status;
	}

	pcd_eval_stopwatch_lap("pcd_dataset_add_data: Decrypt", &watch, 1);

	pcd_secret_release(&input_data->owner_id);

	// Check if it's using the same DISC or if it's the first piece of data
	if (pcd_datasets[dataset_index]->data_count == 0) {
		memcpy(&(pcd_datasets[dataset_index]->policy_type), &(pcd_get_policy_from_payload(plain_payload)->type), sizeof(pcd_policy_type_t));
		pcd_log("INFO: dataset %d's policy is ", dataset_index);
		pcd_print_id((pcd_identity_t *)&(pcd_datasets[dataset_index]->policy_type));
		pcd_log("\n");
	}
	else {
		if (pcd_compare_policy_type(&(pcd_datasets[dataset_index]->policy_type), &(pcd_get_policy_from_payload(plain_payload)->type))) {
			pcd_log("INFO: pcd_dataset_add_data: Policy DISC mismatch!\n");
			pcd_secret_release(&input_data->owner_id);
			free(plain_payload);
			return PCD_POLICY_NSUPPORT;
		}
	}

	// Reset policy passed and add payload to dataset
	pcd_datasets[dataset_index]->dataset_policy_passed = 0;
	pcd_datasets[dataset_index]->payload_pointers[pcd_datasets[dataset_index]->data_count] = plain_payload;
	pcd_datasets[dataset_index]->data_count++;

	pcd_eval_stopwatch_lap("pcd_dataset_add_data: Post processing", &watch, 1);

	return PCD_OK;
}

uint32_t pcd_dataset_check_policy(uint32_t dataset_index, pcd_identity_t *program_owner_id) {
	uint32_t passed = 0;
	//pcd_log("DEBUG: Entering pcd_dataset_check_policy. dataset_index = %d\n", dataset_index);
	if (pcd_dataset_occupied[dataset_index] == 0) {
		pcd_log_error("ERROR: Dataset %d is not in use\n", dataset_index);
		return PCD_NOT_FOUND;
	}

	passed = pcd_policy_eval_over_dataset(pcd_datasets[dataset_index], program_owner_id);
	if (passed == PCD_OK)
		pcd_dataset_current = dataset_index;
	return passed;

}

pcd_runtime_pointer_t pcd_dataset_access(uint32_t dataset_index, uint32_t data_index) {
	pcd_instance_t *app_instance;
	uint32_t data_size;
	pcd_payload_t *payload;
	pcd_runtime_pointer_t ret;
	//uint8_t *payload_test;
	int j, k;

	PCD_EVAL_DEFINE_WATCH(watch);

	pcd_eval_stopwatch_start(&watch);

	if (pcd_dataset_occupied[dataset_index] == 0) {
		pcd_log_error("ERROR: pcd_dataset_access: Dataset %d is not in use\n", dataset_index);
		return PCD_NOT_FOUND;
	}

	if (pcd_datasets[dataset_index]->dataset_policy_passed == 0) {
		pcd_log_error("ERROR: pcd_dataset_access: Dataset %d has not yet been performed a policy check\n");
		return PCD_DENINED;
	}

	if (data_index >= pcd_datasets[dataset_index]->data_count) {
		pcd_log_error("ERROR: pcd_dataset_access: Dataset %d does not have that many data (%d/%d)\n", data_index, pcd_datasets[dataset_index]->data_count);
		return PCD_OUT_OF_RANGE;
	}

	app_instance = pcd_app_get_instance();
	if (app_instance == NULL) {
		pcd_log_error("ERROR: pcd_dataset_access: Should have been here!\n");
		return PCD_UNKNOWN;
	}

	pcd_eval_stopwatch_lap("pcd_dataset_access: Preparation", &watch, 1);

	payload = pcd_datasets[dataset_index]->payload_pointers[data_index];
	data_size = sizeof(pcd_payload_t) + payload->data_size + payload->policy_size + payload->tag_size + payload->attribute_size;
	ret = pcd_runtime_copy_data_into_runtime(app_instance, (void *)payload, data_size);
	//payload_test = (uint8_t *)pcd_runtime_app_to_native(app_instance, ret);

	pcd_eval_stopwatch_lap("pcd_dataset_access: Done copying", &watch, 1);

	return ret;
	
}

uint32_t pcd_dataset_release(uint32_t dataset_index) {
	int i;

	if (pcd_dataset_occupied[dataset_index] == 0) {
		pcd_log_error("ERROR: Dataset %d is not in use\n", dataset_index);
		return PCD_NOT_FOUND;
	}

	// Free all payloads
	for (i = 0; i < pcd_datasets[dataset_index]->data_count; i++) {
		free(pcd_datasets[dataset_index]->payload_pointers[i]);
	}

	// Free the dataset struct
	free(pcd_datasets[dataset_index]);

	// Free the occupied bitmap
	pcd_dataset_occupied[dataset_index] = 0;

	return PCD_OK;
}
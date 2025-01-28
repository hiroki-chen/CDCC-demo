#ifndef _PCD_DATASET_H_
#define _PCD_DATASET_H_

#include <stdint.h>

#include "data.h"
#include "dataset.h"
#include "runtime.h"
#include "identity.h"

#ifndef PCD_MAX_AMOUNT_DATASET
#define PCD_MAX_AMOUNT_DATASET	(16)
#endif

typedef struct pcd_dataset_t {
	uint8_t dataset_policy_passed;
	uint32_t data_count;
	uint32_t data_max_count;
	pcd_policy_type_t policy_type;
	pcd_payload_t *payload_pointers[];
} __attribute__((packed)) pcd_dataset_t;

uint32_t pcd_dataset_new(uint32_t dataset_max_size);
uint32_t pcd_dataset_add_data(uint32_t dataset_index, pcd_enc_data_t *input_data);
uint32_t pcd_dataset_check_policy(uint32_t dataset_index, pcd_identity_t *program_owner_id);
pcd_runtime_pointer_t pcd_dataset_access(uint32_t dataset_index, uint32_t data_index);
uint32_t pcd_dataset_release(uint32_t dataset_index);

#endif

#ifndef _PCD_POLICY_H_
#define _PCD_POLICY_H_

#include <stdint.h>

#include "tag.h"
#include "runtime.h"
#include "data.h"
#include "dataset.h"

#include "policy/policy_def.h"

typedef struct _pcd_policy_disc_t {
	pcd_policy_type_t disc_id;
	pcd_module_t *disc_module;
} pcd_policy_disc_t;

#ifndef PCD_POLICY_TYPE_MAX_INDEX
#define PCD_POLICY_TYPE_MAX_INDEX	0x0008
#endif

#ifndef PCD_POLICY_STACK_SIZE
#define PCD_POLICY_STACK_SIZE (4 * 1024 * 1024)
#endif

#ifndef PCD_POLICY_HEAP_SIZE
#define PCD_POLICY_HEAP_SIZE (16 * 1024 * 1024)
#endif

int pcd_policy_eval_over_dataset(pcd_dataset_t *dataset, pcd_identity_t *program_owner_id);
int pcd_policy_load_disc(char *module_buffer, size_t module_size, pcd_policy_type_t *type);

int pcd_policy_eval_output_over_dataset(pcd_dataset_t *dataset,
					void *data, size_t data_size,
					pcd_identity_t *data_owner_id,
					pcd_policy_t *policy,
					void *attributes, uint64_t attribute_size);

#endif

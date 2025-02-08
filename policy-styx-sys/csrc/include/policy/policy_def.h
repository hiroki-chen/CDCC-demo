#ifndef _PCD_POLICY_DEF_H_
#define _PCD_POLICY_DEF_H_

#include <stdint.h>

#include "tag.h"
#include "uuid.h"

typedef uuid_t pcd_policy_type_t;

typedef struct {
	pcd_policy_type_t type;
	uint64_t policy_size;

	uint8_t policy_buffer[];
} __attribute__((packed)) pcd_policy_t;

static inline int pcd_compare_policy_type(pcd_policy_type_t *i1, pcd_policy_type_t *i2) 
{
	return pcd_compare_uuid((uuid_t *)i1, (uuid_t *)i2);
}

#endif

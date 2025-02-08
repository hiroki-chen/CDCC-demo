#ifndef _PCD_IDENTITY_H_
#define _PCD_IDENTITY_H_

// To avoid dependencies, use locally defined uuid_t
// #include <uuid/uuid.h>

#include "uuid.h"

typedef uuid_t pcd_identity_t;

static inline int pcd_compare_identity(const pcd_identity_t *i1, const pcd_identity_t *i2) 
{
	return pcd_compare_uuid((uuid_t *)i1, (uuid_t *)i2);
}

#endif

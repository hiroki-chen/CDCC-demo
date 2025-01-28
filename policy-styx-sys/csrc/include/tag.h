#ifndef _PCD_TAG_H_
#define _PCD_TAG_H_

#include "uuid.h"

typedef uuid_t pcd_tag_t;

static inline int pcd_compare_tag(pcd_tag_t *t1, pcd_tag_t *t2) 
{
	return pcd_compare_uuid((uuid_t *)t1, (uuid_t *)t2);
}

#endif
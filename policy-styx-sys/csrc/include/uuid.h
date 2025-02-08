#ifndef _PCD_UUID_H_
#define _PCD_UUID_H_

#include <string.h>

typedef struct {
	unsigned char value[16];
} __attribute__((packed)) uuid_t;

static inline int pcd_compare_uuid(const uuid_t *i1, const uuid_t *i2) 
{
	return strncmp((char *)i1, (char *)i2, sizeof(uuid_t));
}

#endif

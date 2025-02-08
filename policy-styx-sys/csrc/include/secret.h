#ifndef _SECRET_REQ_H_
#define _SECRET_REQ_H_

#include <stdint.h>

#include "data.h"
#include "identity.h"

typedef enum {
	PCD_SECRET_REQ_REQUEST,
	PCD_SECRET_REQ_REGISTER,
	PCD_SECRET_REQ_RESPONSE_OK,
	PCD_SECRET_REQ_RESPONSE_NOT_FOUND,
	PCD_SECRET_REQ_RESPONSE_DENINED,
	PCD_SECRET_REQ_RESPONSE_REG_OK
} pcd_secret_req_type_t;

typedef struct _pcd_secret_t {
	uint32_t secret_size;
	char secret[];
} __attribute__((packed)) pcd_secret_t;

typedef struct _pcd_secret_req_t {
	pcd_secret_req_type_t req_type;
		pcd_identity_t id;
		pcd_secret_t secret;
} __attribute__((packed)) pcd_secret_req_t;

int pcd_secret_register(pcd_identity_t *id, pcd_secret_t *input_secret);
int pcd_secret_release(pcd_identity_t *id);

// #ifdef PCD_CONFIG_SECRET_REQUESTER
int pcd_secret_fetch(pcd_identity_t *id, pcd_delegator_addr_t *delegator_addr, pcd_secret_t **output_secret);
// #endif

int pcd_secret_retrieve(pcd_identity_t *id, pcd_secret_t **output_secret);

#endif

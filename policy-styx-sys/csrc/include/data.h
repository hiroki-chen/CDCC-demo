#ifndef _PCD_DATA_H_
#define _PCD_DATA_H_

#include <stdint.h>

#include "identity.h"
#include "crypto/crypto.h"

#include "policy/policy_def.h"

#ifdef __cplusplus
extern "C" {
#endif

typedef uint64_t pcd_protocol_version_t;

#define PCD_PROTOCOL_VERSION 0
#define PCD_DELEGATOR_ADDR_MAX_LEN 64

typedef char pcd_delegator_addr_t[PCD_DELEGATOR_ADDR_MAX_LEN];

// Capsulated encrypted data
typedef struct pcd_enc_data {
  pcd_protocol_version_t protocol_version;
  pcd_identity_t owner_id;
  pcd_crypto_algo_t enc_algo;
  uint64_t enc_size;
  pcd_delegator_addr_t delegator_addr;

  // Encrypted payload
  uint8_t encrypted_payload[];
} __attribute__((packed)) pcd_enc_data_t;

// Plaintext payload before encryption
typedef struct pcd_payload {
  // Data + Policy + Tags + Attributes
  uint64_t data_size;
  uint64_t policy_size;
  uint64_t tag_size;
  uint64_t attribute_size;

  uint8_t payload[];
} __attribute__((packed)) pcd_payload_t;

static inline pcd_policy_t *pcd_get_policy_from_payload(
    pcd_payload_t *payload) {
  return (pcd_policy_t *)(((char *)payload->payload) + payload->data_size);
}

#ifdef __cplusplus
}
#endif

#endif

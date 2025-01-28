#ifndef _PCD_CRYPTO_SHA256_H_
#define _PCD_CRYPTO_SHA256_H_

#include <stdint.h>
#include <stddef.h>

typedef uint8_t pcd_sha256_t[32];

int pcd_crypto_sha256_hash_buffer(uint8_t *buffer, size_t size, pcd_sha256_t *output_hash);

#endif
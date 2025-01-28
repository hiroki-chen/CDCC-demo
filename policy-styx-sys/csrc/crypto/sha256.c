#include <stdint.h>
#include <openssl/evp.h>
#include <openssl/sha.h>

#include "crypto/sha256.h"
#include "crypto/sha256_backend.h"

int pcd_crypto_sha256_hash_buffer(uint8_t *buffer, size_t size,
                                  pcd_sha256_t *output_hash) {
  return pcd_crypto_backend_sha256_hash_buffer(buffer, size, output_hash);
}

int pcd_crypto_backend_sha256_hash_buffer(uint8_t *buffer, size_t size,
                                          pcd_sha256_t *output_hash) {
  EVP_MD_CTX *ctx;
  unsigned int hash_len;

  if (!(ctx = EVP_MD_CTX_new())) {
    return -1;
  }

  if (!EVP_DigestInit_ex(ctx, EVP_sha256(), NULL)) {
    EVP_MD_CTX_free(ctx);
    return -1;
  }

  if (!EVP_DigestUpdate(ctx, buffer, size)) {
    EVP_MD_CTX_free(ctx);
    return -1;
  }

  if (!EVP_DigestFinal_ex(ctx, (unsigned char *)output_hash, &hash_len)) {
    EVP_MD_CTX_free(ctx);
    return -1;
  }

  EVP_MD_CTX_free(ctx);

  return 0;
}
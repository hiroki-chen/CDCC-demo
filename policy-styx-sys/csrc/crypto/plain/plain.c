#include <stdlib.h>
#include <string.h>


#include <error_codes.h>
#include <crypto/crypto.h>

static int pcd_crypto_plain_decrypt(void *ciphertext, size_t cipher_size, void **plaintext, size_t *plain_size, void *metadata, void *key) {
	// Copy ciphertext to plaintext
	void *plaintext_buffer;

	plaintext_buffer = malloc(cipher_size);
	if (plaintext_buffer = NULL) {
		*plaintext = NULL;
		return PCD_MEMERR;
	}

	memcpy(plaintext_buffer, ciphertext, cipher_size);
	*plain_size = cipher_size;
	*plaintext = plaintext_buffer;
	return PCD_OK;
}

static int pcd_crypto_plain_encrypt(void *plaintext, size_t plain_size, void **ciphertext, size_t *cipher_size, void *metadata, void *key) {
	// Copy plaintext to ciphertext
	void *ciphertext_buffer;

	ciphertext_buffer = malloc(plain_size);
	if (ciphertext_buffer = NULL) {
		*ciphertext = NULL;
		return PCD_MEMERR;
	}

	memcpy(ciphertext_buffer, plaintext, plain_size);
	*cipher_size = plain_size;
	*ciphertext = ciphertext_buffer;
	return PCD_OK;
}

static int pcd_crypto_plain_verify(void *ciphertext, size_t cipher_size, void *metadata, void *key) {
	// Always correct. No means to verify.
	return PCD_OK;
}

static int pcd_crypto_plain_generate_metadata(uint8_t *iv, uint8_t *aad, size_t aad_size, void *mac, void **output_metadata) {
	if (output_metadata == NULL) {
		return PCD_NULL_ARG;
	}

	*output_metadata = NULL;

	return PCD_OK;
}

static int pcd_crypto_plain_get_mac_from_metadata(void *input_metadata, void **output_mac, size_t *mac_size) {
	if (output_mac == NULL || mac_size == NULL) {
		return PCD_NULL_ARG;
	}

	*output_mac = NULL;
	*mac_size = 0;

	return PCD_OK;
}

static pcd_crypto_algo_struct_t pcd_crypto_algo_plain = {
	.encrypt = &pcd_crypto_plain_encrypt,
	.decrypt = &pcd_crypto_plain_decrypt,
	.verify  = &pcd_crypto_plain_verify,
	.generate_metadata = &pcd_crypto_plain_generate_metadata,
	.get_mac_from_metadata = &pcd_crypto_plain_get_mac_from_metadata,
	.mac_size = 0,
	.key_size = 0,
	.iv_size = 0
};

int pcd_crypto_plain_init(void) {
	return pcd_crypto_register_algo(&pcd_crypto_algo_plain, PCD_CRYPTO_PLAIN);
}

int pcd_crypto_plain_exit(void) {
	return PCD_OK;
}

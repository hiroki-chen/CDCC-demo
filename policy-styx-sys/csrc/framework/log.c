#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdarg.h>

#include "identity.h"

#include "log_backend.h"

void pcd_print_error(const char *string) {
    fprintf(stderr, "%s\n", string);
}

void pcd_log_error(const char *fmt, ...) {
    char buf[BUFSIZ] = { '\0' };
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, BUFSIZ, fmt, ap);
    va_end(ap);
    pcd_print_error(buf);
}

void pcd_log_warning(const char *fmt, ...) {
    char buf[BUFSIZ] = { '\0' };
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, BUFSIZ, fmt, ap);
    va_end(ap);
    pcd_print_error(buf);
}

void pcd_log(const char *fmt, ...) {
    char buf[BUFSIZ] = { '\0' };
    va_list ap;
    va_start(ap, fmt);
    vsnprintf(buf, BUFSIZ, fmt, ap);
    va_end(ap);
    pcd_print(buf);
}


void pcd_print_id(pcd_identity_t *id) {
	uint8_t *id_ptr = (uint8_t *)id;
	int i;
	pcd_log("%08x-", ((uint32_t*)(id_ptr))[0]);
	pcd_log("%04x-", *((uint16_t*)(&id_ptr[4])));
	pcd_log("%04x-", *((uint16_t*)(&id_ptr[6])));
	for (i = 8; i < 10; i++) {
		pcd_log("%02x", id_ptr[i]);
	}
	pcd_log("-");
	for (i = 10; i < 16; i++) {
		pcd_log("%02x", id_ptr[i]);
	}
}
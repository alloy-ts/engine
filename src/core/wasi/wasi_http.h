#ifndef ALLOY_WASI_HTTP_H
#define ALLOY_WASI_HTTP_H

#include <stdint.h>
#include <stddef.h>

typedef uint32_t wasi_http_handle_t;

int wasi_http_handle_request(const char *cap_token, const char *url, const char *method, wasi_http_handle_t *handle);

#endif

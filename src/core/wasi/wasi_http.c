#include "wasi_http.h"
#include "../capability.h"
#include <stdio.h>

int wasi_http_handle_request(const char *cap_token, const char *url, const char *method, wasi_http_handle_t *handle) {
    if (!alloy_capability_validate(cap_token, CAP_WEB_FETCH, url)) {
        return -1;
    }
    printf("[WASI HTTP] Proxying request to %s\n", url);
    *handle = 1;
    return 0;
}

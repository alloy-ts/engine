#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct {
    char *image_digest;
    int   verified;
} verification_cache_entry_t;

static verification_cache_entry_t *cache = NULL;
static int cache_count = 0;

int alloy_container_verify_image(const char *image_ref) {
    // 1. Check cache by digest (simplified: using ref for now)
    for (int i = 0; i < cache_count; i++) {
        if (strcmp(cache[i].image_digest, image_ref) == 0) return cache[i].verified;
    }

    // 2. cosign keyless verification via Sigstore
    // cosign verify --certificate-identity-regexp=.* --certificate-oidc-issuer=https://token.actions.githubusercontent.com <image>
    char cmd[1024];
    snprintf(cmd, sizeof(cmd), "cosign verify --certificate-identity-regexp=.* --certificate-oidc-issuer=https://token.actions.githubusercontent.com %s > /dev/null 2>&1", image_ref);

    int result = system(cmd);
    int verified = (result == 0);

    // 3. Update cache
    cache = realloc(cache, (cache_count + 1) * sizeof(verification_cache_entry_t));
    cache[cache_count].image_digest = strdup(image_ref);
    cache[cache_count].verified = verified;
    cache_count++;

    return verified;
}

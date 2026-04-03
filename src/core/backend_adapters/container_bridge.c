#include <stdlib.h>
#include <string.h>
#include <stdio.h>

typedef struct {
    char *image_digest;
    int verified;
} alloy_container_cache_entry_t;

static alloy_container_cache_entry_t *verification_cache = NULL;
static int cache_count = 0;
static pthread_mutex_t cache_mutex = PTHREAD_MUTEX_INITIALIZER;

int alloy_container_verify_image(const char *image_ref) {
    pthread_mutex_lock(&cache_mutex);
    for (int i = 0; i < cache_count; i++) {
        if (strcmp(verification_cache[i].image_digest, image_ref) == 0) {
            int v = verification_cache[i].verified;
            pthread_mutex_unlock(&cache_mutex);
            return v ? 0 : -1;
        }
    }
    pthread_mutex_unlock(&cache_mutex);

    // Securely spawn cosign without using system() shell
    pid_t pid = fork();
    if (pid == 0) {
        execlp("cosign", "cosign", "verify", "--certificate-identity-regexp=.*", "--certificate-oidc-issuer=https://token.actions.githubusercontent.com", image_ref, (char *)NULL);
        _exit(127);
    }
    int status;
    waitpid(pid, &status, 0);
    int res = WEXITSTATUS(status);

    pthread_mutex_lock(&cache_mutex);
    verification_cache = realloc(verification_cache, (cache_count + 1) * sizeof(alloy_container_cache_entry_t));
    verification_cache[cache_count].image_digest = strdup(image_ref);
    verification_cache[cache_count].verified = (res == 0);
    cache_count++;
    pthread_mutex_unlock(&cache_mutex);

    return res == 0 ? 0 : -1;
}

int alloy_container_run_capability(const char *name, const char *image, char **cmd, void *grants) {
    char container_name[128];
    snprintf(container_name, sizeof(container_name), "alloy-%s-%d", name, getpid());

    // Securely spawn podman without using system() shell
    pid_t pid = fork();
    if (pid == 0) {
        execlp("podman", "podman", "run", "--name", container_name, "--rm", image, (char *)NULL);
        _exit(127);
    }
    int status;
    waitpid(pid, &status, 0);

    return WEXITSTATUS(status);
}

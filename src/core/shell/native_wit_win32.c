#include "native_wit.h"
#ifdef _WIN32
#include <windows.h>
#include <stdio.h>

int alloy_native_wit_ctx_create(alloy_native_wit_ctx_t *out) {
    HANDLE job = CreateJobObject(NULL, NULL);
    out->dir_fd_count = 1;
    out->dir_fds = (int *)job; // Store job object handle
    out->cwd_fd = 0;
    out->env_count = 1;
    out->envp = malloc(2 * sizeof(char *));
    out->envp[0] = strdup("NODE_ENV=development");
    out->envp[1] = NULL;
    return 0;
}

int alloy_native_wit_ctx_apply(const alloy_native_wit_ctx_t *ctx) {
    HANDLE job = (HANDLE)ctx->dir_fds;
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION limitInfo = { 0 };
    limitInfo.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_DIE_ON_UNHANDLED_EXCEPTION;
    SetInformationJobObject(job, JobObjectExtendedLimitInformation, &limitInfo, sizeof(limitInfo));
    AssignProcessToJobObject(job, GetCurrentProcess());
    return 0;
}

void alloy_native_wit_ctx_free(alloy_native_wit_ctx_t *ctx) {
    CloseHandle((HANDLE)ctx->dir_fds);
    for (int i = 0; i < ctx->env_count; i++) free(ctx->envp[i]);
    free(ctx->envp);
}
#endif

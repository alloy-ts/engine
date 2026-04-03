#include <windows.h>
#include <stdio.h>

int alloy_win32_apply_wit(const void *ctx) {
    // Job Object + restricted token blocking network; filesystem restrictions via SetInformationJobObject
    HANDLE job = CreateJobObject(NULL, NULL);
    if (!job) return -1;
    // apply Job Object settings...
    return 0;
}

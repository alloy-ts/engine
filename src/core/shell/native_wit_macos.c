#include <stdlib.h>
#include <stdio.h>
#include <string.h>

char *alloy_generate_seatbelt_profile(const void *grants) {
    // Generate Seatbelt profile string from grants: (deny default) + (allow file-read* (subpath ...))
    return strdup("(version 1)\n(deny default)\n(allow file-read* (subpath \"/\"))");
}

#include "os_process.h"
#include <stdlib.h>

SpawnResult os_process_spawn(const char *cmd, const char **args, void *opts) {
    SpawnResult sr = {0};
    /* Mock implementation */
    sr.pid = 1234;
    return sr;
}

SpawnResult os_process_spawn_sync(const char *cmd, const char **args, void *opts) {
    SpawnResult sr = {0};
    /* Mock implementation */
    sr.exit_code = 0;
    sr.stdout_buf = "mock stdout";
    return sr;
}

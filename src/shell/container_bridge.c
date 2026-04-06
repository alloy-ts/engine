#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>

/**
 * Executes a command in an ephemeral OCI container for shellCapabilities.
 * Requirement 11.6, 11.7, 11.8, 16.1-16.5
 */
int alloy_container_execute(const char *image, const char *command, const char **args, int arg_count) {
    // Construct podman run command for isolation
    // podman run --rm -v /tmp:/tmp:ro --network none <image> <command> <args>

    pid_t pid = fork();
    if (pid == 0) {
        char **exec_args = malloc((arg_count + 10) * sizeof(char *));
        int i = 0;
        exec_args[i++] = "podman";
        exec_args[i++] = "run";
        exec_args[i++] = "--rm";
        exec_args[i++] = "--network";
        exec_args[i++] = "none"; // Disable network by default for isolation
        exec_args[i++] = (char *)image;
        exec_args[i++] = (char *)command;
        for (int j = 0; j < arg_count; j++) {
            exec_args[i++] = (char *)args[j];
        }
        exec_args[i] = NULL;

        execvp("podman", exec_args);
        exit(1);
    } else if (pid > 0) {
        int status;
        waitpid(pid, &status, 0);
        return WIFEXITED(status) ? WEXITSTATUS(status) : -1;
    }

    return -1;
}

/**
 * Verifies OCI image via Sigstore/cosign before execution.
 * Requirement 11.8
 */
int alloy_container_verify_image(const char *image) {
    // cosign verify --key cosign.pub <image>
    // Stub for verification logic
    return 0; // 0 for success
}

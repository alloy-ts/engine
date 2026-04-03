#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <sys/wait.h>

int alloy_container_verify_image(const char *image_ref) {
    pid_t pid = fork();
    if (pid == 0) {
        char *argv[] = {
            "cosign", "verify",
            "--certificate-identity-regexp=.*",
            "--certificate-oidc-issuer=https://token.actions.githubusercontent.com",
            (char *)image_ref,
            NULL
        };
        // Redirect stdout/err to null to avoid noise
        int fd = open("/dev/null", O_WRONLY);
        dup2(fd, STDOUT_FILENO); dup2(fd, STDERR_FILENO);
        execvp("cosign", argv);
        _exit(1);
    } else {
        int status;
        waitpid(pid, &status, 0);
        return WIFEXITED(status) && WEXITSTATUS(status) == 0;
    }
}

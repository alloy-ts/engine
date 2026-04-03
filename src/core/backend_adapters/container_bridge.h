#ifndef ALLOY_CONTAINER_BRIDGE_H
#define ALLOY_CONTAINER_BRIDGE_H

int alloy_container_verify_image(const char *image_ref);
int alloy_container_run_capability(const char *name, const char *image, char **cmd, void *grants);

#endif /* ALLOY_CONTAINER_BRIDGE_H */

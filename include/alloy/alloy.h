#ifndef ALLOYSCRIPT_ALLOY_H
#define ALLOYSCRIPT_ALLOY_H

#include "config.h"
#include "capability.h"
#include "ipc_protocol.h"
#include "mquickjs.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Main Public API */

typedef struct alloy_runtime alloy_runtime_t;

alloy_runtime_t *alloy_engine_create(void);
void alloy_engine_destroy(alloy_runtime_t *rt);
void alloy_run(alloy_runtime_t *rt);

JSValue alloy_secure_eval(alloy_runtime_t *rt, const char *script);

#ifdef __cplusplus
}
#endif

#endif // ALLOYSCRIPT_ALLOY_H

#ifndef ALLOY_PLUGIN_H
#define ALLOY_PLUGIN_H

#include "engine.h"

typedef struct RuntimePlugin {
    const char *name;
    int (*on_init)(alloy_runtime_t *rt);
    int (*on_eval_begin)(alloy_runtime_t *rt, const char *source);
    int (*on_eval_end)(alloy_runtime_t *rt, const char *result_json);
    int (*on_destroy)(alloy_runtime_t *rt);
} RuntimePlugin;

#endif

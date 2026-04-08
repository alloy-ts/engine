#ifndef ALLOYSCRIPT_PLUGIN_TYPES_H
#define ALLOYSCRIPT_PLUGIN_TYPES_H

#include "js_engine.h"

typedef struct {
    const char *name;
    const char **depends_on;
    int (*on_init)(AlloyJSEngine *engine);
    int (*on_eval_begin)(AlloyJSEngine *engine, const char *source);
    int (*on_eval_end)(AlloyJSEngine *engine);
    int (*on_destroy)(AlloyJSEngine *engine);
} RuntimePlugin;

#endif // ALLOYSCRIPT_PLUGIN_TYPES_H

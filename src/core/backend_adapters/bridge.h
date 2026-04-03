#ifndef ALLOYSCRIPT_BRIDGE_H
#define ALLOYSCRIPT_BRIDGE_H

#include "engine.h"

AlloyJSEngine *alloy_mqjs_engine_create(size_t mem_size, alloy_runtime_t *rt);
AlloyJSEngine *alloy_webview_engine_create(alloy_runtime_t *rt);
JSContext *alloy_mqjs_get_context(AlloyJSEngine *engine);

#endif

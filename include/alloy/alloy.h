#ifndef ALLOYSCRIPT_H
#define ALLOYSCRIPT_H
#include "mquickjs.h"
typedef struct AlloyRuntime AlloyRuntime;
AlloyRuntime* alloy_engine_create(void);
void alloy_engine_destroy(AlloyRuntime *rt);
JSContext* get_rt_ctx(AlloyRuntime *rt);
#endif

#include "../plugin.h"
#include <stdio.h>
#include <stdlib.h>

extern int async_plugin_on_init(AlloyJSEngine *engine, void *userdata);

static RuntimePlugin alloy_async_plugin = {
    .name = "alloy:async",
    .version = "1.0.0",
    .on_init = async_plugin_on_init,
};

RuntimePlugin *alloy_async_plugin_init(void) {
    return &alloy_async_plugin;
}

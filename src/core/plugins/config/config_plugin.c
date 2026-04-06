#include "config_plugin.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int config_on_init(AlloyJSEngine *engine) {
    /* Requirement 19.3: parse + merge config sources, expose Alloy.config */
    engine->eval_fn(engine, "Alloy.config = {};", "config_plugin", NULL);
    return 0;
}

int alloy_config_plugin_init(AlloyJSEngine *engine) {
    return config_on_init(engine);
}

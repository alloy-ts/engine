#include "engine.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

void alloy_event_emit_json(alloy_runtime_t *rt, const char *name, const char *payload_json) {
    // Find listeners and enqueue
}

void alloy_poll_io(alloy_runtime_t *rt) {
    // Poll signals, timers, webview
}

void alloy_drain_next_tick(alloy_runtime_t *rt) {
    // Drain rt->next_tick_queue
}

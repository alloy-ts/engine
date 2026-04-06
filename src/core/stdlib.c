#include <math.h>
#include <stdio.h>
#include <string.h>
#include "mqjs_stdlib.c"
#include "mquickjs_build.h"
#include "../web/urn.h"

#define ALLOY_STDLIB

/* ── URN prototype properties ───────────────────────────────────────────── */
static const JSPropDef js_urn_proto[] = {
    JS_CGETSET_DEF("href",     js_urn_get_href, NULL),
    JS_CGETSET_DEF("nid",      js_urn_get_nid,  NULL),
    JS_CGETSET_DEF("nss",      js_urn_get_nss,  NULL),
    JS_CFUNC_DEF("toString",   0, js_urn_toString),
    JS_CFUNC_DEF("equals",     1, js_urn_equals),
    JS_PROP_END,
};

/* URN has no static class-level properties */
static const JSClassDef js_urn_class =
    JS_CLASS_DEF("URN", 1, js_urn_constructor, JS_CLASS_URN,
                 NULL, js_urn_proto, NULL, js_urn_finaliser);

/* ── DID prototype properties ───────────────────────────────────────────── */
static const JSPropDef js_did_proto[] = {
    /* Inherited from URN: href, nid, nss — overridden here for DID class check */
    JS_CGETSET_DEF("href",      js_did_get_href,      NULL),
    JS_CGETSET_DEF("nid",       js_did_get_nid,       NULL),
    JS_CGETSET_DEF("nss",       js_did_get_nss,       NULL),
    /* DID-specific */
    JS_CGETSET_DEF("method",    js_did_get_method,    NULL),
    JS_CGETSET_DEF("methodId",  js_did_get_method_id, NULL),
    JS_CGETSET_DEF("document",  js_did_get_document,  js_did_set_document),
    JS_CFUNC_DEF("toString",    0, js_did_toString),
    JS_CFUNC_DEF("equals",      1, js_did_equals),
    JS_PROP_END,
};

/* DID inherits from URN (parent_class = &js_urn_class) */
static const JSClassDef js_did_class =
    JS_CLASS_DEF("DID", 1, js_did_constructor, JS_CLASS_DID,
                 NULL, js_did_proto, &js_urn_class, js_did_finaliser);



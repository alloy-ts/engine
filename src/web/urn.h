/*
 * AlloyScript — URN / DID class declarations
 *
 * Shared header included by urn.c, did.c, and stdlib.c.
 * Defines class IDs, data structs, and the function signatures
 * that the stdlib table references by name.
 */
#ifndef ALLOY_URN_H
#define ALLOY_URN_H

#include "../../packages/mquickjs/mquickjs.h"

/* ── Class IDs ──────────────────────────────────────────────────────────── */
#define JS_CLASS_URN (JS_CLASS_USER + 0)
#define JS_CLASS_DID (JS_CLASS_USER + 1)

/* ── Data structs ───────────────────────────────────────────────────────── */

/*
 * UrnData — backing store for a URN instance.
 * Represents:  urn:<nid>:<nss>
 */
typedef struct {
    char value[512]; /* full string, e.g. "urn:isbn:0451450523" */
    char nid[64];    /* namespace identifier, e.g. "isbn"        */
    char nss[448];   /* namespace-specific string                 */
} UrnData;

/*
 * DidData — backing store for a DID instance (extends URN).
 * Represents:  did:<method>:<method-specific-id>
 * The base URN is stored with nid="did", nss="<method>:<id>".
 */
typedef struct {
    UrnData base;         /* inherits URN fields; base.nid == "did" */
    char    method[64];   /* DID method, e.g. "example"             */
    char    method_id[448]; /* method-specific identifier           */
    char   *document;     /* optional DID Document JSON (heap-alloc) */
} DidData;

/* ── URN C functions (referenced by name in stdlib.c) ───────────────────── */
JSValue js_urn_constructor(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
void    js_urn_finaliser(JSContext *ctx, void *opaque);
JSValue js_urn_get_href(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_urn_get_nid(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_urn_get_nss(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_urn_toString(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_urn_equals(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

/* ── DID C functions ────────────────────────────────────────────────────── */
JSValue js_did_constructor(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
void    js_did_finaliser(JSContext *ctx, void *opaque);
JSValue js_did_get_href(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_did_get_nid(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_did_get_nss(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_did_get_method(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_did_get_method_id(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_did_get_document(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_did_set_document(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_did_toString(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);
JSValue js_did_equals(JSContext *ctx, JSValue *this_val, int argc, JSValue *argv);

#endif /* ALLOY_URN_H */

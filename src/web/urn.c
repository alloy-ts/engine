/*
 * AlloyScript — URN class implementation
 *
 * Represents a Uniform Resource Name per RFC 8141:
 *   urn:<NID>:<NSS>
 *
 * JS API:
 *   new URN("urn:isbn:0451450523")
 *   urn.href      // "urn:isbn:0451450523"
 *   urn.nid       // "isbn"
 *   urn.nss       // "0451450523"
 *   urn.toString() // same as href
 *   urn.equals(other) // RFC 8141 case-insensitive NID, case-sensitive NSS
 */
#include "urn.h"
#include <stdlib.h>
#include <string.h>
#include <strings.h> /* strcasecmp */

/* ── Internal parser ────────────────────────────────────────────────────── */

/*
 * Parse "urn:<nid>:<nss>" into d->nid and d->nss.
 * Stores the full string in d->value.
 * Returns 0 on success, -1 if the string is not a valid URN.
 */
static int urn_parse(UrnData *d, const char *raw) {
    if (!raw || strncasecmp(raw, "urn:", 4) != 0)
        return -1;

    strncpy(d->value, raw, sizeof(d->value) - 1);
    d->value[sizeof(d->value) - 1] = '\0';

    const char *rest = raw + 4;
    const char *colon = strchr(rest, ':');
    if (!colon || colon == rest) /* NID must be non-empty */
        return -1;

    size_t nid_len = (size_t)(colon - rest);
    if (nid_len >= sizeof(d->nid))
        return -1;

    memcpy(d->nid, rest, nid_len);
    d->nid[nid_len] = '\0';

    strncpy(d->nss, colon + 1, sizeof(d->nss) - 1);
    d->nss[sizeof(d->nss) - 1] = '\0';

    return 0;
}

/* ── Constructor ────────────────────────────────────────────────────────── */

JSValue js_urn_constructor(JSContext *ctx, JSValue *this_val,
                            int argc, JSValue *argv) {
    if (!(argc & FRAME_CF_CTOR))
        return JS_ThrowTypeError(ctx, "URN must be called with new");
    argc &= ~FRAME_CF_CTOR;

    if (argc < 1)
        return JS_ThrowTypeError(ctx, "URN requires a string argument");

    JSValue obj = JS_NewObjectClassUser(ctx, JS_CLASS_URN);
    UrnData *d = calloc(1, sizeof(UrnData));
    if (!d) return JS_ThrowOutOfMemory(ctx);
    JS_SetOpaque(ctx, obj, d);

    JSCStringBuf buf;
    const char *raw = JS_ToCString(ctx, argv[0], &buf);
    if (!raw) { free(d); return JS_EXCEPTION; }

    if (urn_parse(d, raw) != 0) {
        free(d);
        return JS_ThrowTypeError(ctx,
            "invalid URN: expected 'urn:<nid>:<nss>'");
    }
    return obj;
}

/* ── Finaliser ──────────────────────────────────────────────────────────── */

void js_urn_finaliser(JSContext *ctx, void *opaque) {
    (void)ctx;
    free(opaque);
}

/* ── Getters ────────────────────────────────────────────────────────────── */

/* Accepts both URN and DID class IDs so DID can reuse these getters. */
static UrnData *urn_check(JSContext *ctx, JSValue *this_val) {
    int cid = JS_GetClassID(ctx, *this_val);
    if (cid != JS_CLASS_URN && cid != JS_CLASS_DID) {
        JS_ThrowTypeError(ctx, "expected URN or DID instance");
        return NULL;
    }
    return (UrnData *)JS_GetOpaque(ctx, *this_val);
}

JSValue js_urn_get_href(JSContext *ctx, JSValue *this_val,
                         int argc, JSValue *argv) {
    (void)argc; (void)argv;
    UrnData *d = urn_check(ctx, this_val);
    if (!d) return JS_EXCEPTION;
    return JS_NewString(ctx, d->value);
}

JSValue js_urn_get_nid(JSContext *ctx, JSValue *this_val,
                        int argc, JSValue *argv) {
    (void)argc; (void)argv;
    UrnData *d = urn_check(ctx, this_val);
    if (!d) return JS_EXCEPTION;
    return JS_NewString(ctx, d->nid);
}

JSValue js_urn_get_nss(JSContext *ctx, JSValue *this_val,
                        int argc, JSValue *argv) {
    (void)argc; (void)argv;
    UrnData *d = urn_check(ctx, this_val);
    if (!d) return JS_EXCEPTION;
    return JS_NewString(ctx, d->nss);
}

/* ── Methods ────────────────────────────────────────────────────────────── */

JSValue js_urn_toString(JSContext *ctx, JSValue *this_val,
                         int argc, JSValue *argv) {
    return js_urn_get_href(ctx, this_val, argc, argv);
}

/*
 * urn.equals(other) — RFC 8141 §3 equivalence:
 *   NID comparison is case-insensitive.
 *   NSS comparison is case-sensitive.
 */
JSValue js_urn_equals(JSContext *ctx, JSValue *this_val,
                       int argc, JSValue *argv) {
    if (argc < 1) return JS_FALSE;

    UrnData *a = urn_check(ctx, this_val);
    if (!a) return JS_EXCEPTION;

    int other_cid = JS_GetClassID(ctx, argv[0]);
    if (other_cid != JS_CLASS_URN && other_cid != JS_CLASS_DID)
        return JS_FALSE;

    UrnData *b = (UrnData *)JS_GetOpaque(ctx, argv[0]);
    return JS_NewBool(
        strcasecmp(a->nid, b->nid) == 0 &&
        strcmp(a->nss, b->nss) == 0
    );
}

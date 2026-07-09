/* jianyi C ABI — hand-maintained (three functions; no cbindgen).
 * JSON strings in, JSON strings out. Every JianyiResult must be freed
 * exactly once with jianyi_result_free. */

#ifndef JIANYI_H
#define JIANYI_H

#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct JianyiResult {
    bool success;
    char *data;  /* JSON payload on success, else NULL */
    char *error; /* message on failure, else NULL */
} JianyiResult;

/* Cast the I Ching. Request: {"timestamp"?: rfc3339, "question"?: string}.
 * Omit timestamp for now; omit question for an open cast. Response carries
 * the normalized seed (store verbatim), lines, changing_lines, fingerprint,
 * and resolved primary/resulting/nuclear hexagram records. */
JianyiResult jianyi_iching_cast(const char *request_json);

/* Look up a hexagram. Request: {"number": 1-64} or {"binary": "6 chars"}. */
JianyiResult jianyi_iching_hexagram(const char *request_json);

void jianyi_result_free(JianyiResult result);

#ifdef __cplusplus
}
#endif

#endif /* JIANYI_H */

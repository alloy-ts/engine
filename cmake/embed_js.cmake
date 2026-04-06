# embed_js.cmake — Convert a JS file into a C string literal header
# Usage: cmake -P embed_js.cmake INPUT=foo.js OUTPUT=foo_embed.h

if(NOT INPUT OR NOT OUTPUT)
    message(FATAL_ERROR "Usage: cmake -P embed_js.cmake INPUT=<file> OUTPUT=<file>")
endif()

file(READ "${INPUT}" JS_CONTENT)

# Escape backslashes, double-quotes, and newlines for C string literal
string(REPLACE "\\" "\\\\" JS_CONTENT "${JS_CONTENT}")
string(REPLACE "\"" "\\\"" JS_CONTENT "${JS_CONTENT}")
string(REPLACE "\n" "\\n\"\n\"" JS_CONTENT "${JS_CONTENT}")

file(WRITE "${OUTPUT}" "\"${JS_CONTENT}\"\n")

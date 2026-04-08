cmake_minimum_required(VERSION 3.20)
project(AlloyScript C CXX)

set(CMAKE_C_STANDARD 11)
set(CMAKE_CXX_STANDARD 20)

# ── Feature flags ────────────────────────────────────────────────────────────
option(ALLOY_ENABLE_WEB "Enable alloy:web hidden WebView process + web API shims" ON)
option(ALLOY_ENABLE_GUI "Enable native GUI plugin" OFF)

# ── LTO / ThinLTO ────────────────────────────────────────────────────────────
include(CheckIPOSupported)
check_ipo_supported(RESULT IPO_SUPPORTED OUTPUT IPO_ERROR)
if(IPO_SUPPORTED)
    set(CMAKE_INTERPROCEDURAL_OPTIMIZATION TRUE)
    message(STATUS "[alloy] ThinLTO enabled")
else()
    message(WARNING "[alloy] LTO not supported: ${IPO_ERROR}")
endif()

# ── Paths ────────────────────────────────────────────────────────────────────
set(ALLOY_ROOT       ${CMAKE_SOURCE_DIR})
set(MQJS_DIR         ${ALLOY_ROOT}/packages/mquickjs)
set(WEBVIEW_DIR      ${ALLOY_ROOT}/packages/webview)
set(INCLUDE_DIR      ${ALLOY_ROOT}/include)
set(SRC_CORE         ${ALLOY_ROOT}/src/core)
set(SRC_WEB          ${ALLOY_ROOT}/src/web)
set(SRC_SHIMS        ${ALLOY_ROOT}/src/shims)

# ── Find libsodium (required) ────────────────────────────────────────────────
find_library(SODIUM_LIB sodium REQUIRED)
find_path(SODIUM_INCLUDE sodium.h PATH_SUFFIXES sodium)
if(NOT SODIUM_LIB OR NOT SODIUM_INCLUDE)
    message(FATAL_ERROR "libsodium not found. Install with: brew install libsodium (macOS) or apt install libsodium-dev (Linux)")
endif()
message(STATUS "[alloy] libsodium: ${SODIUM_LIB}")

# ── MicroQuickJS static library ──────────────────────────────────────────────
add_library(mquickjs STATIC
    ${MQJS_DIR}/mquickjs.c
    ${MQJS_DIR}/dtoa.c
    ${MQJS_DIR}/libm.c
    ${MQJS_DIR}/cutils.c
)
target_include_directories(mquickjs PUBLIC ${MQJS_DIR})
target_compile_options(mquickjs PRIVATE -O2 -fno-strict-aliasing)

# ── Build the MQJS stdlib (generates alloy_stdlib_gen.c via mquickjs_build) ──
# The stdlib builder is a host tool that generates the atom table.
# It outputs a complete .c file with js_stdlib_table + JSSTDLibraryDef.
add_executable(mqjs_stdlib_builder
    ${SRC_CORE}/stdlib.c
    ${SRC_WEB}/urn.c
    ${SRC_WEB}/did.c
    ${MQJS_DIR}/mquickjs_build.c
    ${MQJS_DIR}/cutils.c
    ${MQJS_DIR}/dtoa.c
)
target_include_directories(mqjs_stdlib_builder PRIVATE
    ${MQJS_DIR}
    ${SRC_WEB}
)
target_compile_definitions(mqjs_stdlib_builder PRIVATE ALLOY_STDLIB)

add_custom_command(
    OUTPUT  ${SRC_CORE}/alloy_stdlib_gen.c
    COMMAND mqjs_stdlib_builder > ${SRC_CORE}/alloy_stdlib_gen.c
    DEPENDS mqjs_stdlib_builder ${SRC_CORE}/stdlib.c
    COMMENT "Generating AlloyScript MQJS stdlib atom table"
)
add_custom_target(gen_stdlib DEPENDS ${SRC_CORE}/alloy_stdlib_gen.c)

# ── TS → ES5 → C string literal header pipeline ──────────────────────────────
#
# For each runtime TypeScript source:
#   1. bun run scripts/transpile.ts <input.ts> <output.es5.js>
#      Strips types, downlevels to ES5, polyfills async/await.
#   2. cmake -P cmake/embed_js.cmake INPUT=<output.es5.js> OUTPUT=<embed.h>
#      Converts the ES5 JS to a C string literal header.
#
function(transpile_and_embed TS_INPUT ES5_OUTPUT EMBED_OUTPUT)
    # Step 1: TypeScript → ES5
    add_custom_command(
        OUTPUT  ${ES5_OUTPUT}
        COMMAND bun run ${ALLOY_ROOT}/scripts/transpile.ts
                        ${TS_INPUT}
                        ${ES5_OUTPUT}
        DEPENDS ${TS_INPUT} ${ALLOY_ROOT}/src/core/transpiler.ts
        WORKING_DIRECTORY ${ALLOY_ROOT}
        COMMENT "Transpiling ${TS_INPUT}"
    )
    # Step 2: ES5 → C string literal header
    add_custom_command(
        OUTPUT  ${EMBED_OUTPUT}
        COMMAND ${CMAKE_COMMAND}
                -DINPUT=${ES5_OUTPUT}
                -DOUTPUT=${EMBED_OUTPUT}
                -P ${ALLOY_ROOT}/cmake/embed_js.cmake
        DEPENDS ${ES5_OUTPUT}
        COMMENT "Embedding ${ES5_OUTPUT}"
    )
endfunction()

transpile_and_embed(
    ${SRC_CORE}/runtime_bootstrap.ts
    ${CMAKE_BINARY_DIR}/gen/runtime_bootstrap.es5.js
    ${SRC_CORE}/bootstrap_js_embed.h
)
transpile_and_embed(
    ${SRC_SHIMS}/web_api_shims.ts
    ${CMAKE_BINARY_DIR}/gen/web_api_shims.es5.js
    ${SRC_CORE}/web_shims_js_embed.h
)
transpile_and_embed(
    ${ALLOY_ROOT}/src/gui/gui_shims.ts
    ${CMAKE_BINARY_DIR}/gen/gui_shims.es5.js
    ${SRC_CORE}/gui_shims_js_embed.h
)
add_custom_target(gen_embedded_js
    DEPENDS
        ${SRC_CORE}/bootstrap_js_embed.h
        ${SRC_CORE}/web_shims_js_embed.h
        ${SRC_CORE}/gui_shims_js_embed.h
)

# ── WebView (header-only on macOS/Linux, WebView2 on Windows) ────────────────
if(ALLOY_ENABLE_WEB)
    if(APPLE)
        # WKWebView — link against WebKit framework
        find_library(WEBKIT_FW WebKit REQUIRED)
        find_library(COCOA_FW  Cocoa  REQUIRED)
        set(WEBVIEW_LIBS ${WEBKIT_FW} ${COCOA_FW})
    elseif(WIN32)
        # WebView2 — statically link WebView2Loader
        find_library(WEBVIEW2_LIB WebView2Loader PATHS ${WEBVIEW_DIR}/build)
        set(WEBVIEW_LIBS ${WEBVIEW2_LIB})
    else()
        # GTK + WebKitGTK
        find_package(PkgConfig REQUIRED)
        pkg_check_modules(GTK3 REQUIRED gtk+-3.0)
        pkg_check_modules(WEBKIT REQUIRED webkit2gtk-4.0)
        set(WEBVIEW_LIBS ${GTK3_LIBRARIES} ${WEBKIT_LIBRARIES})
        set(WEBVIEW_CFLAGS ${GTK3_CFLAGS_OTHER} ${WEBKIT_CFLAGS_OTHER})
    endif()
endif()

# ── Core sources ─────────────────────────────────────────────────────────────
# NOTE: src/core/engine.c and src/core/backend_adapters/mqjs_bridge.c are
# legacy files from the existing codebase. The new Phase 1 architecture uses
# alloy_engine_create.c + mqjs_adapter.c instead.
set(ALLOY_CORE_SOURCES
    ${SRC_CORE}/alloy_engine_create.c
    ${SRC_CORE}/alloy_stdlib.c
    ${SRC_CORE}/capability.c
    ${SRC_CORE}/ipc_channel.c
    ${SRC_CORE}/event_loop.c
    ${SRC_CORE}/embedded_js.c
    ${SRC_CORE}/mqjs_compat.c
    ${SRC_CORE}/backend_adapters/mqjs_adapter.c
    ${SRC_CORE}/plugins/config/config_plugin.c
    ${SRC_CORE}/crypto/x25519.c
    ${SRC_CORE}/crypto/xchacha20poly1305.c
    ${SRC_WEB}/urn.c
    ${SRC_WEB}/did.c
)

if(ALLOY_ENABLE_WEB)
    list(APPEND ALLOY_CORE_SOURCES
        ${SRC_WEB}/web_runtime.c
        ${SRC_CORE}/backend_adapters/webview_adapter.c
    )
endif()

if(ALLOY_ENABLE_GUI)
    list(APPEND ALLOY_CORE_SOURCES
        src/gui/gui_plugin.c
        src/gui/backends/mock_backend.c
    )
endif()

add_library(alloy_core STATIC ${ALLOY_CORE_SOURCES})
add_dependencies(alloy_core gen_stdlib gen_embedded_js)

target_include_directories(alloy_core PUBLIC
    ${INCLUDE_DIR}
    ${MQJS_DIR}
    ${WEBVIEW_DIR}/core/include
    ${SODIUM_INCLUDE}
    ${SRC_CORE}
    ${SRC_WEB}
)

target_link_libraries(alloy_core PUBLIC
    mquickjs
    ${SODIUM_LIB}
)

if(ALLOY_ENABLE_WEB)
    target_compile_definitions(alloy_core PUBLIC ALLOY_ENABLE_WEB=1)
    target_link_libraries(alloy_core PUBLIC ${WEBVIEW_LIBS})
    if(DEFINED WEBVIEW_CFLAGS)
        target_compile_options(alloy_core PUBLIC ${WEBVIEW_CFLAGS})
    endif()
endif()

# ── alloy_host — core process ─────────────────────────────────────────────────
add_executable(alloy_host ${SRC_CORE}/main_host.c)
target_link_libraries(alloy_host PRIVATE alloy_core)

if(DEFINED ALLOY_USER_SCRIPT_HEADER)
    target_compile_definitions(alloy_host PRIVATE
        ALLOY_USER_SCRIPT_HEADER="${ALLOY_USER_SCRIPT_HEADER}"
    )
    message(STATUS "[alloy] embedding user script: ${ALLOY_USER_SCRIPT_HEADER}")
endif()

# ── alloy_gui — Native GUI process (alloy:gui plugin, ALLOY_ENABLE_GUI only) ──
if(ALLOY_ENABLE_GUI)
    add_executable(alloy_gui
        ${SRC_CORE}/ipc_channel.c
        src/gui/alloy_gui_main.c
        src/gui/layout_engine.cpp
        src/gui/backends/mock_backend.c
    )
    target_compile_definitions(alloy_gui PRIVATE ALLOY_MOCK_GUI)
    target_include_directories(alloy_gui PRIVATE
        ${INCLUDE_DIR}
        ${SODIUM_INCLUDE}
    )
    target_link_libraries(alloy_gui PRIVATE
        ${SODIUM_LIB}
    )

    add_executable(alloy_renderer
        ${SRC_CORE}/ipc_channel.c
        src/gui/alloy_renderer_main.c
    )
    target_include_directories(alloy_renderer PRIVATE
        ${INCLUDE_DIR}
        ${SODIUM_INCLUDE}
    )
    target_link_libraries(alloy_renderer PRIVATE
        ${SODIUM_LIB}
    )
endif()

# ── alloy_web — WebView sidecar (alloy:web plugin, ALLOY_ENABLE_WEB only) ─────
if(ALLOY_ENABLE_WEB)
    add_executable(alloy_web ${SRC_WEB}/webview_process.c)
    target_include_directories(alloy_web PRIVATE
        ${INCLUDE_DIR}
        ${WEBVIEW_DIR}/core/include
        ${SODIUM_INCLUDE}
    )
    target_link_libraries(alloy_web PRIVATE
        ${SODIUM_LIB}
        ${WEBVIEW_LIBS}
    )
    if(DEFINED WEBVIEW_CFLAGS)
        target_compile_options(alloy_web PRIVATE ${WEBVIEW_CFLAGS})
    endif()
endif()

# ── Tests ────────────────────────────────────────────────────────────────────
enable_testing()

add_executable(alloy_test_core tests/test_core.c)
target_link_libraries(alloy_test_core PRIVATE alloy_core)
add_test(NAME core_tests COMMAND alloy_test_core)

add_executable(alloy_test_plugins tests/test_plugin_lifecycle.c)
target_link_libraries(alloy_test_plugins PRIVATE alloy_core)
add_test(NAME plugin_lifecycle_tests COMMAND alloy_test_plugins)

if(ALLOY_ENABLE_WEB)
    target_compile_definitions(alloy_test_plugins PRIVATE ALLOY_ENABLE_WEB=1)

    add_executable(alloy_test_web tests/test_web.c)
    target_link_libraries(alloy_test_web PRIVATE alloy_core)
    add_test(NAME web_tests COMMAND alloy_test_web)
endif()

if(ALLOY_ENABLE_GUI)
    target_compile_definitions(alloy_core PUBLIC ALLOY_ENABLE_GUI=1)
    add_executable(alloy_test_gui tests/test_gui_plugin.c)
    target_link_libraries(alloy_test_gui PRIVATE alloy_core)
    add_test(NAME gui_plugin_tests COMMAND alloy_test_gui)

    add_executable(alloy_test_gui_props tests/test_gui_properties.c)
    target_link_libraries(alloy_test_gui_props PRIVATE alloy_core)
    add_test(NAME gui_property_tests COMMAND alloy_test_gui_props)
endif()

# ── Install ──────────────────────────────────────────────────────────────────
install(TARGETS alloy_host DESTINATION bin)
if(ALLOY_ENABLE_WEB)
    install(TARGETS alloy_web DESTINATION bin)
endif()

# ── Summary ──────────────────────────────────────────────────────────────────
message(STATUS "")
message(STATUS "AlloyScript Phase 1 build configuration:")
message(STATUS "  ALLOY_ENABLE_WEB : ${ALLOY_ENABLE_WEB}")
message(STATUS "  LTO              : ${IPO_SUPPORTED}")
message(STATUS "  libsodium        : ${SODIUM_LIB}")
message(STATUS "")
